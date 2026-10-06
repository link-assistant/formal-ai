//! Embedded suite for `check-dependencies-latest.rs` (`rust-script --test`).
//!
//! Lives beside the script because `scripts/check-file-size.rs` caps a Rust
//! file at 1000 lines; the unit crate compiles it too through the script's
//! own `#[path]` include.

use super::*;

#[test]
fn cargo_manifest_forms() {
    let text = "\
[package]
name = \"formal-ai\"

[dependencies]
base64 = \"0.23\"
clap = { version = \"4.6\", features = [\"derive\"] }
local-crate = { path = \"../local\" }
vendored = { git = \"https://example.com/vendored\" }
inherited = { workspace = true }
links-notation = \"0.16.1\" # blocked: https://github.com/link-foundation/lino-objects-codec/issues/60

[dev-dependencies]
pretty_assertions = \"1.4\"
";
    let parsed = parse_cargo_manifest(text);
    let names: Vec<&str> = parsed.iter().map(|(n, _, _, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["base64", "clap", "links-notation", "pretty_assertions"]
    );
    assert_eq!(parsed[0].1, "0.23");
    assert_eq!(parsed[0].2, 5);
    assert_eq!(parsed[1].1, "4.6");
    assert_eq!(
        parsed[2].3.as_deref(),
        Some("https://github.com/link-foundation/lino-objects-codec/issues/60")
    );
}

#[test]
fn cargo_lock_pairs() {
    let text = "\
[[package]]
name = \"base64\"
version = \"0.23.0\"

[[package]]
name = \"clap\"
version = \"4.6.7\"
";
    let versions = parse_cargo_lock(text);
    assert_eq!(versions.get("clap").map(String::as_str), Some("4.6.7"));
}

#[test]
fn package_json_members_and_blocked_notes() {
    let text = "\
{
  \"name\": \"fixture\",
  \"dependencies\": {
    \"react\": \"19.3.0\",
    \"electron\": \"^44.1.0\",
    \"electron//\": \"blocked: https://github.com/link-assistant/formal-ai/issues/1169\"
  },
  \"devDependencies\": {
    \"playwright\": \"^1.63.0\"
  }
}
";
    let parsed = parse_package_json(text);
    let names: Vec<&str> = parsed.iter().map(|(n, _, _, _)| n.as_str()).collect();
    assert_eq!(names, ["react", "electron", "playwright"]);
    assert_eq!(parsed[0].1, "19.3.0");
    assert_eq!(parsed[1].2, 5);
    assert_eq!(
        parsed[1].3.as_deref(),
        Some("https://github.com/link-assistant/formal-ai/issues/1169")
    );
}

#[test]
fn package_lock_and_bun_lock_resolutions() {
    let npm_lock = "\
{
  \"packages\": {
    \"\": {},
    \"node_modules/react\": {
      \"version\": \"19.3.0\",
      \"resolved\": \"https://registry.npmjs.org/react/-/react-19.3.0.tgz\"
    },
    \"node_modules/react/node_modules/scheduler\": {
      \"version\": \"0.28.0\"
    }
  }
}
";
    let versions = parse_package_lock(npm_lock);
    assert_eq!(versions.get("react").map(String::as_str), Some("19.3.0"));
    assert!(!versions.contains_key("react@scheduler"));

    let bun_lock = "\
{
  \"workspaces\": { \"\": { \"dependencies\": { \"react\": \"19.3.0\" } } },
  \"packages\": {
    \"react\": [\"react@19.3.0\", \"\", {}, \"sha512-…\"],
    \"@scope/tool\": [\"@scope/tool@2.1.0\", \"\", {}, \"sha512-…\"]
  }
}
";
    let versions = parse_bun_lock(bun_lock);
    assert_eq!(versions.get("react").map(String::as_str), Some("19.3.0"));
    assert_eq!(
        versions.get("@scope/tool").map(String::as_str),
        Some("2.1.0")
    );
}

#[test]
fn uses_and_from_pins() {
    let workflow = "\
steps:
  - uses: actions/checkout@v4
  - uses: ./local/action
  - uses: zizmorcore/zizmor-action@v0.6.4
";
    let pins = parse_uses_pins(workflow);
    assert_eq!(pins.len(), 2);
    assert_eq!(
        pins[1],
        (
            "zizmorcore/zizmor-action".to_string(),
            "v0.6.4".to_string(),
            4
        )
    );

    let dockerfile = "\
FROM ubuntu:24.04 AS build
FROM ${BINARY_SOURCE}-binary AS selected-binary
FROM konrad/box-dind:2.10.2
";
    let bases = parse_dockerfile_from(dockerfile);
    assert_eq!(bases.len(), 2);
    assert_eq!(
        bases[1],
        ("konrad/box-dind".to_string(), "2.10.2".to_string(), 3)
    );
}

#[test]
fn blocked_annotations_must_name_an_issue() {
    assert_eq!(
        blocked_annotation("x = \"1\" # blocked: https://github.com/o/r/issues/7"),
        Some("https://github.com/o/r/issues/7".to_string())
    );
    assert_eq!(blocked_annotation("x = \"1\" # blocked: not-a-url"), None);
    assert_eq!(blocked_annotation("x = \"1\""), None);
}

#[test]
fn ranges_and_minors() {
    assert_eq!(base_version("^1.63.0"), "1.63.0");
    assert_eq!(base_version("=0.3.0"), "0.3.0");
    assert_eq!(base_version("~2.4"), "2.4");
    assert_eq!(minor_of("1.98.1"), "1.98");
    assert_eq!(minor_of("v7.0.1"), "7.0");
}

#[test]
fn comparison_rules() {
    let snapshot = Snapshot {
        refreshed_at: String::new(),
        entries: [
            (
                (Ecosystem::CratesIo, "clap".to_string()),
                ("4.6.7".to_string(), String::new()),
            ),
            (
                (Ecosystem::CratesIo, "egg".to_string()),
                ("0.11.0".to_string(), String::new()),
            ),
            (
                (Ecosystem::Npm, "react".to_string()),
                ("19.3.0".to_string(), String::new()),
            ),
            (
                (Ecosystem::RustToolchain, "rustc".to_string()),
                ("1.98.1".to_string(), String::new()),
            ),
        ]
        .into_iter()
        .collect(),
    };
    let dependency = |ecosystem: Ecosystem, name: &str, resolved: &str| Dependency {
        ecosystem,
        name: name.to_string(),
        declared: Some("0".to_string()),
        resolved: resolved.to_string(),
        path: PathBuf::from("manifest"),
        line: 1,
        blocked: None,
    };
    let dependencies = vec![
        dependency(Ecosystem::CratesIo, "clap", "4.6.7"),
        dependency(Ecosystem::CratesIo, "egg", "0.10.0"),
        dependency(Ecosystem::Npm, "react", "19.3.0"),
        dependency(Ecosystem::Npm, "absent", "1.0.0"),
        dependency(Ecosystem::RustToolchain, "rustc", "1.98"),
    ];
    let (findings, unverifiable) = compare(&dependencies, &snapshot);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].dependency.name, "egg");
    assert_eq!(unverifiable, ["npm/absent"]);
}

#[test]
fn snapshot_round_trip() {
    let directory =
        std::env::temp_dir().join(format!("issue-1169-snapshot-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("temp dir");
    let path = directory.join("snapshot.lino");
    let snapshot = Snapshot {
        refreshed_at: "2026-09-30T15:04:05Z".to_string(),
        entries: [
            (
                (Ecosystem::CratesIo, "clap".to_string()),
                (
                    "4.6.7".to_string(),
                    "https://crates.io/api/v1/crates/clap".to_string(),
                ),
            ),
            (
                (Ecosystem::Npm, "@scope/tool".to_string()),
                (
                    "2.1.0".to_string(),
                    "https://registry.npmjs.org/@scope/tool".to_string(),
                ),
            ),
        ]
        .into_iter()
        .collect(),
    };
    write_snapshot(&path, &snapshot);
    let read = read_snapshot(&path).expect("read back");
    assert_eq!(
        read.entries
            .get(&(Ecosystem::Npm, "@scope/tool".to_string()))
            .map(|(v, _)| v.clone()),
        Some("2.1.0".to_string())
    );
    assert_eq!(
        read.entries
            .get(&(Ecosystem::CratesIo, "clap".to_string()))
            .map(|(v, _)| v.clone()),
        Some("4.6.7".to_string())
    );
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn apply_rewrites_manifests_not_locks() {
    let text = "\
[dependencies]
clap = { version = \"4.6\", features = [\"derive\"] }
egg = \"0.10.0\"
";
    let dependency = |name: &str, declared: &str, line: usize| Dependency {
        ecosystem: Ecosystem::CratesIo,
        name: name.to_string(),
        declared: Some(declared.to_string()),
        resolved: "0.10.0".to_string(),
        path: PathBuf::from("Cargo.toml"),
        line,
        blocked: None,
    };
    let rewritten = rewrite_cargo_line(text, &dependency("clap", "4.6", 2), "4.6.7");
    assert!(
        rewritten.contains("clap = { version = \"4.6.7\","),
        "{rewritten}"
    );
    let rewritten = rewrite_cargo_line(text, &dependency("egg", "0.10.0", 3), "0.11.0");
    assert!(rewritten.contains("egg = \"0.11.0\""), "{rewritten}");

    let json = "{\n  \"dependencies\": {\n    \"marked\": \"^18.0.11\"\n  }\n}\n";
    let dependency = Dependency {
        ecosystem: Ecosystem::Npm,
        name: "marked".to_string(),
        declared: Some("^18.0.11".to_string()),
        resolved: "18.0.11".to_string(),
        path: PathBuf::from("package.json"),
        line: 3,
        blocked: None,
    };
    let rewritten = rewrite_npm_line(json, &dependency, "18.0.14");
    assert!(
        rewritten.contains("\"marked\": \"^18.0.14\""),
        "{rewritten}"
    );
}

#[test]
fn apply_leaves_blocked_lines_alone() {
    let blocked = Dependency {
        ecosystem: Ecosystem::CratesIo,
        name: "links-notation".to_string(),
        declared: Some("0.16.1".to_string()),
        resolved: "0.16.1".to_string(),
        path: PathBuf::from("Cargo.toml"),
        line: 3,
        blocked: Some(
            "https://github.com/link-foundation/lino-objects-codec/issues/60".to_string(),
        ),
    };
    let text = "[dependencies]\nlinks-notation = \"0.16.1\" # blocked: https://github.com/link-foundation/lino-objects-codec/issues/60\n";
    let rewritten = rewrite_cargo_line(text, &blocked, "0.22.0");
    assert_eq!(rewritten, text);
}
