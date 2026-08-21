//! Regression coverage for issue #841's published TUI integration metadata.

use std::collections::BTreeSet;
use std::path::PathBuf;

use formal_ai::client_contract_learning::{
    ClientContractObservation, learn_client_contracts, load_observations,
};
use formal_ai::seed::{client_integrations, response_for};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn assert_no_edge_padding(svg: &str, artifact: &str) {
    let mut remaining = svg;
    let mut text_run_count = 0;

    while let Some(text_start) = remaining.find("<text") {
        let text = &remaining[text_start..];
        let body_start = text
            .find('>')
            .unwrap_or_else(|| panic!("{artifact} has an unterminated <text> element"))
            + 1;
        let body = &text[body_start..];
        let body_end = body
            .find("</text>")
            .unwrap_or_else(|| panic!("{artifact} has an unterminated <text> body"));
        let visible_text = &body[..body_end];
        assert_eq!(
            visible_text.trim(),
            visible_text,
            "{artifact} includes terminal row padding in a visible SVG text run"
        );
        text_run_count += 1;
        remaining = &body[body_end + "</text>".len()..];
    }

    assert!(text_run_count > 0, "{artifact} contains no SVG text runs");
}

#[test]
fn invalid_typed_json_settings_have_messages_for_every_supported_language() {
    let cases = [
        ("en", "invalid typed JSON setting"), // English
        ("ru", "недопустимая типизированная настройка JSON"),
        ("hi", "अमान्य टाइप की गई JSON सेटिंग"),
        ("zh", "无效的带类型 JSON 设置"),
    ];

    for (language, expected_text) in cases {
        let response = response_for("client_integration_invalid_typed_json_setting", language)
            .unwrap_or_else(|| panic!("missing {language} invalid-setting response"));
        assert!(response.contains(expected_text), "{language}: {response}");
        assert!(response.contains("{rendered}"), "{language}: {response}");
        assert!(response.contains("{error}"), "{language}: {response}");
    }
}

#[test]
fn repeated_tui_captures_propose_only_stable_human_gated_contract_facts() {
    let observations = [
        r#"{
            "client_id": "opencode",
            "capability": "tui_replay",
            "task_wording": "Find the hive-mind-control center folder on my Desktop",
            "delivery": "tool_call",
            "evidence": "first/recording.svg",
            "observed_contract": {
                "tui_initial_geometry": ["80x30"],
                "tui_artifact": ["recording.gif", "recording.svg"],
                "tui_renderer_feature": ["css_keyframes", "one_capture_only"]
            }
        }"#,
        r#"{
            "client_id": "opencode",
            "capability": "tui_replay",
            "task_wording": "Locate and verify my Desktop hive-mind-control center directory",
            "delivery": "tool_call",
            "evidence": "second/recording.svg",
            "observed_contract": {
                "tui_initial_geometry": ["80x30"],
                "tui_artifact": ["recording.gif", "recording.svg"],
                "tui_renderer_feature": ["css_keyframes"]
            }
        }"#,
    ]
    .map(|json| {
        serde_json::from_str::<ClientContractObservation>(json)
            .expect("TUI observation should deserialize")
    });

    let report = learn_client_contracts(&observations, &client_integrations());
    let proposed = report
        .proposals
        .iter()
        .map(|proposal| (proposal.field.as_str(), proposal.value.as_str()))
        .collect::<BTreeSet<_>>();

    assert_eq!(report.independently_worded_groups, 1);
    assert!(report.awaiting_human_review);
    assert_eq!(
        proposed,
        BTreeSet::from([
            ("tui_artifact", "recording.gif"),
            ("tui_artifact", "recording.svg"),
            ("tui_initial_geometry", "80x30"),
            ("tui_renderer_feature", "css_keyframes"),
        ])
    );
    assert!(!report.links_notation().contains("one_capture_only"));
    assert!(report.links_notation().contains("awaiting_human_review"));
}

#[test]
fn committed_real_tui_sessions_produce_a_deterministic_review_artifact() {
    let observations_path =
        root().join("docs/case-studies/issue-841/tui-contract-learning/observations.jsonl");
    let observations = load_observations(&[&observations_path]).expect("load TUI observations");
    let report = learn_client_contracts(&observations, &client_integrations());

    assert_eq!(report.observation_count, 6);
    assert_eq!(report.independently_worded_groups, 3);
    assert_eq!(report.findings.len(), 3);
    assert_eq!(report.proposals.len(), 42);
    assert!(report.awaiting_human_review);
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.status == "confirmed")
    );

    for client in ["opencode", "claude", "codex"] {
        let client_observations = observations
            .iter()
            .filter(|observation| observation.client_id == client)
            .collect::<Vec<_>>();
        assert_eq!(client_observations.len(), 2, "{client}");
        assert_ne!(
            client_observations[0].task_wording, client_observations[1].task_wording,
            "{client} observations must come from independent task wordings"
        );
    }

    for observation in &observations {
        assert!(
            root().join(&observation.evidence).is_file(),
            "observation evidence is missing: {}",
            observation.evidence
        );
        let replay = read(&observation.evidence);
        for required in [
            "@keyframes",
            "steps(1, end)",
            "@font-face",
            "data:font/woff2;base64,",
            "textLength=",
            "lengthAdjust=\"spacingAndGlyphs\"",
            "xml:space=\"preserve\"",
            "rx=\"0\"",
        ] {
            assert!(
                replay.contains(required),
                "{} omitted {required}",
                observation.evidence
            );
        }
        assert!(
            !replay.contains("<animate"),
            "{} regressed to lossy SVG animation elements",
            observation.evidence
        );
        assert_no_edge_padding(&replay, &observation.evidence);

        let snapshot = root()
            .join(&observation.evidence)
            .with_file_name("snapshot.svg");
        assert_no_edge_padding(
            &std::fs::read_to_string(&snapshot).expect("read replay snapshot"),
            &snapshot.display().to_string(),
        );

        let gif = root()
            .join(&observation.evidence)
            .with_file_name("recording.gif");
        assert_eq!(
            &std::fs::read(&gif).expect("read replay GIF")[..6],
            b"GIF89a",
            "{} is not a valid GIF89a replay",
            gif.display()
        );
    }

    let expected =
        read("docs/case-studies/issue-841/tui-contract-learning/tui-contract-learning-report.lino");
    assert_eq!(expected, format!("{}\n", report.links_notation()));
}

#[test]
fn formal_ai_executes_tui_contract_learning_through_the_real_agent_cli() {
    let workflow = read(".github/workflows/release.yml");
    assert!(workflow.contains("run_issue_841_tui_learning.sh"));
    assert!(workflow.contains(
        "/tmp/formal-ai-tui-artifacts/path-discovery/local-search/tui-contract-observations.jsonl"
    ));

    let expected =
        read("docs/case-studies/issue-841/tui-contract-learning/tui-contract-learning-report.lino");
    let agent_authored = read(
        "docs/case-studies/issue-841/tui-contract-learning/agent-authored-tui-contract-learning-report.lino",
    );
    assert_eq!(
        agent_authored, expected,
        "Agent CLI must write exactly Formal AI's deterministic learning report"
    );

    let plan = read("docs/case-studies/issue-841/tui-contract-learning/general-change-plan.lino");
    assert!(plan.contains("capability \"Run\""));
    assert!(plan.contains("formal-ai clients learn"));
    assert!(plan.contains("> 'tui-contract-learning-report.lino'"));
    assert!(plan.contains("command \"cat tui-contract-learning-report.lino\""));

    let stream = read("docs/case-studies/issue-841/tui-contract-learning/agent-stream.jsonl");
    assert!(stream.contains("\"status\":\"success\""));
    assert!(stream.contains("Completed the general change request"));
}
