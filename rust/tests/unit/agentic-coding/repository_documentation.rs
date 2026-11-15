//! Native route qualification, executed only by CI. Inputs contain no fixed answer.
use super::super::planner::AgenticPlan;
use super::plan_repository_documentation;
use crate::protocol::{ChatMessage, ToolCall};
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const TOOLS: [&str; 3] = ["read", "edit", "write"];
/// Issue #1560: a work-item final is a localized report, never a JSON object;
/// its record stays in the trace.
fn recorded(answer: &str) -> Value {
    assert!(
        !crate::agentic_coding::work_item_report::parses_as_json_object(answer),
        "{answer}"
    );
    crate::agentic_coding::work_item_report::reported_work_item_record(answer)
        .expect("a work-item final keeps its record")
}
struct Run {
    state: Value,
    transcript: Vec<(String, String)>,
}
fn append(messages: &mut Vec<ChatMessage>, tool: &str, arguments: &Value, content: &str) {
    let identifier = format!("repository-documentation-{}", messages.len());
    let mut assistant = ChatMessage::assistant("");
    assistant
        .tool_calls
        .push(ToolCall::function(&identifier, tool, arguments.to_string()));
    messages.push(assistant);
    let mut result = ChatMessage::new("tool", content);
    result.tool_call_id = Some(identifier);
    result.name = Some(tool.to_owned());
    if tool == "read" {
        result.source_read = Some(
            json!({"path":arguments["filePath"],"success":true,"complete":true,"format":"raw"}),
        );
    }
    messages.push(result);
}
fn drive(task: &str, files: &mut BTreeMap<String, String>) -> Run {
    let mut messages = vec![ChatMessage::user(task)];
    let mut transcript = Vec::new();
    for _ in 0..24 {
        let planned = plan_repository_documentation(task, &messages, &TOOLS, None, &mut None)
            .expect("OwnedRepositoryDocumentationTask");
        match planned {
            AgenticPlan::Final(answer) => {
                return Run {
                    state: recorded(&answer),
                    transcript,
                };
            }
            AgenticPlan::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1, "one physical step at a time");
                let call = &calls[0];
                let arguments: Value =
                    serde_json::from_str(&call.arguments).expect("FileToolArguments");
                let path = arguments["filePath"]
                    .as_str()
                    .expect("OwnedFilePath")
                    .to_owned();
                transcript.push((call.tool.clone(), path.clone()));
                match call.tool.as_str() {
                    "read" => {
                        let content = files.get(&path).expect("FrozenSourceOrGuide").clone();
                        append(&mut messages, "read", &arguments, &content);
                    }
                    "edit" => {
                        let before = arguments["oldString"].as_str().expect("ObservedPreimage");
                        let after = arguments["newString"].as_str().expect("DerivedCandidate");
                        assert_eq!(
                            files.get(&path).map(String::as_str),
                            Some(before),
                            "exact edit precondition"
                        );
                        assert_ne!(before, after, "useful edit only");
                        files.insert(path, after.to_owned());
                        append(&mut messages, "edit", &arguments, "");
                    }
                    tool => panic!("unexpected physical effect/tool: {tool}"),
                }
            }
        }
    }
    panic!("repository documentation exceeded finite24step bound");
}
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/repository-documentation-original.json"
    ))
    .expect("FrozenOriginalRepositoryInput")
}
/// Issue #1483: the edit is independently accepted against the raw sources,
/// while integration semantics stay open, so the result is partial.
fn assert_partial(state: &Value, authored: bool) {
    assert_eq!(state["schema"], "repository-work-item-controller/v1");
    assert_eq!(state["complete"], false);
    assert_eq!(state["verified"], true);
    assert_eq!(state["authored"], authored);
    assert_eq!(
        state["authorshipBoundary"],
        json!({"mechanism":"supplied-documentation-derivation","targetOutputObserved":authored,
            "autonomousCredit":0,"laterReviewerWork":"not-observed"})
    );
    for kind in ["source-anchored-facts"]
        .into_iter()
        .chain(authored.then_some("preimage-preserved"))
    {
        assert_eq!(check_status(state, kind), "passed", "{kind}");
    }
    let remaining = state["remainingWork"]
        .as_array()
        .expect("RemainingAcceptance");
    assert!(remaining.contains(&json!("consumer-discovery")));
    assert!(remaining.contains(&json!("independent-integration-semantics")));
}
fn check_status<'a>(state: &'a Value, kind: &str) -> &'a str {
    state["checks"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|check| check["kind"] == kind)
        .and_then(|check| check["status"].as_str())
        .unwrap_or_default()
}
fn original_files(fixture: &Value) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    for file in fixture["files"].as_array().expect("FrozenInputFiles") {
        files.insert(
            file["path"].as_str().expect("FrozenPath").to_owned(),
            file["source"].as_str().expect("FrozenSource").to_owned(),
        );
    }
    files
}
fn original_task(fixture: &Value) -> String {
    format!(
        "{}\n\n{}",
        fixture["issue"]["title"].as_str().expect("OriginalTitle"),
        fixture["issue"]["body"].as_str().expect("OriginalBody")
    )
}
fn requirement_status<'a>(state: &'a Value, needle: &str) -> Vec<&'a str> {
    state["requirements"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|row| {
            row["text"]
                .as_str()
                .is_some_and(|text| text.contains(needle))
        })
        .filter_map(|row| row["status"].as_str())
        .collect()
}
#[test]
fn original_frozen_work_item_edits_once_reads_fresh_sources_and_repeats_without_writes() {
    let fixture = fixture();
    assert_eq!(
        fixture["sourceCommit"],
        "4c18fa47059e19963601a35d749db5c493d4245f"
    );
    let frozen = fixture["files"].as_array().expect("FrozenInputFiles");
    assert_eq!(
        frozen.len(),
        4,
        "four exact pre-fix inputs, no fixed guide answer"
    );
    let mut files = BTreeMap::new();
    for file in frozen {
        let source = file["source"].as_str().expect("FrozenSource");
        assert_eq!(
            sha256_hex(source.as_bytes()),
            file["sha256"].as_str().expect("FrozenSourceHash")
        );
        files.insert(
            file["path"].as_str().expect("FrozenPath").to_owned(),
            source.to_owned(),
        );
    }
    let original_files = files.clone();
    let guide_path = "docs/associative-tech-stack.md";
    let original_guide = files[guide_path].clone();
    let task = format!(
        "{}\n\n{}",
        fixture["issue"]["title"].as_str().expect("OriginalTitle"),
        fixture["issue"]["body"].as_str().expect("OriginalBody")
    );
    let run = drive(&task, &mut files);
    assert_eq!(run.state["reason"], "UsefulCandidateReadbackVerified");
    assert_partial(&run.state, true);
    assert_eq!(check_status(&run.state, "task-claims"), "passed");
    let claims = run.state["claims"]
        .as_array()
        .expect("JudgedClaims")
        .iter()
        .map(|claim| (claim["name"].clone(), claim["status"].clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        claims,
        [
            "command-stream",
            "web-capture",
            "web-search",
            "@link-assistant/web-search"
        ]
        .map(|name| (json!(name), json!("confirmed")))
    );
    assert_eq!(
        requirement_status(&run.state, "`command-stream = \"1.5\"`"),
        ["verified"]
    );
    assert_eq!(
        requirement_status(&run.state, "`getDefaultProviderIds`"),
        ["verified"]
    );
    assert_eq!(
        requirement_status(&run.state, "relationship to knowledge acquisition"),
        ["unverified"]
    );
    assert!(
        run.state["candidateEdits"][0]["diff"]["lines"]
            .as_array()
            .expect("AddedLines")
            .contains(&json!(
                "+| `command-stream` | `1.5` | `dependencies` | `unqualified` | `{}` |"
            ))
    );
    let report = crate::agentic_coding::work_item_report::work_item_report(&run.state, &task);
    assert!(report.starts_with("Partly done:"), "{report}");
    let guide = files[guide_path].clone();
    assert!(guide.starts_with(&original_guide));
    assert_ne!(
        sha256_hex(guide.as_bytes()),
        sha256_hex(original_guide.as_bytes())
    );
    let facts = run.state["facts"].as_array().expect("ObservedFacts");
    for (name, version) in [
        ("command-stream", "1.5"),
        ("web-capture", "0.3.37"),
        ("web-search", "0.6.0"),
        ("@link-assistant/web-search", "0.11.1"),
    ] {
        assert!(
            facts.iter().any(|row| row["kind"] == "manifest-declaration"
                && row["name"] == name
                && row["version"] == version),
            "{name}"
        );
        assert!(guide.contains(&format!("`{name}`")));
    }
    let usage = facts
        .iter()
        .find(|row| row["kind"] == "observed-source-usage")
        .expect("ActualSourceSyntaxFacts");
    let calls = usage["calls"].as_array().expect("ObservedImportedCalls");
    for name in ["getDefaultProviderIds", "mergeResults"] {
        assert!(calls.iter().any(|call| call["imported"] == name), "{name}");
    }
    let edits: Vec<_> = run
        .transcript
        .iter()
        .enumerate()
        .filter(|(_, (tool, _))| tool == "edit")
        .collect();
    assert_eq!(edits.len(), 1);
    let edit_index = edits[0].0;
    assert_eq!(edits[0].1.1, guide_path);
    let source_paths: BTreeSet<_> = original_files
        .keys()
        .filter(|path| path.as_str() != guide_path)
        .cloned()
        .collect();
    let tail = &run.transcript[edit_index + 1..];
    assert!(tail.iter().all(|(tool, _)| tool == "read"));
    assert_eq!(tail.last().map(|(_, path)| path.as_str()), Some(guide_path));
    let freshly_read: BTreeSet<_> = tail[..tail.len() - 1]
        .iter()
        .map(|(_, path)| path.clone())
        .collect();
    assert_eq!(freshly_read, source_paths);
    for path in &source_paths {
        assert_eq!(
            files[path], original_files[path],
            "source inputs remain unchanged"
        );
    }
    let again = drive(&task, &mut files);
    assert_eq!(again.state["reason"], "VerifiedNoOp");
    assert_partial(&again.state, false);
    assert!(again.transcript.iter().all(|(tool, _)| tool == "read"));
    assert_eq!(files[guide_path], guide);
}
#[test]
fn a_version_claim_the_manifest_contradicts_is_rejected_before_any_edit() {
    let fixture = fixture();
    let mut files = original_files(&fixture);
    let before = files.clone();
    let task = original_task(&fixture);
    let contradicted = task.replacen(
        "`command-stream = \"1.5\"`",
        "`command-stream = \"1.9\"`",
        1,
    );
    assert_ne!(contradicted, task);
    let run = drive(&contradicted, &mut files);
    assert_eq!(run.state["reason"], "ContradictedIntegrationClaim");
    assert_eq!(run.state["authored"], false);
    assert_eq!(run.state["verified"], false);
    assert!(run.transcript.iter().all(|(tool, _)| tool == "read"));
    assert_eq!(files, before);
    assert_eq!(
        run.state["unresolved"],
        json!([{"reference":"command-stream = \"1.9\"","reason":"ContradictedIntegrationClaim","observed":["1.5"]}])
    );
    assert_eq!(check_status(&run.state, "task-claims"), "failed");
    assert_eq!(
        requirement_status(&run.state, "`command-stream = \"1.9\"`"),
        ["contradicted"]
    );
    let report =
        crate::agentic_coding::work_item_report::work_item_report(&run.state, &contradicted);
    assert!(
        report.contains("`command-stream = \"1.9\"`: `ContradictedIntegrationClaim`"),
        "{report}"
    );
}
#[test]
fn renamed_components_follow_manifest_and_import_alias_without_known_component_answers() {
    let mut files = BTreeMap::from([
        ("native/Project.toml".to_owned(), "[dependencies]\nribbon-core = {version=\"2.7\",default-features=false,features=[\"parse\"]}\n".to_owned()),
        ("web/client.mjs".to_owned(), "import {decode as readPacket} from \"ribbon-core\";\nexport function load(value){return readPacket(value,{mode:\"strict\"});}\n".to_owned()),
        ("docs/runtime.md".to_owned(), "# Runtime\n\nPreserve these local notes.\n".to_owned()),
    ]);
    let before = files.clone();
    let task = "Document the ribbon-core dependency in docs/runtime.md based on native/Project.toml and web/client.mjs. Preserve the existing guide and include observed declarations and imported calls.";
    let run = drive(task, &mut files);
    assert_eq!(run.state["reason"], "UsefulCandidateReadbackVerified");
    assert_partial(&run.state, true);
    // Every term the one requirement names is established by the anchored facts.
    assert_eq!(requirement_status(&run.state, "ribbon-core"), ["verified"]);
    let guide = &files["docs/runtime.md"];
    assert!(guide.starts_with(&before["docs/runtime.md"]));
    for text in ["`ribbon-core`", "`2.7`", "`decode`"] {
        assert!(guide.contains(text));
    }
    assert!(!guide.contains("command-stream"));
    assert!(
        run.state["facts"]
            .as_array()
            .expect("ObservedFacts")
            .iter()
            .any(|row| {
                row.get("calls")
                    .and_then(Value::as_array)
                    .is_some_and(|calls| calls.iter().any(|call| call["binding"] == "readPacket"))
            })
    );
    for path in ["native/Project.toml", "web/client.mjs"] {
        assert_eq!(files[path], before[path]);
    }
}
#[test]
fn malformed_manifests_and_negated_documentation_never_mutate() {
    let mut files = BTreeMap::from([
        (
            "one.json".to_owned(),
            "{\"dependencies\":{\"river\":\"1\",\"river\":\"2\"}}".to_owned(),
        ),
        (
            "two.json".to_owned(),
            "{\"dependencies\":{\"sand\":\"3\"}}".to_owned(),
        ),
        ("guide.md".to_owned(), "# Keep this guide\n".to_owned()),
    ]);
    let before = files.clone();
    for task in [
        "Document dependencies from one.json and two.json in guide.md. Preserve existing text.",
        "Do not document dependencies from one.json or two.json in guide.md. Preserve existing text.",
    ] {
        let run = drive(task, &mut files);
        for key in ["complete", "authored", "verified"] {
            assert_eq!(run.state[key], false);
        }
        assert!(run.transcript.iter().all(|(tool, _)| tool == "read"));
        assert_eq!(files, before);
    }
}

#[test]
fn a_consumer_search_coverage_record_is_skipped_by_the_anchored_facts_check() {
    use crate::agentic_coding::documentation_acceptance::source_anchored_facts;
    let sources =
        [json!({"path":"package.json","source":"{\"dependencies\":{\"river\":\"1.2.3\"}}"})];
    let declaration = json!({"kind":"manifest-declaration","manifest":"package.json","name":"river","version":"1.2.3","section":"dependencies"});
    let coverage = json!({"kind":"observed-consumer-discovery","state":"incomplete","gaps":["search-truncated"]});
    let artifact = "| river | 1.2.3 | dependencies |\n";
    let anchored = source_anchored_facts(
        &json!([declaration.clone(), coverage.clone()]),
        &sources,
        artifact,
    );
    assert_eq!(anchored["status"], "passed", "{anchored}");
    assert_eq!(anchored["checked"], 1);
    assert_eq!(anchored["missing"], json!([]));
    let unknown = source_anchored_facts(
        &json!([declaration, {"kind":"unknown-fact"}]),
        &sources,
        artifact,
    );
    assert_eq!(unknown["status"], "unavailable");
    let only_coverage = source_anchored_facts(&json!([coverage]), &sources, artifact);
    assert_eq!(only_coverage["status"], "unavailable");
}
