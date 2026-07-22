//! Regression coverage for issue #819 local path discovery.

use std::{collections::BTreeSet, fs, path::Path};

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::seed::shell_intent_vocabulary;
use formal_ai::ChatMessage;

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
    let vocabulary = shell_intent_vocabulary();

    for action in &vocabulary.local_path_search_actions {
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

    for scope in &vocabulary.local_path_search_scopes {
        for cue in &scope.cues {
            let prompt = format!("Find hive-control-center folder {cue}");
            let (_, arguments) = first_tool_call(&prompt);
            let command = arguments["command"].as_str().expect("shell command");
            assert!(command.starts_with("find "), "scope {cue:?}: {command}");
            assert!(command.contains(&scope.root), "scope {cue:?}: {command}");
        }
    }

    for kind in &vocabulary.local_path_search_kinds {
        for cue in &kind.cues {
            let prompt = format!("Find hive-control-center {cue} on my desktop");
            let (_, arguments) = first_tool_call(&prompt);
            let command = arguments["command"].as_str().expect("shell command");
            assert!(command.starts_with("find "), "kind {cue:?}: {command}");
            assert!(command.contains(&kind.predicate), "kind {cue:?}: {command}");
        }
    }
}

#[test]
fn fuzzy_find_command_locates_the_reported_folder_name() {
    let (_, arguments) = first_tool_call("Find hive-mind-control center folder on my desktop");
    let command = arguments["command"].as_str().expect("shell command");
    let fixture =
        std::env::temp_dir().join(format!("formal-ai-issue819-find-{}", std::process::id()));
    let expected = fixture.join("Archive/hive-control-center");
    std::fs::create_dir_all(&expected).expect("reported folder fixture");

    let output = std::process::Command::new("bash")
        .args(["-c", command])
        .env("FORMAL_AI_DESKTOP_DIR", &fixture)
        .output()
        .expect("execute generated find command");

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        expected.to_string_lossy()
    );
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
                    assert!(command.starts_with("find \".\""), "{id}: {command}")
                }
                marker => assert!(command.contains(marker), "{id}: {command}"),
            }
            assert!(command.contains(expected_predicate), "{id}: {command}");
            assert!(command.ends_with("-print -quit"), "{id}: {command}");
            languages.insert(language);
            passed += 1;
        }
    }

    assert_eq!(languages, BTreeSet::from(["en", "hi", "ru", "zh"]));
    assert_eq!(passed, minimum_pass_count);
    assert_eq!(passed, 56);
}
