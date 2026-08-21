//! Regression coverage for issue #819 local path discovery.

use std::{collections::BTreeSet, fs, path::Path};

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::protocol::ToolCall;
use formal_ai::seed::{
    self, ROLE_LOCAL_PATH_DIRECTORY_KIND, ROLE_LOCAL_PATH_FILE_KIND, ROLE_LOCAL_PATH_SCOPE_CURRENT,
    ROLE_LOCAL_PATH_SCOPE_DESKTOP, ROLE_LOCAL_PATH_SCOPE_HOME, ROLE_LOCAL_PATH_SEARCH_ACTION,
};

fn first_tool_call(prompt: &str) -> (String, serde_json::Value) {
    let plan = plan_chat_step(
        &[ChatMessage::user(prompt)],
        &["bash", "websearch", "webfetch"],
    )
    .expect(prompt);
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("{prompt:?} did not produce a tool call: {plan:?}");
    };
    let call = calls.first().expect("one tool call");
    let arguments = serde_json::from_str(&call.arguments).expect("JSON tool arguments");
    (call.tool.clone(), arguments)
}

fn next_tool_call(messages: &[ChatMessage]) -> PlannedToolCall {
    let plan = plan_chat_step(messages, &["bash", "websearch", "webfetch"]).expect("next plan");
    let AgenticPlan::ToolCalls(calls) = plan else {
        panic!("expected a tool call: {plan:?}");
    };
    assert_eq!(calls.len(), 1);
    calls.into_iter().next().unwrap()
}

#[test]
fn reported_desktop_request_uses_find_instead_of_the_web() {
    let (tool, arguments) = first_tool_call("Find hive-mind-control center folder on my desktop");

    assert_eq!(tool, "bash");
    let command = arguments["command"].as_str().expect("shell command");
    assert!(command.starts_with("find "), "{command}");
    assert!(command.contains("Desktop"), "{command}");
    assert!(command.contains("-type d"), "{command}");
    assert!(command.contains("hive"), "{command}");
    assert!(command.contains("control"), "{command}");
    assert!(command.contains("center"), "{command}");
}

#[test]
fn local_path_discovery_generalizes_across_language_action_kind_and_scope() {
    for (language, prompt, expected_root, expected_kind) in [
        (
            "English",
            "Locate quarterly-report.pdf on this computer",
            "FORMAL_AI_HOME_DIR",
            "-type f",
        ),
        (
            "ru",
            "Найди папку hive-control-center на моём рабочем столе",
            "FORMAL_AI_DESKTOP_DIR",
            "-type d",
        ),
        (
            "hi",
            "मेरे डेस्कटॉप पर hive-control-center फ़ोल्डर खोजें",
            "FORMAL_AI_DESKTOP_DIR",
            "-type d",
        ),
        (
            "zh",
            "在我的桌面上查找 hive-control-center 文件夹",
            "FORMAL_AI_DESKTOP_DIR",
            "-type d",
        ),
    ] {
        let (tool, arguments) = first_tool_call(prompt);
        assert_eq!(tool, "bash", "{language}: {prompt}");
        let command = arguments["command"].as_str().expect("shell command");
        assert!(command.starts_with("find "), "{language}: {command}");
        assert!(command.contains(expected_root), "{language}: {command}");
        assert!(command.contains(expected_kind), "{language}: {command}");
    }
}

#[test]
fn every_seeded_local_path_phrase_routes_to_find() {
    let lexicon = seed::lexicon();

    for action in lexicon.words_for_role(ROLE_LOCAL_PATH_SEARCH_ACTION) {
        let prompt = format!("{action} hive-control-center folder on my desktop");
        let (tool, arguments) = first_tool_call(&prompt);
        assert_eq!(tool, "bash", "action {action:?}");
        assert!(
            arguments["command"]
                .as_str()
                .is_some_and(|command| command.starts_with("find ")),
            "action {action:?}: {arguments}"
        );
    }

    for (role, root) in [
        (ROLE_LOCAL_PATH_SCOPE_DESKTOP, "FORMAL_AI_DESKTOP_DIR"),
        (ROLE_LOCAL_PATH_SCOPE_HOME, "FORMAL_AI_HOME_DIR"),
        (ROLE_LOCAL_PATH_SCOPE_CURRENT, "\".\""),
    ] {
        for cue in lexicon.words_for_role(role) {
            let prompt = format!("Find hive-control-center folder {cue}");
            let (_, arguments) = first_tool_call(&prompt);
            let command = arguments["command"].as_str().expect("shell command");
            assert!(command.starts_with("find "), "scope {cue:?}: {command}");
            assert!(command.contains(root), "scope {cue:?}: {command}");
        }
    }

    for (role, predicate) in [
        (ROLE_LOCAL_PATH_DIRECTORY_KIND, "-type d"),
        (ROLE_LOCAL_PATH_FILE_KIND, "-type f"),
    ] {
        for cue in lexicon.words_for_role(role) {
            let prompt = format!("Find hive-control-center {cue} on my desktop");
            let (_, arguments) = first_tool_call(&prompt);
            let command = arguments["command"].as_str().expect("shell command");
            assert!(command.starts_with("find "), "kind {cue:?}: {command}");
            assert!(command.contains(predicate), "kind {cue:?}: {command}");
        }
    }
}

#[test]
fn fuzzy_find_command_locates_the_reported_folder_name() {
    let prompt = "Find hive-mind-control center folder on my desktop";
    let mut messages = vec![ChatMessage::user(prompt)];
    let exact = next_tool_call(&messages);
    let fixture =
        std::env::temp_dir().join(format!("formal-ai-issue819-find-{}", std::process::id()));
    let expected = fixture.join("Archive/hive-control-center");
    std::fs::create_dir_all(&expected).expect("reported folder fixture");

    let exact_arguments: serde_json::Value =
        serde_json::from_str(&exact.arguments).expect("exact arguments");
    let exact_command = exact_arguments["command"].as_str().expect("exact command");
    let exact_output = std::process::Command::new("bash")
        .args(["-c", exact_command])
        .env("FORMAL_AI_DESKTOP_DIR", &fixture)
        .output()
        .expect("execute generated find command");
    assert!(exact_output.stdout.is_empty(), "{exact_output:?}");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "exact",
        exact.tool,
        exact.arguments,
    )]));
    messages.push(ChatMessage::tool_result("exact", "bash", "(no output)"));

    let widened = next_tool_call(&messages);
    let widened_arguments: serde_json::Value =
        serde_json::from_str(&widened.arguments).expect("widened arguments");
    let widened_command = widened_arguments["command"]
        .as_str()
        .expect("widened command");
    assert!(widened_command.contains("*hive*"), "{widened_command}");
    let output = std::process::Command::new("bash")
        .args(["-c", widened_command])
        .env("FORMAL_AI_DESKTOP_DIR", &fixture)
        .output()
        .expect("execute widened find command");

    assert!(output.status.success(), "{output:?}");
    let widened_output = String::from_utf8(output.stdout).unwrap();
    let expected_text = expected.to_string_lossy();
    assert!(
        widened_output.lines().any(|line| line == expected_text),
        "{widened_command}: {widened_output}"
    );
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "widened",
        widened.tool,
        widened.arguments,
    )]));
    messages.push(ChatMessage::tool_result("widened", "bash", &widened_output));
    let answer = plan_chat_step(&messages, &["bash", "websearch", "webfetch"])
        .expect("grounded final answer");
    let AgenticPlan::Final(answer) = answer else {
        panic!("expected final answer, got {answer:?}");
    };
    assert!(answer.contains(expected_text.as_ref()), "{answer}");
    assert!(!answer.ends_with("/Archive"), "{answer}");
    std::fs::remove_dir_all(&fixture).expect("remove isolated fixture");
}

#[test]
fn open_web_find_requests_still_use_web_search() {
    for prompt in [
        "Find information online about hive mind research",
        "Search the web for hive control centers",
    ] {
        let (tool, _) = first_tool_call(prompt);
        assert_eq!(tool, "websearch", "{prompt}");
    }
}

fn lino_records(text: &str) -> Vec<Vec<&str>> {
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(current);
            current = Vec::new();
        }
        current.push(line);
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

fn lino_field<'a>(record: &[&'a str], wanted: &str) -> &'a str {
    record
        .iter()
        .filter_map(|line| line.trim().split_once(' '))
        .find_map(|(name, raw)| (name == wanted).then(|| raw.trim().trim_matches('"')))
        .unwrap_or_else(|| panic!("missing {wanted:?} in {record:?}"))
}

#[test]
fn local_path_discovery_benchmark_routes_every_case_to_find() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("data/benchmarks/local-path-discovery-suite.lino"))
        .expect("local-path benchmark manifest");
    let suite = lino_records(&manifest);
    let minimum_pass_count: usize = lino_field(&suite[0], "minimum_pass_count")
        .parse()
        .expect("numeric minimum_pass_count");
    let mut languages = BTreeSet::new();
    let mut passed = 0usize;

    for language in ["en", "ru", "hi", "zh"] {
        let fixture = fs::read_to_string(root.join(format!(
            "data/benchmarks/local-path-discovery/{language}.lino"
        )))
        .unwrap_or_else(|error| panic!("missing {language} benchmark partition: {error}"));
        for record in lino_records(&fixture) {
            assert_eq!(lino_field(&record, "record_type"), "local_path_search_case");
            assert_eq!(
                lino_field(&record, "source"),
                "self_authored_multilingual_variation"
            );
            assert_eq!(lino_field(&record, "language"), language);
            assert_eq!(lino_field(&record, "expected_tool"), "bash");
            assert_eq!(lino_field(&record, "prohibited_tool"), "websearch");

            let id = lino_field(&record, "id");
            let prompt = lino_field(&record, "prompt");
            let expected_root = lino_field(&record, "expected_root");
            let expected_predicate = lino_field(&record, "expected_predicate");
            let (tool, arguments) = first_tool_call(prompt);
            let command = arguments["command"].as_str().expect("find command");

            assert_eq!(tool, "bash", "{id}: {prompt}");
            assert!(command.starts_with("find "), "{id}: {command}");
            match expected_root {
                "CURRENT_DIRECTORY" => {
                    assert!(command.starts_with("find \".\""), "{id}: {command}");
                }
                marker => assert!(command.contains(marker), "{id}: {command}"),
            }
            assert!(command.contains(expected_predicate), "{id}: {command}");
            assert!(command.ends_with("-print"), "{id}: {command}");
            assert!(!command.contains("-print -quit"), "{id}: {command}");
            languages.insert(language);
            passed += 1;
        }
    }

    assert_eq!(languages, BTreeSet::from(["en", "hi", "ru", "zh"]));
    assert_eq!(passed, minimum_pass_count);
    assert_eq!(passed, 56);
}
