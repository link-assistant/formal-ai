//! The client registry must *name* the env var that points a client at its
//! config, not imply an `XDG_CONFIG_HOME` relocation (issue #1161).
//!
//! On 2026-09-27 a Hive Mind host isolated the Formal AI agent's config by
//! relocating `XDG_CONFIG_HOME` ([link-assistant/hive-mind#2314]). That
//! variable steers every XDG-respecting program at once, so `gh`'s
//! `hosts.yml` and git's config moved with it and the session lost its GitHub
//! authentication. The fix is declarative: every `global_configs` entry in
//! `data/seed/client-integrations.lino` may carry `config_env "VAR"` children
//! naming the variables that relocate that entry's own file, and a host must
//! use those instead of moving the home directory.
//!
//! These tests pin the declarations per client and the rule against
//! fabricating variables for entries that have none. The JSON projection
//! (`formal-ai clients --format json`, `global_configs[].config_env`) is a
//! thin additive mapping in `src/cli_clients.rs`, a bin module that the lib
//! test target cannot import, so the assertions here read the parsed seed
//! structs — the same data the JSON is rendered from — plus the `.lino`
//! sources directly.

use std::path::PathBuf;

use formal_ai::seed::{
    ClientIntegration, ClientIntegrationGlobalConfig, ConfigFormat, client_integrations,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn integration(id: &str) -> ClientIntegration {
    client_integrations()
        .into_iter()
        .find(|integration| integration.id == id)
        .unwrap_or_else(|| panic!("{id} integration should be seeded"))
}

/// The default-protocol global config (the `global` block with no protocol
/// qualifier) of one client.
fn default_global(id: &str) -> ClientIntegrationGlobalConfig {
    let integration = integration(id);
    integration
        .global_configs
        .into_iter()
        .find(|config| config.protocol.is_empty())
        .unwrap_or_else(|| panic!("{id} should have a default-protocol global config"))
}

#[test]
fn agent_global_config_names_its_own_relocation_env() {
    let global = default_global("agent");
    assert_eq!(global.path, ".config/link-assistant-agent/opencode.json");
    assert!(
        global
            .config_env
            .contains(&"LINK_ASSISTANT_AGENT_CONFIG".to_owned()),
        "agent config file must be relocatable via LINK_ASSISTANT_AGENT_CONFIG, got {:?}",
        global.config_env
    );
    assert!(
        global
            .config_env
            .contains(&"LINK_ASSISTANT_AGENT_CONFIG_DIR".to_owned()),
        "agent config dir must be relocatable via LINK_ASSISTANT_AGENT_CONFIG_DIR, got {:?}",
        global.config_env
    );
    assert_eq!(
        global.config_env.len(),
        2,
        "agent declares exactly the two variables a host may set, got {:?}",
        global.config_env
    );
}

#[test]
fn codex_family_global_configs_name_codex_home() {
    let codex = default_global("codex");
    assert_eq!(codex.path, ".codex/config.toml");
    assert!(
        codex.config_env.contains(&"CODEX_HOME".to_owned()),
        "codex global config must be relocatable via CODEX_HOME, got {:?}",
        codex.config_env
    );

    // T3 Code rides the same Codex config file for its OpenAI protocol and
    // already isolates that file through CODEX_HOME in one-shot mode.
    let t3 = integration("t3code");
    let openai_global = t3
        .global_configs
        .into_iter()
        .find(|config| config.protocol == "openai")
        .expect("t3code should have an openai-protocol global config");
    assert!(
        openai_global.config_env.contains(&"CODEX_HOME".to_owned()),
        "t3code openai global config must be relocatable via CODEX_HOME, got {:?}",
        openai_global.config_env
    );
}

#[test]
fn opencode_family_global_configs_name_both_opencode_variables() {
    for id in ["opencode", "opencode-vscode", "opencode-desktop"] {
        let global = default_global(id);
        assert_eq!(global.path, ".config/opencode/opencode.json");
        for variable in ["OPENCODE_CONFIG", "OPENCODE_CONFIG_DIR"] {
            assert!(
                global.config_env.contains(&variable.to_owned()),
                "{id} global config must be relocatable via {variable}, got {:?}",
                global.config_env
            );
        }
    }
}

#[test]
fn entries_without_a_relocation_variable_declare_none() {
    // Shell-profile globals (.profile) and fixed-path clients with no
    // in-repo evidence of a relocation variable must declare an empty list,
    // so a host never reads a fabricated variable and relocates the wrong
    // thing (the exact failure mode of issue #1161).
    for id in ["claude", "gemini", "qwen", "aider", "cursor", "grok"] {
        let integration = integration(id);
        for config in &integration.global_configs {
            assert!(
                config.config_env.is_empty(),
                "{id} global[{}] declares {:?} but no relocation variable is evidenced for {}",
                if config.protocol.is_empty() {
                    integration.default_protocol.as_str()
                } else {
                    config.protocol.as_str()
                },
                config.config_env,
                config.path
            );
        }
    }
}

#[test]
fn config_env_is_additive_to_the_existing_file_contract() {
    // The new declaration must not displace the fields the `with --global`
    // writer and its undo path already rely on.
    let agent = default_global("agent");
    assert_eq!(agent.format, ConfigFormat::Json);
    assert_eq!(agent.backup_suffix, ".formal-ai.bak");
    assert_ne!(
        agent.json_settings,
        [] as [(std::string::String, std::string::String); 0]
    );

    let codex = default_global("codex");
    assert_eq!(codex.format, ConfigFormat::Toml);
    assert_ne!(
        codex.toml_settings,
        [] as [(std::string::String, std::string::String); 0]
    );
}

#[test]
fn embedded_mirror_of_the_seed_file_stays_in_sync() {
    // `client_integrations()` parses the *embedded* mirror
    // (`rust/embedded/data/seed/client-integrations.lino`) that
    // `scripts/mirror-package-data.rs --write` keeps byte-equal to
    // `data/seed/client-integrations.lino`. A hand edit of the source that
    // skips the mirror would make every declaration test above read stale
    // data, so the two files are asserted byte-identical here.
    let source = read("data/seed/client-integrations.lino");
    let mirror = read("rust/embedded/data/seed/client-integrations.lino");
    assert_eq!(
        source, mirror,
        "data/seed/client-integrations.lino and its rust/embedded mirror diverged; \
         run `rust-script scripts/mirror-package-data.rs --write`"
    );
    for expected in [
        "config_env \"CODEX_HOME\"",
        "config_env \"OPENCODE_CONFIG\"",
        "config_env \"OPENCODE_CONFIG_DIR\"",
        "config_env \"LINK_ASSISTANT_AGENT_CONFIG\"",
        "config_env \"LINK_ASSISTANT_AGENT_CONFIG_DIR\"",
    ] {
        assert!(
            source.contains(expected),
            "seed file should carry the declaration {expected}"
        );
    }
}

/// R7: the relocation is exercised end to end, not only declared. The matrix
/// leg configures the client into a scratch home, moves the written directory
/// out, and starts the real CLI with an empty `HOME` and only the declared
/// variable pointing at the moved config; the session must then reach the
/// server through the recording proxy. The case reads the variables from the
/// registry (`global_configs[0].config_env`), so every client that declares
/// one is covered and none is named in the script.
#[test]
fn the_matrix_leg_serves_a_session_from_each_declared_relocation_variable() {
    let leg = read("experiments/agentic_cli_matrix/run_leg.sh");
    let start = leg
        .find("case_relocated() {")
        .expect("run_leg.sh defines the relocated case");
    let body = &leg[start..start + leg[start..].find("\n}\n").expect("case body ends")];
    for needle in [
        ".global_configs[0].config_env",
        "with --globally",
        "env HOME=\"$clean\"",
        "\"$variable=$value\"",
        "matrix_assert_proxy_ok",
        // The client's own default model outranks its config's `model`
        // (agentic CLI matrix run 37594789426), so the model is passed.
        ".model_arg",
        ".model_arg_after_first_arg",
    ] {
        assert!(body.contains(needle), "the relocated case lost `{needle}`");
    }
    assert!(
        !body.contains("XDG_CONFIG_HOME"),
        "the relocated case must not fall back to relocating XDG_CONFIG_HOME (issue #1161)"
    );
    assert!(
        leg.contains("run_case relocated case_relocated"),
        "run_leg.sh never runs the relocated case"
    );
}
