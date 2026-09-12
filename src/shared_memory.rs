use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::PathBuf;

pub const MEMORY_PATH_ENV: &str = "FORMAL_AI_MEMORY_PATH";
pub const MEMORY_DIRECTORY_NAME: &str = ".formal-ai";
pub const MEMORY_FILE_NAME: &str = "memory.lino";

/// Where a test binary keeps its memory store.
///
/// With no `FORMAL_AI_MEMORY_PATH`, the fallback below is
/// `$HOME/.formal-ai/memory.lino` -- the developer's own store. A test suite
/// resolving that had two consequences, both observed:
///
/// * Every test in a binary shared one store, so the parallel rebuilds of
///   `<memory>.seed.links` in `seed_links::mirror()` interleaved. The size
///   fields of the size-balanced tree stopped agreeing, `fix_size`
///   (`left + right + 1`) overflowed inside `platform-trees`, and the process
///   aborted during unwinding -- blaming whichever test was running, which then
///   passed in isolation.
/// * The suite left a 64 MB `memory.links` and a 10 MB `memory.lino` in the
///   home directory of the machine that ran it.
///
/// Under `cargo test` the fallback is therefore a directory private to the
/// process. An explicit `FORMAL_AI_MEMORY_PATH` always wins, so a test that
/// wants a specific store still gets one.
#[must_use]
pub fn test_memory_directory() -> Option<PathBuf> {
    if !running_under_cargo_test() {
        return None;
    }
    Some(std::env::temp_dir().join(format!("formal-ai-tests-{}", std::process::id())))
}

/// Set by a test harness to say "this process is a test".
///
/// Inferring it does not work. `CARGO_TARGET_TMPDIR` is a *compile-time*
/// variable cargo passes to integration tests and is absent at runtime;
/// `cfg!(test)` is false because the library is built as a plain dependency of
/// each integration-test binary. Both compile and both silently answer "no".
///
/// Inferring it from the executable path does not work either, and that failure
/// is worth recording because it passed locally and failed in CI. Cargo builds
/// test binaries into `target/<profile>/deps/`, so that looked decisive -- until
/// issue #1055 had the shared build job compile them once and
/// `scripts/run-prebuilt-tests.sh` run them from `dist/tests/unit`, where
/// nothing about the path says "test":
///
/// ```text
/// current_exe Ok("/home/runner/work/formal-ai/formal-ai/dist/tests/unit")
/// ```
///
/// What both layouts do share is the name cargo gives the binary.
/// `dist/tests/unit` and `target/debug/deps/unit-<hash>` differ in every
/// component except that final `unit`, so the check is on the file name, with
/// `deps/` still accepted for the per-file integration binaries whose names are
/// not known in advance. A harness may also say so outright via
/// `declare_test_process()`, which is the escape hatch when a new layout
/// appears rather than another silent "no".
static IS_TEST_PROCESS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The test targets this repository builds (`Cargo.toml` `[[test]]` names) plus
/// the per-file integration binaries, which cargo names after their source.
const TEST_TARGET_NAMES: [&str; 3] = ["unit", "integration", "source"];

/// Declare that this process is a test harness.
///
/// Available for a layout the inference below does not recognise. Calling it is
/// always correct for a test binary and never correct for a shipped one.
pub fn declare_test_process() {
    IS_TEST_PROCESS.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[must_use]
pub fn running_under_cargo_test() -> bool {
    if IS_TEST_PROCESS.load(std::sync::atomic::Ordering::Relaxed) {
        return true;
    }
    std::env::current_exe()
        .ok()
        .as_deref()
        .is_some_and(is_test_executable)
}

/// Whether an executable path names a libtest harness this repository builds.
///
/// Covers both layouts in use: `target/<profile>/deps/<target>-<hash>` from a
/// plain `cargo test`, and `dist/tests/<target>` from the prebuilt binaries
/// issue #1055 introduced. The shipped binary is `formal-ai`, which is not a
/// test target name, so it is never matched.
#[must_use]
pub fn is_test_executable(executable: &std::path::Path) -> bool {
    let Some(name) = executable.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    // `cargo test` appends `-<hash>`; the prebuilt copies do not.
    let target = name.split_once('-').map_or(name, |(stem, _)| stem);
    TEST_TARGET_NAMES.contains(&target)
        || executable
            .parent()
            .and_then(std::path::Path::file_name)
            .is_some_and(|parent| parent == OsStr::new("deps"))
}

#[must_use]
pub fn resolve_memory_path_from(
    memory_path: Option<&OsStr>,
    home: Option<&OsStr>,
    app_data: Option<&OsStr>,
    windows: bool,
) -> PathBuf {
    if let Some(path) = memory_path
        .filter(|value| !value.to_string_lossy().trim().is_empty())
        .map(PathBuf::from)
    {
        return path;
    }

    if windows {
        return app_data
            .or(home)
            .map_or_else(|| PathBuf::from("."), PathBuf::from)
            .join("formal-ai")
            .join(MEMORY_FILE_NAME);
    }

    home.map_or_else(|| PathBuf::from("."), PathBuf::from)
        .join(MEMORY_DIRECTORY_NAME)
        .join(MEMORY_FILE_NAME)
}

#[must_use]
pub fn shared_memory_path() -> PathBuf {
    let configured = std::env::var_os(MEMORY_PATH_ENV);
    let home = std::env::var_os("HOME");
    // `resolve_memory_path_from` stays pure -- `tests/issue_756.rs` pins the
    // exact path it derives from a given HOME -- so the test-binary fallback is
    // applied here, where the environment is already being read.
    //
    // Only the *unmanaged* default is redirected. A test that has already
    // pointed `FORMAL_AI_MEMORY_PATH` or `HOME` somewhere of its own has said
    // where it wants the store, and overriding that would break tests doing
    // exactly the right thing (`tests/issue_756.rs` moves `HOME` to a temporary
    // directory and asserts the store appears under it).
    if configured
        .as_deref()
        .is_none_or(|value| value.to_string_lossy().trim().is_empty())
        && !is_relocated_home(home.as_deref())
        && let Some(directory) = test_memory_directory()
    {
        return directory.join(MEMORY_FILE_NAME);
    }
    resolve_memory_path_from(
        configured.as_deref(),
        home.as_deref(),
        std::env::var_os("APPDATA").as_deref(),
        cfg!(windows),
    )
}

/// Whether `HOME` has been pointed at a scratch directory.
///
/// A test that relocates `HOME` puts it under the system temporary directory
/// (`tests/issue_756.rs` uses `std::env::temp_dir().join(...)`), while a real
/// home directory is never there. That is the distinction the redirect needs:
/// a test which has already said where it wants the store must keep it, and
/// only the unmanaged default gets moved.
///
/// Caching this in a `OnceLock` would not work -- the first call can happen
/// inside the very scope that moved `HOME` -- so it is recomputed each time.
#[must_use]
fn is_relocated_home(home: Option<&OsStr>) -> bool {
    home.is_some_and(|home| std::path::Path::new(home).starts_with(std::env::temp_dir()))
}

pub fn ensure_shared_memory_file(path: &std::path::Path) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        #[cfg(unix)]
        let existed = parent.exists();
        fs::create_dir_all(parent)?;
        #[cfg(unix)]
        if !existed {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
    }
    OpenOptions::new().create(true).append(true).open(path)?;
    Ok(())
}
