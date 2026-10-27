//! Registry snapshot I/O; action sub-paths share exact authenticated release records.
use super::*;
use std::fmt::Write as _;

/// Read legacy full entries and bounded direct references to shared action releases.
pub fn read_snapshot(path: &Path) -> Option<Snapshot> {
    let text = fs::read_to_string(path).ok()?;
    let mut refreshed_at = String::new();
    let mut entries: BTreeMap<(Ecosystem, String), (String, String)> = BTreeMap::new();
    let mut current: Option<(Ecosystem, String)> = None;
    let mut action_releases = BTreeMap::new();
    let mut aliases = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("refreshed_at ") {
            refreshed_at = quoted_value(rest).unwrap_or_default();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("registry_entry ") {
            let id = rest.trim().to_string();
            current = ecosystem_of(&id).map(|ecosystem| (ecosystem, bare_id(&id, ecosystem)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("release-alias ") {
            let Some((Ecosystem::GitHubAction, name)) = &current else {
                return None;
            };
            if aliases.insert(name.clone(), quoted_value(rest)?).is_some() {
                return None;
            }
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(' ')
            && let Some((Ecosystem::GitHubAction, name)) = &current
            && let Some(value) = quoted_value(value)
        {
            let release = action_releases
                .entry(name.clone())
                .or_insert_with(|| ActionRelease {
                    tag: String::new(),
                    commit: String::new(),
                    source_url: String::new(),
                    tag_source_url: String::new(),
                });
            match key {
                "latest" => release.tag = value,
                "release-commit" | "release_commit" => release.commit = value,
                "source_url" => release.source_url = value,
                "tag-source-url" | "tag_source_url" => release.tag_source_url = value,
                _ => {}
            }
        }
        if let Some(rest) = trimmed.strip_prefix("source_url ")
            && let (Some(key), Some(url)) = (&current, quoted_value(rest))
            && let Some(entry) = entries.get_mut(key)
        {
            entry.1 = url;
        }
        if let Some(rest) = trimmed.strip_prefix("latest ")
            && let (Some(key), Some(latest)) = (&current, quoted_value(rest))
        {
            entries.insert(key.clone(), (latest, String::new()));
        }
    }
    // Only direct references to a complete release of the same publisher qualify.
    // Reject chains and cycles without following them or partially filling records.
    for (name, target) in &aliases {
        let key = (Ecosystem::GitHubAction, name.clone());
        if aliases.contains_key(target)
            || entries.contains_key(&key)
            || action_releases.contains_key(name)
            || action_repository(name) != action_repository(target)
        {
            return None;
        }
        let answer = entries
            .get(&(Ecosystem::GitHubAction, target.clone()))?
            .clone();
        let release = action_releases.get(target)?.clone();
        if !valid_action_release(name, &answer.0, &release) {
            return None;
        }
        entries.insert(key, answer);
        action_releases.insert(name.clone(), release);
    }
    Some(Snapshot {
        refreshed_at,
        action_releases,
        entries,
    })
}

/// Write each shared publisher release once, retaining a named record for every action.
pub fn write_snapshot(path: &Path, snapshot: &Snapshot) {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "# Written by `rust-script scripts/check-dependencies-latest.rs --refresh-snapshot`.\n# One record per registry answer; shared action releases use direct aliases.\ndependency_registry_snapshot\n  record_type \"dependency_registry_snapshot\"\n  refreshed_at \"{}\"",
        snapshot.refreshed_at
    );
    let mut shared_releases = BTreeMap::new();
    for ((ecosystem, name), (latest, url)) in &snapshot.entries {
        let release = snapshot.action_releases.get(name).filter(|release| {
            *ecosystem == Ecosystem::GitHubAction && valid_action_release(name, latest, release)
        });
        if let Some(release) = release {
            let identity = (latest, url, &release.commit, &release.tag_source_url);
            if let Some(target) = shared_releases.get(&identity) {
                let _ = writeln!(
                    text,
                    "registry_entry {}/{}\n  release-alias \"{}\"",
                    ecosystem.key(),
                    name,
                    target
                );
                continue;
            }
            shared_releases.insert(identity, name);
        }
        let _ = writeln!(
            text,
            "registry_entry {}/{}\n  latest \"{latest}\"\n  source_url \"{url}\"",
            ecosystem.key(),
            name
        );
        if let Some(release) = release {
            let _ = writeln!(
                text,
                "  release-commit \"{}\"\n  tag-source-url \"{}\"",
                release.commit, release.tag_source_url
            );
        }
    }
    if let Some(directory) = path.parent() {
        let _ = fs::create_dir_all(directory);
    }
    fs::write(path, text).expect("write the registry snapshot");
}
