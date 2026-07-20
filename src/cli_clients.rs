//! `formal-ai clients` — print the seed-baked registry of external agentic CLI
//! clients that `formal-ai with` can drive.
//!
//! The registry lives in `data/seed/client-integrations.lino`, so the same data
//! that builds a wrapper invocation also answers "which tools are supported and
//! what can each of them do?". Issue #671 needs that answer as machine-readable
//! data: the multi-CLI end-to-end matrix
//! (`experiments/agentic_cli_matrix/run_leg.sh`) derives its legs and its
//! per-leg capability assertions from this output, so a client added to the
//! seed cannot silently escape end-to-end coverage.

use std::fmt::Write as _;

use clap::ValueEnum;
use serde_json::{json, Map, Value};

use formal_ai::seed::{client_integrations, ClientIntegration, ConfigFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ClientsFormat {
    /// Human-readable listing, one block per client.
    Text,
    /// One JSON array describing every client.
    Json,
}

/// Render the client registry in the requested format.
#[must_use]
pub fn render_clients(format: ClientsFormat) -> String {
    let integrations = client_integrations();
    match format {
        ClientsFormat::Json => {
            let array = Value::Array(integrations.iter().map(client_json).collect());
            serde_json::to_string_pretty(&array).unwrap_or_else(|_| String::from("[]"))
        }
        ClientsFormat::Text => integrations.iter().map(client_text).collect(),
    }
}

const fn config_format_name(format: &ConfigFormat) -> &'static str {
    match format {
        ConfigFormat::Toml => "toml",
        ConfigFormat::Json => "json",
        ConfigFormat::ShellEnv => "shell_env",
    }
}

fn client_json(integration: &ClientIntegration) -> Value {
    let invocation = &integration.invocation;
    let mut endpoints = Map::new();
    for (protocol, path) in &integration.endpoints {
        endpoints.insert(protocol.clone(), Value::String(path.clone()));
    }
    let global_configs: Vec<Value> = integration
        .global_configs
        .iter()
        .map(|config| {
            json!({
                "protocol": if config.protocol.is_empty() {
                    integration.default_protocol.clone()
                } else {
                    config.protocol.clone()
                },
                "format": config_format_name(&config.format),
                "path": config.path,
                "backup_suffix": config.backup_suffix,
            })
        })
        .collect();

    json!({
        "id": integration.id,
        "aliases": integration.aliases,
        "label": integration.label,
        "command": integration.command,
        "command_env": integration.command_env,
        "platform_commands": integration
            .platform_commands
            .iter()
            .map(|entry| json!({ "platform": entry.key, "command": entry.value }))
            .collect::<Vec<_>>(),
        "provider_id": integration.provider_id,
        "default_protocol": integration.default_protocol,
        "supported_protocols": integration.supported_protocols,
        "endpoints": Value::Object(endpoints),
        "api_key_env": integration.api_key_env,
        "interactive_args": invocation.interactive_args,
        "interactive_args_require_prompt": invocation.interactive_args_require_prompt,
        "non_interactive_args": invocation.non_interactive_args,
        // A client with no headless invocation (the packaged desktop app, the
        // VS Code extension) can only ever be exercised interactively; the
        // matrix uses this flag to pick the leg shape instead of hardcoding ids.
        "supports_non_interactive": !invocation.non_interactive_args.is_empty(),
        "no_summarize_args": invocation.no_summarize_args,
        "supports_no_summarize": !invocation.no_summarize_args.is_empty(),
        "session_root": invocation.session_root,
        "global_configs": global_configs,
    })
}

fn client_text(integration: &ClientIntegration) -> String {
    let invocation = &integration.invocation;
    let mut out = format!("# {}\n", integration.id);
    let _ = writeln!(out, "  label: {}", integration.label);
    let _ = writeln!(out, "  command: {}", integration.command);
    if !integration.aliases.is_empty() {
        let _ = writeln!(out, "  aliases: {}", integration.aliases.join(", "));
    }
    let _ = writeln!(
        out,
        "  protocols: {} (default {})",
        integration.supported_protocols.join(", "),
        integration.default_protocol
    );
    for (protocol, path) in &integration.endpoints {
        let _ = writeln!(out, "  endpoint[{protocol}]: {path}");
    }
    let _ = writeln!(
        out,
        "  non_interactive: {}",
        if invocation.non_interactive_args.is_empty() {
            String::from("unsupported (interactive only)")
        } else {
            invocation.non_interactive_args.join(" ")
        }
    );
    let _ = writeln!(
        out,
        "  no_summarize: {}",
        if invocation.no_summarize_args.is_empty() {
            String::from("no client flag (server must answer summarize requests)")
        } else {
            invocation.no_summarize_args.join(" ")
        }
    );
    for config in &integration.global_configs {
        let _ = writeln!(
            out,
            "  global[{}]: {} ({})",
            if config.protocol.is_empty() {
                integration.default_protocol.as_str()
            } else {
                config.protocol.as_str()
            },
            config.path,
            config_format_name(&config.format)
        );
    }
    out.push('\n');
    out
}

pub fn run_clients(format: ClientsFormat) {
    print!("{}", render_clients(format));
}
