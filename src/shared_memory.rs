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

/// Whether this process is a `cargo test` binary.
///
/// The obvious signals do not work here. `CARGO_TARGET_TMPDIR` is a
/// *compile-time* variable cargo passes to integration tests; it is absent from
/// the runtime environment, which was measured on this repository:
///
/// ```text
/// CARGO_TARGET_TMPDIR = None
/// exe = Some(".../target/debug/deps/probe_env-70a1c6defe99c803")
/// ```
///
/// `cfg!(test)` is false too, because the library is compiled as a plain
/// dependency of each integration-test binary rather than in test mode.
///
/// What is left is the executable itself. Cargo builds every test binary into
/// `target/<profile>/deps/` with a hash suffix, and installs nothing there, so
/// a parent directory named `deps` whose grandparent is `target` identifies a
/// test (or bench) binary and not a shipped one.
#[must_use]
pub fn running_under_cargo_test() -> bool {
    std::env::current_exe()
        .ok()
        .as_deref()
        .and_then(std::path::Path::parent)
        .is_some_and(is_cargo_deps_directory)
}

#[must_use]
pub fn is_cargo_deps_directory(directory: &std::path::Path) -> bool {
    directory.file_name() == Some(OsStr::new("deps"))
        && directory
            .parent()
            .and_then(std::path::Path::parent)
            .and_then(std::path::Path::file_name)
            == Some(OsStr::new("target"))
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
