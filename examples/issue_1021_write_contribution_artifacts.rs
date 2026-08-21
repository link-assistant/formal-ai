//! Write the process artifacts of this contribution by driving the generator.
//!
//! Issue #1021 asks that the process artifacts be "tested by driving the
//! generator, not by asserting on a hand-written fixture". They are also
//! *written* that way: this example reads the author-supplied half of the
//! change from `docs/case-studies/issue-1021/closed-circle-run/input.json`,
//! renders it through [`formal_ai::contribution_artifacts::compose`], and puts
//! the result where the repository's gates look for it — one fragment per
//! contribution in `changelog.d/`, and the umbrella pull-request body next to
//! the captured session.
//!
//! Run it with `cargo run --example issue_1021_write_contribution_artifacts`.
//! `tests/unit/issue_1021_closed_circle.rs` then fails if either committed file
//! drifts from what `compose` renders.

use std::fs;
use std::path::Path;

use formal_ai::contribution_artifacts::{Contribution, compose};
use serde_json::Value;

/// Read a string field, or fail naming it.
fn text(entry: &Value, field: &str) -> String {
    entry[field]
        .as_str()
        .unwrap_or_else(|| panic!("{field} must be a string in the closed-circle input"))
        .to_owned()
}

/// Rebuild the [`Contribution`] an input entry describes.
fn contribution(entry: &Value, issue: u64, repository: &str) -> Contribution {
    Contribution {
        issue,
        repository: repository.to_owned(),
        slug: text(entry, "slug"),
        timestamp: text(entry, "timestamp"),
        bump: text(entry, "bump"),
        category: text(entry, "category"),
        title: text(entry, "title"),
        problem: text(entry, "problem"),
        cause: text(entry, "cause"),
        change: text(entry, "change"),
        verification: entry["verification"]
            .as_array()
            .expect("verification commands")
            .iter()
            .map(|command| command.as_str().expect("a command").to_owned())
            .collect(),
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input_path = root.join("docs/case-studies/issue-1021/closed-circle-run/input.json");
    let input: Value = serde_json::from_str(
        &fs::read_to_string(&input_path).expect("the closed-circle input must be committed"),
    )
    .expect("the closed-circle input is JSON");

    let issue = input["issue"].as_u64().expect("issue number");
    let repository = text(&input, "repository");

    for entry in input["contributions"].as_array().expect("contributions") {
        let artifacts = compose(&contribution(entry, issue, &repository))
            .expect("the seed defines this bump and category");
        let path = root.join(&artifacts.changelog_fragment_path);
        fs::create_dir_all(path.parent().expect("a parent directory"))
            .expect("the changelog directory");
        fs::write(&path, &artifacts.changelog_fragment).expect("write the fragment");
        println!("wrote {}", artifacts.changelog_fragment_path);
    }

    let umbrella = compose(&contribution(&input["pull_request"], issue, &repository))
        .expect("the seed defines the umbrella bump and category");
    let body_file = text(&input, "body_file");
    let body_path = root.join(&body_file);
    fs::create_dir_all(body_path.parent().expect("a parent directory")).expect("the run directory");
    fs::write(&body_path, &umbrella.pull_request_body).expect("write the body");
    println!("wrote {body_file}");
    println!("title: {}", umbrella.pull_request_title);
}
