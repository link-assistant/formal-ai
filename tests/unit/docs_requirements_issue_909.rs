use std::fs;
use std::path::Path;

use formal_ai::seed::{ClientIntegration, ConfigFormat, client_integrations};

fn integration(id: &str) -> ClientIntegration {
    client_integrations()
        .into_iter()
        .find(|integration| integration.id == id)
        .unwrap_or_else(|| panic!("{id} integration"))
}

#[test]
fn issue_909_headless_global_configuration_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // R909-1: gemini's global configuration carries the settings file that
    // *selects* an auth type, not only the exports.
    let gemini = integration("gemini");
    let gemini_global = gemini.global_config_for("gemini");
    assert_eq!(gemini_global.format, ConfigFormat::ShellEnv);
    let settings = gemini_global
        .companion_files
        .iter()
        .find(|companion| companion.path == ".gemini/settings.json")
        .expect("gemini settings companion");
    assert_eq!(settings.format, ConfigFormat::Json);
    assert_eq!(settings.backup_suffix, ".formal-ai.bak");
    assert!(
        settings.json_settings.iter().any(
            |(key, value)| key == "security.auth.selectedType" && value == "{google_auth_type}"
        )
    );

    // R909-4: the contract is declared apart from the settings that write it.
    assert!(settings.headless_requirements.iter().any(|(kind, target)| {
        kind == "json" && target == ".gemini/settings.json:security.auth.selectedType"
    }));
    assert!(
        gemini_global
            .headless_requirements
            .iter()
            .any(|(kind, target)| kind == "env" && target == "GEMINI_API_KEY")
    );
    assert!(
        gemini_global
            .headless_requirements
            .iter()
            .any(|(kind, target)| kind == "env" && target == "{protocol_base_env}")
    );

    // R909-2: qwen selects its OpenAI auth path only on the complete triple.
    let qwen = integration("qwen");
    let qwen_global = qwen.global_config_for("openai");
    for name in ["OPENAI_API_KEY", "OPENAI_BASE_URL", "OPENAI_MODEL"] {
        assert!(
            qwen_global.shell_env.iter().any(|env| env.key == name),
            "qwen global configuration should export {name}"
        );
        assert!(
            qwen_global
                .headless_requirements
                .iter()
                .any(|(kind, target)| kind == "env" && target == name),
            "qwen should require {name} for a headless start"
        );
    }
    assert!(
        qwen_global
            .shell_env
            .iter()
            .any(|env| env.key == "OPENAI_MODEL" && env.value == "{model}")
    );

    // R909-3: the refusals that make a silent misconfiguration loud.
    for refusal in ["Invalid auth method selected", "No auth type is selected"] {
        for tool in [&gemini, &qwen] {
            assert!(
                tool.verification
                    .auth_refusals
                    .iter()
                    .any(|declared| declared == refusal),
                "{} should declare the auth refusal {refusal}",
                tool.id
            );
        }
    }

    let seed = read(root.join("data/seed/client-integrations.lino"));
    assert_contains_all(
        "data/seed/client-integrations.lino",
        &seed,
        &[
            "companion",
            "headless_require",
            "auth_refusal",
            "shell_env \"OPENAI_MODEL={model}\"",
        ],
    );

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "## Issue #909 Headless-Ready Global Client Configuration",
            "| R909-1 |",
            "| R909-2 |",
            "| R909-3 |",
            "| R909-4 |",
            "| R909-5 |",
            "| R909-6 |",
            // R909-7 came out of review on this issue rather than its body: the
            // seed sharding that keeps one changed block from dirtying the rest
            // of `data/seed`.
            "| R909-7 |",
        ],
    );

    let traceability = read(root.join("docs/requirements-traceability.md"));
    for id in [
        "R909-1", "R909-2", "R909-3", "R909-4", "R909-5", "R909-6", "R909-7",
    ] {
        assert!(
            traceability.contains(&format!("| {id} |")),
            "docs/requirements-traceability.md should carry a row for {id}"
        );
    }

    // Rule 8 of CONTRIBUTING: the issue keeps a case study with its raw data.
    let case_study = read(root.join("docs/case-studies/issue-909/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-909/README.md",
        &case_study,
        &["Invalid auth method selected", "OPENAI_MODEL", "R909-6"],
    );
    for artifact in ["raw-data/issue.json", "raw-data/issue-comments.json"] {
        assert!(
            root.join("docs/case-studies/issue-909")
                .join(artifact)
                .exists(),
            "docs/case-studies/issue-909/{artifact} should be preserved"
        );
    }

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "with-formal-ai -g --verify gemini",
            ".gemini/settings.json",
            "OPENAI_MODEL",
        ],
    );

    // R909-5: the reproduction stays runnable without a live client.
    assert!(
        root.join("experiments/issue-909-headless-config-gaps.sh")
            .exists()
    );
}

/// The verification surface speaks to the operator, so its four intents must
/// answer in every language the registry declares — english, russian, hindi,
/// chinese, spanish — not only in the language the fix was written in.
#[test]
fn issue_909_verification_messages_answer_in_every_registered_language() {
    let languages: Vec<&str> = formal_ai::language::registered_languages()
        .into_iter()
        .map(formal_ai::Language::slug)
        .collect();
    assert!(
        languages.contains(&"en"),
        "the registry should declare at least english, got {languages:?}"
    );

    for intent in [
        "client_global_config_requirement_missing",
        "client_global_config_auth_refusal",
        "client_global_config_probe_started",
        "client_global_config_probe_skipped",
    ] {
        for language in &languages {
            let text = formal_ai::seed::response_for(intent, language)
                .unwrap_or_else(|| panic!("{intent} should answer in {language}"));
            assert!(
                !text.trim().is_empty(),
                "{intent} should carry text in {language}"
            );
        }
    }
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
