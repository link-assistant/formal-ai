//! The closed circle of issue #1021, pinned as a replayable session.
//!
//! Issue #1021 asks for the whole loop to be covered, not its pieces: "the
//! closed circle pinned as a replayable session like `cargo test
//! self_coding_session_replays`". The circle it names runs from a prompt a user
//! typed, through the code the answer is made of, to the process artifacts a
//! contribution carries and the commands that would publish it.
//!
//! `docs/case-studies/issue-1021/closed-circle-run/input.json` holds the only
//! thing a replay cannot derive: what the author of the change supplies — which
//! issue, what broke, why, what changed, how it is covered — plus the reported
//! prompts themselves. Its `contributions` are the per-change changelog
//! fragments; its `pull_request` is the one umbrella entry the body is composed
//! from, because a pull request closes the whole issue rather than one third of
//! it. Everything else in `session.json` is computed here by driving the same
//! public API a real run drives:
//!
//! * [`plan_chat_step`] for the prompts that resolve to a shell command,
//! * [`UniversalSolver::solve`] for the prompts that resolve to an intent,
//! * [`formal_ai::contribution_artifacts::compose`] for the changelog fragment
//!   and the pull-request body, and
//! * [`formal_ai::contribution_write_path`] for the publishing ladder, exercised
//!   in **both** states from one process.
//!
//! So the committed capture is generator output rather than a fixture: nothing
//! below asserts on a hand-written sample, and the committed changelog fragments
//! in `changelog.d/` are compared against what `compose` renders today.
//!
//! Regenerate the capture after an intended change with
//! `FORMAL_AI_UPDATE_CLOSED_CIRCLE=1 cargo test --test unit -- issue_1021_closed_circle`.

use std::fs;

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::contribution_artifacts::{Contribution, compose};
use formal_ai::contribution_write_path::{
    Publication, WritePathDecision, decide_with, plan_publication_with,
};
use formal_ai::seed::contribution_artifact_vocabulary;
use formal_ai::{ChatMessage, UniversalSolver};
use serde_json::{Map, Value, json};

/// Directory holding the captured run.
fn run_dir() -> String {
    format!(
        "{}/docs/case-studies/issue-1021/closed-circle-run",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// The author-supplied half of the session.
fn input() -> Value {
    let path = format!("{}/input.json", run_dir());
    serde_json::from_str(&fs::read_to_string(&path).expect("closed-circle input"))
        .expect("closed-circle input is JSON")
}

/// The shell command the agentic planner resolves for `prompt`, or `None` when
/// it routes somewhere other than a command execution tool.
fn shell_command(prompt: &str) -> Option<String> {
    let plan = plan_chat_step(&[ChatMessage::user(prompt)], &["exec_command"])?;
    let AgenticPlan::ToolCalls(calls) = plan else {
        return None;
    };
    let arguments: Value = serde_json::from_str(&calls[0].arguments).ok()?;
    arguments["command"].as_str().map(str::to_owned)
}

/// A ladder decision, as the stable word a log would carry.
fn decision_word(decision: WritePathDecision) -> String {
    match decision {
        WritePathDecision::Unaffected => "unaffected".to_owned(),
        WritePathDecision::Permitted => "permitted".to_owned(),
        WritePathDecision::Refused(refusal) => format!("refused:{}", refusal.slug()),
    }
}

/// Read a string field, or panic naming the field.
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

/// Drive the whole circle and return the session it produces.
fn session(input: &Value) -> Value {
    let issue = input["issue"].as_u64().expect("issue number");
    let repository = text(input, "repository");
    let solver = UniversalSolver::default();

    let mut range = Vec::new();
    for entry in input["range"].as_array().expect("reported prompts") {
        let prompt = text(entry, "prompt");
        let mut record = Map::new();
        record.insert("issue".to_owned(), entry["issue"].clone());
        record.insert("prompt".to_owned(), Value::String(prompt.clone()));
        match text(entry, "mode").as_str() {
            "command" => {
                record.insert(
                    "command".to_owned(),
                    shell_command(&prompt).map_or(Value::Null, Value::String),
                );
            }
            "intent" => {
                record.insert(
                    "intent".to_owned(),
                    Value::String(solver.solve(&prompt).intent),
                );
            }
            other => panic!("unknown replay mode {other}"),
        }
        range.push(Value::Object(record));
    }

    let vocab = contribution_artifact_vocabulary().write_path;
    let mut ladder = Map::new();
    for (state, opted_in) in [("opt_in_absent", false), ("opt_in_present", true)] {
        let decisions: Vec<Value> = input["write_path_probe"]
            .as_array()
            .expect("probe commands")
            .iter()
            .map(|command| {
                let command = command.as_str().expect("a command");
                json!({
                    "command": command,
                    "decision": decision_word(decide_with(command, &vocab, opted_in)),
                })
            })
            .collect();
        ladder.insert(state.to_owned(), Value::Array(decisions));
    }

    let contributions: Vec<Contribution> = input["contributions"]
        .as_array()
        .expect("contributions")
        .iter()
        .map(|entry| contribution(entry, issue, &repository))
        .collect();
    let rendered: Vec<Value> = contributions
        .iter()
        .map(|contribution| {
            let artifacts = compose(contribution).expect("the seed defines this bump and category");
            json!({
                "slug": contribution.slug,
                "changelog_fragment_path": artifacts.changelog_fragment_path,
                "changelog_fragment": artifacts.changelog_fragment,
                "pull_request_title": artifacts.pull_request_title,
                "pull_request_body": artifacts.pull_request_body,
            })
        })
        .collect();

    let umbrella = compose(&contribution(&input["pull_request"], issue, &repository))
        .expect("the seed defines the umbrella bump and category");
    let publication = Publication {
        repository: repository.clone(),
        branch: text(input, "branch"),
        title: umbrella.pull_request_title.clone(),
        body_file: text(input, "body_file"),
    };
    let mut published = Map::new();
    for (state, opted_in) in [("opt_in_absent", false), ("opt_in_present", true)] {
        let planned = plan_publication_with(&publication, &vocab, opted_in).map_or_else(
            |refusal| json!(format!("refused:{}", refusal.slug())),
            |commands| json!(commands),
        );
        published.insert(state.to_owned(), planned);
    }

    json!({
        "issue": issue,
        "repository": repository,
        "range": range,
        "write_path": Value::Object(ladder),
        "publication": Value::Object(published),
        "pull_request": {
            "title": umbrella.pull_request_title,
            "body": umbrella.pull_request_body,
        },
        "contributions": rendered,
    })
}

/// The whole circle replays: every reported prompt still routes where issue
/// #1021 asked it to, the ladder still answers the same way in both states, and
/// the artifacts still render the same text — byte for byte against the capture
/// committed with this change.
#[test]
fn closed_circle_session_replays() {
    let input = input();
    let fresh = serde_json::to_string_pretty(&session(&input)).expect("session JSON");
    let path = format!("{}/session.json", run_dir());
    if std::env::var("FORMAL_AI_UPDATE_CLOSED_CIRCLE").as_deref() == Ok("1") {
        fs::write(&path, format!("{fresh}\n")).expect("write the capture");
    }
    let committed = fs::read_to_string(&path).expect("the committed session capture");
    assert_eq!(
        committed.trim(),
        fresh.trim(),
        "the closed-circle capture is stale; regenerate it with FORMAL_AI_UPDATE_CLOSED_CIRCLE=1"
    );
}

/// The lines of a composed changelog fragment that survive into `CHANGELOG.md`:
/// everything after the `---` frontmatter, minus the blank lines and the
/// `### Section` heading the release groups entries under.
fn released_body_lines(fragment: &str) -> Vec<&str> {
    fragment
        .split_once("---\n")
        .and_then(|(_, rest)| rest.split_once("---\n"))
        .map_or(fragment, |(_, body)| body)
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with("###"))
        .collect()
}

/// The process artifacts in the repository are this generator's output, not
/// prose someone typed next to it: each committed fragment in `changelog.d/`
/// and the pull-request body in the case study are compared against what
/// `compose` renders now.
#[test]
fn the_committed_process_artifacts_are_generator_output() {
    let input = input();
    let session = session(&input);
    let root = env!("CARGO_MANIFEST_DIR");
    let changelog = fs::read_to_string(format!("{root}/CHANGELOG.md")).expect("CHANGELOG.md");
    for rendered in session["contributions"].as_array().expect("contributions") {
        let path = format!("{root}/{}", text(rendered, "changelog_fragment_path"));
        let composed = text(rendered, "changelog_fragment");
        // A fragment does not outlive the release that ships it: v0.346.0
        // (c11b23d34) consumed this one and the assertion failed on every run
        // afterwards. Follow the entry across its lifecycle -- byte-identical
        // while it is still a fragment, and once released, every body line it
        // composed must appear in the CHANGELOG.md section it became.
        if let Ok(committed) = fs::read_to_string(&path) {
            assert_eq!(committed, composed, "{path}");
            continue;
        }
        for line in released_body_lines(&composed) {
            assert!(
                changelog.contains(line),
                "the fragment composed for {path} was consumed by a release, so \
                 CHANGELOG.md must carry its line {line:?}"
            );
        }
    }
    let body_path = format!("{root}/{}", text(&input, "body_file"));
    let committed = fs::read_to_string(&body_path).expect("the composed pull-request body");
    assert_eq!(
        committed,
        text(&session["pull_request"], "body"),
        "{body_path}"
    );
}

/// The gates the artifacts exist for still describe the shape they are rendered
/// in, read out of the gate scripts rather than restated here.
#[test]
fn the_artifacts_satisfy_the_gates_that_read_them() {
    let root = env!("CARGO_MANIFEST_DIR");
    let session = session(&input());
    let link_gate = fs::read_to_string(format!("{root}/scripts/check-pull-request-link.rs"))
        .expect("the pull-request link gate");
    let body = text(&session["pull_request"], "body");
    let keyword = contribution_artifact_vocabulary().closing_keyword;
    assert!(
        link_gate.to_lowercase().contains(&keyword.to_lowercase()),
        "the gate must accept the keyword the seed renders: {keyword}"
    );
    assert!(
        body.lines()
            .next()
            .is_some_and(|line| line.starts_with(&keyword)),
        "the body must open with the closing keyword, got: {body}"
    );
    for rendered in session["contributions"].as_array().expect("contributions") {
        let fragment = text(rendered, "changelog_fragment");
        assert!(
            fragment.starts_with("---\nbump: "),
            "a fragment must open with its bump frontmatter, got: {fragment}"
        );
        assert!(
            text(rendered, "changelog_fragment_path").starts_with("changelog.d/"),
            "a fragment belongs in the directory the gate reads"
        );
    }
}
