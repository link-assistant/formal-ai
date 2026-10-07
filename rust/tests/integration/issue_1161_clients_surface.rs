//! Issue #1161 R3: the relocation variables are a registry surface of the
//! real binary, not only parsed seed data. `formal-ai clients --format json`
//! carries `global_configs[].config_env` and the one-shot
//! `config_env`/`config_dir_env`/`config_content_env`/`temp_home_env` fields,
//! and the text form prints the variables on each `global[...]` line. The
//! projection lives in the bin module `src/cli_clients.rs`, which the lib test
//! target cannot import, so this test runs the built binary. Every expected
//! value is read from the parsed seed (`formal_ai::seed::client_integrations`),
//! so the test checks the projection, not a copy of the data.

use std::process::Command;

use formal_ai::seed::client_integrations;
use serde_json::Value;

fn clients(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .arg("clients")
        .args(args)
        .output()
        .expect("run formal-ai clients");
    assert!(
        output.status.success(),
        "formal-ai clients {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("clients output is UTF-8")
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn clients_json_carries_each_global_config_relocation_variable() {
    let parsed: Value = serde_json::from_str(&clients(&["--format", "json"]))
        .expect("clients --format json is JSON");
    let records = parsed.as_array().expect("an array of client records");
    let seeded = client_integrations();
    assert_eq!(records.len(), seeded.len(), "one record per seeded client");
    let mut declared = 0;
    for integration in &seeded {
        let record = records
            .iter()
            .find(|record| record["id"] == integration.id.as_str())
            .unwrap_or_else(|| panic!("{} has a JSON record", integration.id));
        let globals = record["global_configs"]
            .as_array()
            .unwrap_or_else(|| panic!("{} has global_configs", integration.id));
        assert_eq!(globals.len(), integration.global_configs.len());
        for (json, config) in globals.iter().zip(&integration.global_configs) {
            assert_eq!(json["path"], config.path.as_str());
            assert_eq!(
                strings(&json["config_env"]),
                config.config_env,
                "{} global {} config_env",
                integration.id,
                config.path
            );
            declared += config.config_env.len();
        }
        let invocation = &integration.invocation;
        for (field, expected) in [
            ("config_env", &invocation.config_env),
            ("config_dir_env", &invocation.config_dir_env),
            ("config_content_env", &invocation.config_content_env),
            ("temp_home_env", &invocation.temp_home_env),
        ] {
            assert_eq!(
                record[field],
                expected.as_str(),
                "{} top-level {field}",
                integration.id
            );
        }
        // Additive: the matrix still reads the verification surface it read
        // before issue #1161.
        assert!(
            record["verification"]["surface"].is_string(),
            "{} keeps verification.surface",
            integration.id
        );
    }
    assert!(
        declared >= 5,
        "the registry declares the agent, codex and opencode relocation variables"
    );
}

#[test]
fn clients_text_prints_the_relocation_variables_on_each_global_line() {
    let text = clients(&[]);
    for integration in client_integrations() {
        // Each client prints one block headed `# <id>` and ended by a blank line.
        let header = format!("# {}\n", integration.id);
        let start = text
            .find(&header)
            .unwrap_or_else(|| panic!("{} has a text block", integration.id));
        let block = &text[start..];
        let block = &block[..block.find("\n\n").unwrap_or(block.len())];
        for config in &integration.global_configs {
            let line = block
                .lines()
                .find(|line| {
                    line.trim_start().starts_with("global[")
                        && line.contains(&format!(": {} (", config.path))
                })
                .unwrap_or_else(|| {
                    panic!("{} prints a global line for {}", integration.id, config.path)
                });
            if config.config_env.is_empty() {
                assert!(!line.contains(" via "), "{line} declares no variable");
            } else {
                assert!(
                    line.ends_with(&format!(" via {}", config.config_env.join(" or "))),
                    "{line} names {:?}",
                    config.config_env
                );
            }
        }
    }
}
