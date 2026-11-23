//! Manifest coverage shared by the maintained dependency updater.
use super::*;

#[cfg(not(test))]
pub(super) fn tracked_npm_paths(repo: &Path) -> Result<Vec<PathBuf>, String> {
    let output = Command::new("node")
        .arg(repo.join("scripts/dependency-manifest-inventory.mjs"))
        .arg("--repo")
        .arg(repo)
        .output()
        .map_err(|error| format!("cannot observe complete manifest coverage: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cannot observe complete manifest coverage: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let inventory: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("invalid manifest inventory: {error}"))?;
    inventory["npm"]
        .as_array()
        .ok_or_else(|| "manifest inventory requires npm paths".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(|name| repo.join(name))
                .ok_or_else(|| "npm manifest entries must be paths".to_string())
        })
        .collect()
}

pub(super) fn collect_cargo(repo: &Path, dependencies: &mut Vec<Dependency>) -> Result<(), String> {
    let policy_path = repo.join("scripts/dependency-manifests.json");
    let manifests = match fs::read_to_string(&policy_path) {
        Ok(text) => {
            let policy: serde_json::Value = serde_json::from_str(&text)
                .map_err(|error| format!("{}: {error}", policy_path.display()))?;
            policy["activeCargo"]
                .as_array()
                .ok_or_else(|| "manifest policy requires activeCargo".to_string())?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(PathBuf::from)
                        .ok_or_else(|| "activeCargo entries must be paths".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?
        }
        // Existing isolated parser fixtures have no repository policy.
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                && !repo.join("scripts/check-dependencies-latest.rs").exists() =>
        {
            vec![PathBuf::from("rust/Cargo.toml")]
        }
        Err(error) => return Err(format!("{}: {error}", policy_path.display())),
    };
    for manifest in manifests {
        let manifest_path = repo.join(&manifest);
        let text = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
        let lock_path = manifest_path.with_file_name("Cargo.lock");
        let lock_versions = match fs::read_to_string(&lock_path) {
            Ok(text) => parse_cargo_lock(&text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(error) => return Err(format!("{}: {error}", lock_path.display())),
        };
        for (name, declared, line, blocked) in parse_cargo_manifest(&text) {
            let resolved = lock_versions
                .get(&name)
                .cloned()
                .unwrap_or_else(|| base_version(&declared).to_string());
            dependencies.push(Dependency {
                ecosystem: Ecosystem::CratesIo,
                name,
                declared: Some(declared),
                resolved,
                path: manifest_path.clone(),
                line,
                blocked,
            });
        }
    }
    Ok(())
}
