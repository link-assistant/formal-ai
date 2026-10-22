//! Explicit caller-installed source operation capabilities, never task metadata.
//!
//! The caller trusts the installed resource home, including imported dependencies.
//! Digests bind Node and the two entrypoints, not the transitive resource closure.
use std::sync::OnceLock;

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct InstalledSourceCommands {
    node_executable: String,
    resource_home: String,
    node_sha256: String,
    source_cst_transform_sha256: String,
    unary_consolidation_sha256: String,
}
static INSTALLED: OnceLock<Option<InstalledSourceCommands>> = OnceLock::new();

/// Configure once from the trusted server caller, before serving any requests.
/// Resources are optional: absence grants no process capability.
pub fn configure_installed_source_commands(json: Option<&str>) -> Result<(), String> {
    let installed = json
        .map(|text| {
            if text.len() > 16_384 {
                return Err("source_command_configuration_limit".to_owned());
            }
            let value: InstalledSourceCommands = serde_json::from_str(text)
                .map_err(|_| "source_command_configuration_invalid".to_owned())?;
            validate(&value).ok_or_else(|| "source_command_installation_invalid".to_owned())?;
            Ok(value)
        })
        .transpose()?;
    if let Some(existing) = INSTALLED.get() {
        return if existing == &installed {
            Ok(())
        } else {
            Err("source_command_already_configured".to_owned())
        };
    }
    INSTALLED
        .set(installed)
        .map_err(|_| "source_command_configuration_race".to_owned())
}

/// A fixed operation selects only its caller-installed, revalidated executable.
#[must_use]
pub fn installed_source_command(operation: &str) -> Option<[String; 2]> {
    let installed = INSTALLED.get()?.as_ref()?;
    validate(installed)?;
    let relative = match operation {
        "source-cst-transform" => "scripts/source-cst-transform-request.mjs",
        "unary-consolidation" => "scripts/consolidate-stateless-unary-request.mjs",
        _ => return None,
    };
    Some([
        installed.node_executable.clone(),
        std::path::Path::new(&installed.resource_home)
            .join(relative)
            .to_str()?
            .to_owned(),
    ])
}

#[cfg(target_arch = "wasm32")]
fn validate(_: &InstalledSourceCommands) -> Option<()> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn validate(installed: &InstalledSourceCommands) -> Option<()> {
    use std::path::Path;
    let home = Path::new(&installed.resource_home);
    if !home.is_absolute()
        || std::fs::canonicalize(home).ok()?.as_path() != home
        || !std::fs::symlink_metadata(home).ok()?.is_dir()
    {
        return None;
    }
    check_file(
        Path::new(&installed.node_executable),
        &installed.node_sha256,
        true,
    )?;
    for (relative, digest) in [
        (
            "scripts/source-cst-transform-request.mjs",
            &installed.source_cst_transform_sha256,
        ),
        (
            "scripts/consolidate-stateless-unary-request.mjs",
            &installed.unary_consolidation_sha256,
        ),
    ] {
        check_file(&home.join(relative), digest, false)?;
    }
    Some(())
}

#[cfg(not(target_arch = "wasm32"))]
fn check_file(path: &std::path::Path, expected: &str, executable: bool) -> Option<()> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    if expected.len() != 64
        || !expected
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || !path.is_absolute()
        || std::fs::canonicalize(path).ok()?.as_path() != path
    {
        return None;
    }
    let mut file = crate::provenance_snapshot::open_observed(path).ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.len() > 256 * 1024 * 1024 {
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if executable && metadata.permissions().mode() & 0o111 == 0 {
            return None;
        }
    }
    #[cfg(not(unix))]
    let _ = executable;
    let mut digest = Sha256::new();
    let mut observed = 0_u64;
    let mut buffer = vec![0_u8; 65_536];
    for _ in 0..=256 * 1024 * 1024 {
        let count = file.read(&mut buffer).ok()?;
        if count == 0 {
            break;
        }
        observed = observed.checked_add(count as u64)?;
        if observed > 256 * 1024 * 1024 {
            return None;
        }
        digest.update(&buffer[..count]);
    }
    if observed != metadata.len()
        || crate::source_fetch::hex_lower(&digest.finalize()) != expected
        || !crate::provenance_snapshot::same_observed_file(&file, path).ok()?
    {
        return None;
    }
    Some(())
}
