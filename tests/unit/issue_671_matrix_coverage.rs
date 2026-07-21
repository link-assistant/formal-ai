//! The multi-CLI end-to-end matrix has to cover *every* client we ship, not the
//! handful somebody remembered (issue #671).
//!
//! PR #648 closed #647 with `claude` "intentionally not run" and `grok`/`aider`
//! "inferred from the shared adapters"; hands-on testing then produced issue
//! #650 with four defects. These guards make that failure mode impossible to
//! repeat: adding a client to `data/seed/client-integrations.lino` without a
//! pinned version, a CI leg and a documented row fails the build.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use formal_ai::seed::client_integrations;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let path = root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn seeded_ids() -> Vec<String> {
    client_integrations()
        .iter()
        .map(|integration| integration.id.clone())
        .collect()
}

/// Client ids pinned in the lockfile, in file order.
fn locked_order() -> Vec<String> {
    read("experiments/agentic_cli_matrix/clients.lock")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let id = fields.next()?;
            // A row is only a pin if it also names an installer and a spec.
            (fields.next().is_some() && fields.next().is_some()).then(|| id.to_owned())
        })
        .collect()
}

fn locked_ids() -> BTreeSet<String> {
    locked_order().into_iter().collect()
}

#[test]
fn agentic_cli_matrix_covers_every_seeded_client() {
    let locked = locked_ids();
    let missing: Vec<_> = seeded_ids()
        .into_iter()
        .filter(|id| !locked.contains(id))
        .collect();

    assert!(
        missing.is_empty(),
        "clients missing a pinned version in experiments/agentic_cli_matrix/clients.lock: {missing:?}"
    );
}

#[test]
fn every_pinned_client_is_a_real_seeded_client() {
    let seeded: BTreeSet<String> = seeded_ids().into_iter().collect();
    let stale: Vec<_> = locked_ids()
        .into_iter()
        .filter(|id| !seeded.contains(id))
        .collect();

    // A pin for a client we no longer ship would install a CLI nothing drives.
    assert!(
        stale.is_empty(),
        "clients.lock pins ids that are not in the seed registry: {stale:?}"
    );
}

#[test]
fn every_seeded_client_has_a_ci_leg() {
    let workflow = read(".github/workflows/agentic-cli-matrix.yml");
    let missing: Vec<_> = seeded_ids()
        .into_iter()
        .filter(|id| !workflow.contains(&format!("- client: {id}\n")))
        .collect();

    assert!(
        missing.is_empty(),
        "clients with no leg in .github/workflows/agentic-cli-matrix.yml: {missing:?}"
    );
}

#[test]
fn every_seeded_client_has_a_documented_matrix_row() {
    let guide = read("docs/testing/agentic-cli-tools.md");
    let missing: Vec<_> = seeded_ids()
        .into_iter()
        .filter(|id| !guide.contains(&format!("| `{id}` |")))
        .collect();

    assert!(
        missing.is_empty(),
        "clients missing a row in the docs/testing/agentic-cli-tools.md matrix table: {missing:?}"
    );
}

#[test]
fn every_ci_leg_gets_its_own_port_range() {
    let workflow = read(".github/workflows/agentic-cli-matrix.yml");
    let ports: Vec<u32> = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("base_port: "))
        .filter_map(|value| value.trim().parse().ok())
        .collect();

    assert_eq!(
        ports.len(),
        seeded_ids().len(),
        "every leg needs a base_port: {ports:?}"
    );
    let unique: BTreeSet<_> = ports.iter().copied().collect();
    assert_eq!(unique.len(), ports.len(), "duplicate base_port: {ports:?}");

    // Each leg starts a server and a proxy per case, and `run_leg.sh` spaces its
    // cases 10 ports apart, so neighbouring legs must not overlap.
    let ordered: Vec<_> = unique.into_iter().collect();
    for pair in ordered.windows(2) {
        assert!(
            pair[1] - pair[0] >= 50,
            "leg port ranges overlap: {} and {}",
            pair[0],
            pair[1]
        );
    }
}

/// CI runs the legs in parallel; `run_matrix.sh` runs them one after another on
/// a single machine and derives each port from the client's lockfile position.
/// The two must agree, or a local reproduction of a red leg quietly drives a
/// different port than CI did.
#[test]
fn ci_leg_ports_match_the_local_runner_formula() {
    let workflow = read(".github/workflows/agentic-cli-matrix.yml");
    for (index, id) in locked_order().iter().enumerate() {
        let expected = 8900 + index * 60;
        let leg = workflow
            .split("- client: ")
            .find(|chunk| chunk.starts_with(&format!("{id}\n")))
            .unwrap_or_else(|| panic!("no CI leg for {id}"));
        assert!(
            leg.contains(&format!("base_port: {expected}\n")),
            "leg {id} must use base_port {expected} (position {index} in clients.lock)"
        );
    }
}

/// Recorded transcripts committed under `recorded/`.
fn recorded_transcripts() -> Vec<(String, PathBuf)> {
    let dir = root().join("experiments/agentic_cli_matrix/recorded");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for client_dir in entries.flatten() {
        let client = client_dir.file_name().to_string_lossy().into_owned();
        let Ok(files) = std::fs::read_dir(client_dir.path()) else {
            continue;
        };
        for file in files.flatten() {
            if file.path().extension().is_some_and(|ext| ext == "jsonl") {
                found.push((client.clone(), file.path()));
            }
        }
    }
    found
}

/// Issue #671's acceptance criteria name these three by hand: PR #648 shipped
/// `claude` "intentionally not run" and `grok`/`aider` "inferred from the shared
/// adapters". A committed transcript is the evidence that each was really run.
#[test]
fn the_never_run_integrations_have_recorded_sessions() {
    let recorded = recorded_transcripts();
    for id in ["claude", "grok", "aider"] {
        assert!(
            recorded.iter().any(|(client, _)| client == id),
            "no recorded session under experiments/agentic_cli_matrix/recorded/{id}/ \
             — PR #648 shipped this integration without ever running it"
        );
    }
}

#[test]
fn every_recorded_transcript_is_replayable() {
    let locked = locked_ids();
    let recorded = recorded_transcripts();
    assert!(!recorded.is_empty(), "no recorded transcripts committed");

    for (client, path) in recorded {
        assert!(
            locked.contains(&client),
            "recorded/{client} is not a pinned client"
        );
        let text = std::fs::read_to_string(&path).expect("read transcript");
        let rows: Vec<&str> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        assert!(!rows.is_empty(), "{} is empty", path.display());
        for row in rows {
            let value: serde_json::Value = serde_json::from_str(row)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            // Bodies carry the run's temp paths and session ids, so a transcript
            // that kept them would differ from the next re-record for reasons
            // that mean nothing — see `matrix_record_case` in lib.sh.
            for field in ["request_body", "response_body"] {
                assert!(
                    value.get(field).is_none(),
                    "{} still carries {field}",
                    path.display()
                );
            }
            let status = value.get("status").and_then(serde_json::Value::as_u64);
            assert!(
                status.is_none_or(|code| code < 400),
                "{} records a failing exchange: {status:?}",
                path.display()
            );
        }
    }
}

#[test]
fn matrix_scripts_are_executable() {
    for script in [
        "experiments/agentic_cli_matrix/install_client.sh",
        "experiments/agentic_cli_matrix/run_leg.sh",
        "experiments/agentic_cli_matrix/run_matrix.sh",
        "experiments/agentic_cli_matrix/replay.sh",
    ] {
        let path: &Path = &root().join(script);
        assert!(path.exists(), "{script} is missing");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(path)
                .expect("metadata")
                .permissions()
                .mode();
            assert!(
                mode & 0o111 != 0,
                "{script} is not executable (mode {mode:o})"
            );
        }
    }
}
