use formal_ai::provenance_native::{CpuHashMode, digest};
use formal_ai::provenance_snapshot::{
    SnapshotLimits, SourceFile, cpu_backend, language_extensions, merkle_proof, merkle_root,
    snapshot_repository, source_snapshot, verify_merkle_proof,
};
use serde_json::Value;

fn fixtures() -> Value {
    serde_json::from_str(include_str!("../fixtures/provenance-snapshot.json"))
        .expect("declared snapshot vectors")
}
fn extensions() -> Vec<(String, String)> {
    language_extensions(include_str!(
        "../../embedded/data/seed/page-formalization-rules.lino"
    ))
    .expect("maintained language table")
}
#[test]
fn canonical_snapshot_matches_original_javascript_roots_and_mixed_changes() {
    let fixture = fixtures();
    let rows = fixture["rows"].as_array().expect("declared rows");
    assert_eq!(rows.len(), 5);
    let mut totals = std::collections::HashSet::new();
    for row in rows {
        let files: Vec<_> = row["files"]
            .as_array()
            .expect("files")
            .iter()
            .map(|file| SourceFile {
                path: file["path"].as_str().expect("path").to_owned(),
                content: file["content"]
                    .as_str()
                    .expect("source bytes")
                    .as_bytes()
                    .to_vec(),
            })
            .collect();
        let result =
            source_snapshot(&files, &extensions(), CpuHashMode::Auto).expect("canonical snapshot");
        let result = serde_json::to_value(result).expect("snapshot JSON");
        assert_eq!(result, row["expected"], "{}", row["name"]);
        assert!(totals.insert(result["total"].as_str().expect("root").to_owned()));
    }
    assert_ne!(
        rows[1]["expected"]["languages"][1]["root"],
        rows[2]["expected"]["languages"][1]["root"]
    );
    assert_eq!(
        rows[1]["expected"]["languages"][0]["root"],
        rows[2]["expected"]["languages"][0]["root"]
    );
    assert_ne!(
        rows[1]["expected"]["languages"][0]["root"],
        rows[3]["expected"]["languages"][0]["root"]
    );
    assert_eq!(
        rows[1]["expected"]["languages"][1]["root"],
        rows[3]["expected"]["languages"][1]["root"]
    );
}
#[test]
fn canonical_inclusion_siblings_and_roots_match_original_javascript() {
    let fixture = fixtures();
    let rows = fixture["proofs"].as_array().expect("proof rows");
    assert_eq!(rows.len(), 3);
    for row in rows {
        let leaves: Vec<_> = row["inputs"]
            .as_array()
            .expect("leaf inputs")
            .iter()
            .map(|value| {
                digest(
                    value.as_str().expect("leaf text").as_bytes(),
                    CpuHashMode::Auto,
                )
            })
            .collect();
        assert_eq!(
            blake3::Hash::from(merkle_root(&leaves, CpuHashMode::Auto).expect("root"))
                .to_hex()
                .to_string(),
            row["root"]
        );
        for (selected, expected) in row["proofs"].as_array().expect("proofs").iter().enumerate() {
            assert_eq!(
                serde_json::to_value(
                    merkle_proof(&leaves, selected, CpuHashMode::Auto).expect("proof")
                )
                .expect("proof JSON"),
                *expected
            );
        }
        let expected_root = merkle_root(&leaves, CpuHashMode::Auto).expect("root");
        for selected in 0..leaves.len() {
            let mut proof = merkle_proof(&leaves, selected, CpuHashMode::Auto).expect("proof");
            assert!(
                verify_merkle_proof(leaves[selected], &proof, expected_root, CpuHashMode::Auto)
                    .expect("verify")
            );
            assert!(
                !verify_merkle_proof(
                    digest(b"foreign", CpuHashMode::Auto),
                    &proof,
                    expected_root,
                    CpuHashMode::Auto
                )
                .expect("wrong leaf")
            );
            proof.siblings.push("0".repeat(64));
            assert!(
                !verify_merkle_proof(leaves[selected], &proof, expected_root, CpuHashMode::Auto)
                    .expect("extra sibling")
            );
            let _ = proof.siblings.pop();
            if !proof.siblings.is_empty() {
                proof.siblings[0] = "0".repeat(64);
                assert!(
                    !verify_merkle_proof(
                        leaves[selected],
                        &proof,
                        expected_root,
                        CpuHashMode::Auto
                    )
                    .expect("tampered sibling")
                );
            }
        }
        assert!(merkle_proof(&leaves, leaves.len(), CpuHashMode::Auto).is_err());
    }
}
#[test]
fn native_snapshot_backend_and_path_refusals_are_explicit() {
    assert_eq!(
        cpu_backend("auto").expect("CPU fallback"),
        CpuHashMode::Auto
    );
    assert_eq!(cpu_backend("cpu").expect("CPU"), CpuHashMode::Auto);
    assert!(cpu_backend("gpu").is_err());
    assert!(cpu_backend("AUTO").is_err());
    for path in ["../a", "/a", "a//b", "a\\b", "a/./b"] {
        assert!(
            source_snapshot(
                &[SourceFile {
                    path: path.to_owned(),
                    content: Vec::new()
                }],
                &extensions(),
                CpuHashMode::Auto
            )
            .is_err()
        );
    }
    let file = || SourceFile {
        path: "a.rs".to_owned(),
        content: Vec::new(),
    };
    assert!(source_snapshot(&[file(), file()], &extensions(), CpuHashMode::Auto).is_err());
}
#[test]
fn native_repository_snapshot_retains_all_files_and_refuses_budgets() {
    struct OwnedWorkspace(std::path::PathBuf);
    impl Drop for OwnedWorkspace {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let workspace = OwnedWorkspace(
        std::env::temp_dir().join(format!("formal-ai-snapshot-{}-{nonce}", std::process::id())),
    );
    std::fs::create_dir(&workspace.0).expect("exclusive private fixture directory");
    let root = workspace.0.as_path();
    let git = |arguments: &[&str]| {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(arguments)
            .status()
            .expect("Git fixture");
        assert!(status.success());
    };
    git(&["init", "--quiet"]);
    std::fs::write(root.join("a.rs"), b"fn main() {}\n").expect("Rust fixture");
    std::fs::write(root.join("b.js"), b"export const answer = 17;\n").expect("JS fixture");
    git(&["add", "a.rs", "b.js"]);
    let result =
        snapshot_repository(root, "auto", SnapshotLimits::default()).expect("bounded snapshot");
    assert_eq!(result["inventory"]["file_count"], 2);
    assert_eq!(result["inventory"]["exclusions"], serde_json::json!([]));
    assert_eq!(
        result["languages"]
            .as_array()
            .expect("language roots")
            .len(),
        2
    );
    assert!(
        snapshot_repository(
            root,
            "cpu",
            SnapshotLimits {
                max_files: 1,
                ..SnapshotLimits::default()
            }
        )
        .is_err()
    );
    assert!(
        snapshot_repository(
            root,
            "cpu",
            SnapshotLimits {
                max_file_bytes: 0,
                ..SnapshotLimits::default()
            }
        )
        .is_err()
    );
    assert!(
        snapshot_repository(
            root,
            "cpu",
            SnapshotLimits {
                max_total_bytes: 0,
                ..SnapshotLimits::default()
            }
        )
        .is_err()
    );
    assert!(snapshot_repository(root, "gpu", SnapshotLimits::default()).is_err());
    std::fs::create_dir(root.join("nested")).expect("nested directory");
    assert!(snapshot_repository(&root.join("nested"), "cpu", SnapshotLimits::default()).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("a.rs"), root.join("link.rs"))
            .expect("symlink fixture");
        git(&["add", "link.rs"]);
        assert!(snapshot_repository(root, "cpu", SnapshotLimits::default()).is_err());
    }
}
