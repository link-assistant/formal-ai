//! Issue #1169: every dependency at its latest release, enforced by a gate.
//!
//! The gate lives in `scripts/check-dependencies-latest.rs` and carries its
//! own parser tests; this file pins the whole gate against fixture trees —
//! the collection walk plus the offline comparison plus the `--apply`
//! rewrite — so the pieces cannot drift apart silently.

#[path = "../../../scripts/check-dependencies-latest.rs"]
mod check_dependencies_latest;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use check_dependencies_latest::{
    EXIT_UNVERIFIABLE, Ecosystem, Snapshot, apply_latest, collect, compare, offline_snapshot,
    read_snapshot, write_snapshot,
};

fn fixture_tree(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "issue-1169-gate-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    let _ = fs::remove_dir_all(&root);
    root
}

fn write(path: &Path, body: &str) {
    fs::create_dir_all(path.parent().expect("a parent")).expect("fixture dirs");
    fs::write(path, body).expect("fixture file");
}

fn snapshot_with(entries: Vec<(Ecosystem, &str)>) -> Snapshot {
    Snapshot {
        action_releases: BTreeMap::new(),
        refreshed_at: "2026-09-30T15:04:05Z".to_string(),
        entries: entries
            .into_iter()
            .map(|(ecosystem, name)| {
                let name = name.to_string();
                let latest = latest_of(&name).to_string();
                ((ecosystem, name), (latest, String::new()))
            })
            .collect(),
    }
}

fn latest_of(name: &str) -> &str {
    match name {
        "drifting-crate" => "0.11.0",
        "current-crate" => "4.6.7",
        "blocked-crate" => "0.22.0",
        "react" => "19.3.0",
        "marked" => "18.0.14",
        "@scope/tool" => "2.1.0",
        "zizmorcore/zizmor-action" => "v0.6.4",
        "konard/box-dind" => "2.10.2",
        "rustc" => "1.98.1",
        _ => "0.0.0",
    }
}

fn full_fixture(root: &Path) {
    write(
        &root.join("rust/Cargo.toml"),
        "[package]\nname = \"fixture\"\nrust-version = \"1.98\"\n\n[dependencies]\ncurrent-crate = \"4.6\"\ndrifting-crate = \"0.10\"\nblocked-crate = \"0.16.1\" # blocked: https://github.com/link-foundation/lino-objects-codec/issues/60\n",
    );
    write(
        &root.join("rust/Cargo.lock"),
        "[[package]]\nname = \"current-crate\"\nversion = \"4.6.7\"\n\n[[package]]\nname = \"drifting-crate\"\nversion = \"0.10.0\"\n\n[[package]]\nname = \"blocked-crate\"\nversion = \"0.16.1\"\n",
    );
    write(
        &root.join("package.json"),
        "{\n  \"dependencies\": {\n    \"react\": \"19.3.0\"\n  }\n}\n",
    );
    write(
        &root.join("bun.lock"),
        "{\n  \"packages\": {\n    \"react\": [\"react@19.3.0\", \"\", {}, \"sha512-…\"]\n  }\n}\n",
    );
    // A directory with a manifest but no lockfile: currency falls back to
    // the manifest range's base version.
    write(
        &root.join("tooling/package.json"),
        "{\n  \"dependencies\": {\n    \"marked\": \"^18.0.14\",\n    \"@scope/tool\": \"2.0.0\",\n    \"marked//\": \"blocked: https://github.com/link-assistant/formal-ai/issues/1234\"\n  }\n}\n",
    );
    write(
        &root.join(".github/workflows/ci.yml"),
        "steps:\n  - uses: zizmorcore/zizmor-action@v0.6.4\n",
    );
    write(&root.join("Dockerfile"), "FROM konard/box-dind:2.10.2\n");
}

#[test]
fn offline_gate_flags_drift_and_honors_blocked_annotations() {
    let root = fixture_tree("offline");
    full_fixture(&root);
    let snapshot_path = root.join("data/meta/dependency-registry-snapshot.lino");
    let snapshot = snapshot_with(vec![
        (Ecosystem::CratesIo, "current-crate"),
        (Ecosystem::CratesIo, "drifting-crate"),
        (Ecosystem::CratesIo, "blocked-crate"),
        (Ecosystem::Npm, "react"),
        (Ecosystem::Npm, "marked"),
        (Ecosystem::Npm, "@scope/tool"),
        (Ecosystem::GitHubAction, "zizmorcore/zizmor-action"),
        (Ecosystem::DockerImage, "konard/box-dind"),
        (Ecosystem::RustToolchain, "rustc"),
    ]);
    write_snapshot(&snapshot_path, &snapshot);
    let read = read_snapshot(&snapshot_path).expect("snapshot round-trips");

    let dependencies = collect(&root, None).expect("complete manifest collection");
    let (findings, unverifiable) = compare(&dependencies, &read);

    assert!(unverifiable.is_empty(), "{unverifiable:?}");
    let drifted: Vec<&str> = findings
        .iter()
        .map(|f| f.dependency.name.as_str())
        .collect();
    // blocked-crate is reported but blocked; the stale @scope/tool and the
    // drifted crate fail; everything at its latest stays silent, and the
    // rust-version floor "1.98" matches the 1.98.x channel.
    assert_eq!(
        drifted,
        ["drifting-crate", "blocked-crate", "@scope/tool"],
        "{findings:?}"
    );
    let blocked = findings
        .iter()
        .find(|f| f.dependency.name == "blocked-crate")
        .expect("the blocked pin is reported");
    assert_eq!(
        blocked.dependency.blocked.as_deref(),
        Some("https://github.com/link-foundation/lino-objects-codec/issues/60")
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn apply_moves_unblocked_manifests_and_leaves_blocked_alone() {
    let root = fixture_tree("apply");
    full_fixture(&root);
    let snapshot = snapshot_with(vec![
        (Ecosystem::CratesIo, "current-crate"),
        (Ecosystem::CratesIo, "drifting-crate"),
        (Ecosystem::CratesIo, "blocked-crate"),
        (Ecosystem::Npm, "react"),
        (Ecosystem::Npm, "marked"),
        (Ecosystem::Npm, "@scope/tool"),
        (Ecosystem::GitHubAction, "zizmorcore/zizmor-action"),
        (Ecosystem::DockerImage, "konard/box-dind"),
        (Ecosystem::RustToolchain, "rustc"),
    ]);
    let dependencies = collect(&root, None).expect("complete manifest collection");
    let (findings, _) = compare(&dependencies, &snapshot);
    apply_latest(&findings);

    let manifest = fs::read_to_string(root.join("rust/Cargo.toml")).expect("reread Cargo.toml");
    assert!(
        manifest.contains("drifting-crate = \"0.11.0\""),
        "{manifest}"
    );
    assert!(
        manifest.contains("blocked-crate = \"0.16.1\" # blocked:"),
        "{manifest}"
    );
    let tooling =
        fs::read_to_string(root.join("tooling/package.json")).expect("reread package.json");
    assert!(tooling.contains("\"@scope/tool\": \"2.1.0\""), "{tooling}");
    // The lockfile is the re-resolution step's business, not the gate's.
    let lock = fs::read_to_string(root.join("rust/Cargo.lock")).expect("reread Cargo.lock");
    assert!(lock.contains("version = \"0.10.0\""), "{lock}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn the_repository_tree_collects_every_ecosystem() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repository.join("package.json")).expect("read root package.json"),
    )
    .expect("parse root package.json independently of the dependency collector");
    let declared_react = manifest["dependencies"]["react"]
        .as_str()
        .expect("root package.json declares a React dependency pin");
    let workflow = fs::read_to_string(repository.join(".github/workflows/dependencies-latest.yml"))
        .expect("read the maintained workflow independently of the dependency collector");
    let checkout = workflow
        .lines()
        .find_map(|line| line.trim().strip_prefix("- uses: actions/checkout@"))
        .expect("the maintained workflow checks out the repository")
        .trim_start_matches('v');
    let dependencies = collect(repository, None).expect("complete manifest collection");
    let ecosystems: Vec<Ecosystem> = {
        let mut seen: Vec<Ecosystem> = dependencies.iter().map(|d| d.ecosystem).collect();
        seen.sort();
        seen.dedup();
        seen
    };
    assert_eq!(
        ecosystems,
        [
            Ecosystem::CratesIo,
            Ecosystem::Npm,
            Ecosystem::GitHubAction,
            Ecosystem::DockerImage,
            Ecosystem::RustToolchain,
        ]
    );
    // Spot anchors: the pins this branch itself moved.
    for (ecosystem, name, resolved) in [
        (Ecosystem::DockerImage, "konard/box-dind", "2.10.2"),
        (Ecosystem::GitHubAction, "actions/checkout", checkout),
        (Ecosystem::Npm, "react", declared_react),
    ] {
        assert!(
            dependencies
                .iter()
                .any(|d| d.ecosystem == ecosystem && d.name == name && d.resolved == resolved),
            "missing {name} at {resolved} in the collected set"
        );
    }
    // The current hold-backs are package.json notes (the links-notation
    // Cargo pin was lifted when lino-objects-codec 0.8.0 shipped): each is
    // collected with its tracking reference, never with prose (R1169-3).
    let command_stream = dependencies
        .iter()
        .find(|d| d.name == "command-stream" && d.path.ends_with("desktop/package.json"))
        .expect("desktop declares command-stream");
    assert_eq!(
        command_stream.blocked.as_deref(),
        Some("https://github.com/advisories/GHSA-vfj7-8cjw-p6xm")
    );
    for dependency in dependencies.iter().filter(|d| d.blocked.is_some()) {
        let reference = dependency.blocked.as_deref().unwrap_or_default();
        assert!(
            reference.starts_with("https://")
                && (reference.contains("/issues/") || reference.contains("/advisories/GHSA-")),
            "{} is held back by {reference:?}, which is not a tracking reference",
            dependency.name
        );
    }
    // Moving refs (`dtolnay/rust-toolchain@stable`) are not release pins.
    assert!(
        !dependencies
            .iter()
            .any(|d| d.ecosystem == Ecosystem::GitHubAction && d.resolved == "stable"),
        "a moving branch ref was collected as a release pin"
    );
}

#[test]
fn offline_without_a_snapshot_is_unverifiable_not_clean() {
    let root = fixture_tree("no-snapshot");
    full_fixture(&root);
    let error = offline_snapshot(&root)
        .err()
        .expect("no snapshot was written");
    assert!(error.contains("--refresh-snapshot"), "{error}");
    assert_eq!(EXIT_UNVERIFIABLE, 3);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn active_nested_cargo_manifests_are_collected_without_rewriting_preserved_reproducers() {
    let root = fixture_tree("nested-cargo-policy");
    full_fixture(&root);
    write(
        &root.join("scripts/dependency-manifests.json"),
        r#"{
        "activeCargo": ["rust/Cargo.toml", "tools/registry/Cargo.toml", "tools/unlocked/Cargo.toml"]
    }"#,
    );
    write(
        &root.join("tools/registry/Cargo.toml"),
        "[dependencies]\nregex = \"1\"\n",
    );
    write(
        &root.join("tools/registry/Cargo.lock"),
        "[[package]]\nname = \"regex\"\nversion = \"1.12.9\"\n",
    );
    write(
        &root.join("tools/unlocked/Cargo.toml"),
        "[dependencies]\nserde_json = \"1\"\n",
    );
    write(
        &root.join("historical/Cargo.toml"),
        "[dependencies]\nold = \"0.1\"\n",
    );
    let dependencies = collect(&root, None).expect("all active manifests");
    let nested = dependencies
        .iter()
        .find(|item| item.path == root.join("tools/registry/Cargo.toml"))
        .expect("nested registry dependency");
    assert_eq!(nested.name, "regex");
    assert_eq!(nested.resolved, "1.12.9");
    assert!(
        dependencies
            .iter()
            .any(|item| item.name == "serde_json" && item.resolved == "1")
    );
    assert!(!dependencies.iter().any(|item| item.name == "old"));
    fs::remove_file(root.join("tools/registry/Cargo.toml")).expect("remove active manifest");
    assert!(
        collect(&root, None)
            .expect_err("missing active manifest is not clean")
            .contains("tools/registry/Cargo.toml")
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn checked_npm_inventory_excludes_untracked_packages_and_refuses_missing_tracked_files() {
    let root = fixture_tree("checked-npm");
    full_fixture(&root);
    write(
        &root.join("untracked/package.json"),
        r#"{"dependencies":{"surprise":"1"}}"#,
    );
    assert!(
        collect(&root, None)
            .expect("legacy walk")
            .iter()
            .any(|item| item.name == "surprise")
    );
    let paths = [root.join("package.json")];
    let tracked = collect(&root, Some(&paths)).expect("checked npm inventory");
    assert!(!tracked.iter().any(|item| item.name == "surprise"));
    fs::remove_file(&paths[0]).expect("remove tracked manifest");
    assert!(
        collect(&root, Some(&paths))
            .expect_err("lost file is not a clean observation")
            .contains("package.json")
    );
    let _ = fs::remove_dir_all(root);
}
