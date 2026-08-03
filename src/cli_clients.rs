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

use std::error::Error;
use std::fmt::Write as _;
use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};
use serde_json::{json, Map, Value};

use formal_ai::client_contract_learning::{
    learn_client_contracts, load_observations, observe_proxy_transcript,
};
use formal_ai::seed::{client_integrations, ClientIntegration, ConfigFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ClientsFormat {
    /// Human-readable listing, one block per client.
    Text,
    /// One JSON array describing every client.
    Json,
}

#[derive(Debug, Subcommand)]
pub enum ClientsAction {
    /// Normalize one real proxy transcript into a reusable observation.
    Observe {
        #[arg(long, value_name = "PATH")]
        transcript: PathBuf,
        #[arg(long)]
        client: String,
        #[arg(long)]
        capability: String,
        #[arg(long)]
        task_wording: String,
        #[arg(long, default_value = "")]
        expected_marker: String,
    },
    /// Infer human-gated contract amendments from observation JSONL files.
    Learn {
        #[arg(required = true, value_name = "OBSERVATIONS_JSONL")]
        observations: Vec<PathBuf>,
    },
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
    let verification = &integration.verification;
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

    let mut record = json!({
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
        "orchestration_args": invocation.orchestration_args,
        "vendor_orchestration_args": invocation.vendor_orchestration_args,
        "vendor_model_arg": invocation.vendor_model_arg,
        "orchestration_arg_replacements": invocation
            .orchestration_arg_replacements
            .iter()
            .map(|(from, to)| json!({ "from": from, "to": to }))
            .collect::<Vec<_>>(),
        // A client with no headless invocation (the packaged desktop app, the
        // VS Code extension) can only ever be exercised interactively; the
        // matrix uses this flag to pick the leg shape instead of hardcoding ids.
        "supports_non_interactive": !invocation.non_interactive_args.is_empty(),
        "no_summarize_args": invocation.no_summarize_args,
        "supports_no_summarize": !invocation.no_summarize_args.is_empty(),
        "session_root": invocation.session_root,
        "global_configs": global_configs,
        "verification": {
            "surface": verification.surface,
            "file_delivery": verification.file_delivery,
            "headless_args": verification.headless_args,
            "interactive_env": verification
                .interactive_env
                .iter()
                .map(|entry| format!("{}={}", entry.key, entry.value))
                .collect::<Vec<_>>(),
            "interactive_preamble": verification.interactive_preamble,
            "required_request_tools": verification.required_request_tools,
            "required_response_tools": verification.required_response_tools,
            "forbidden_output": verification.forbidden_output,
            "launch_args": verification.launch_args,
            "launch_ready": verification.launch_ready,
            "launch_required_output": verification.launch_required_output,
            "launch_http_path": verification.launch_http_path,
            "launch_subcommands": verification.launch_subcommands,
            "extension_glob": verification.extension_glob,
            "chromium_sandbox_fallback": verification.chromium_sandbox_fallback,
            "sandbox_user_namespaces": verification.sandbox_user_namespaces,
            "vendor_auth_error": verification.vendor_auth_error,
        },
    });
    // Added after construction rather than inline: the literal above already
    // sits at the json! macro's expansion depth.
    if let Some(object) = record.as_object_mut() {
        object.insert("prompt_args".to_string(), json!(invocation.prompt_args));
    }
    record
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
        "  orchestration: {}",
        invocation.orchestration_args.join(" ")
    );
    let _ = writeln!(
        out,
        "  vendor_orchestration: {}",
        invocation.vendor_orchestration_args.join(" ")
    );
    let _ = writeln!(out, "  vendor_model_arg: {}", invocation.vendor_model_arg);
    let orchestration_replacements = invocation
        .orchestration_arg_replacements
        .iter()
        .map(|(from, to)| format!("{from}={to}"))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(
        out,
        "  orchestration_replacements: {orchestration_replacements}"
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
    let _ = writeln!(
        out,
        "  verification_surface: {}",
        verification_surface(integration)
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

const fn verification_surface(integration: &ClientIntegration) -> &str {
    let surface = integration.verification.surface.as_str();
    if surface.is_empty() {
        "unspecified"
    } else {
        surface
    }
}

pub fn run_clients(
    format: ClientsFormat,
    action: Option<ClientsAction>,
) -> Result<(), Box<dyn Error>> {
    match action {
        None => print!("{}", render_clients(format)),
        Some(ClientsAction::Observe {
            transcript,
            client,
            capability,
            task_wording,
            expected_marker,
        }) => {
            let observation = observe_proxy_transcript(
                &transcript,
                &client,
                &capability,
                &task_wording,
                &expected_marker,
            )?;
            println!("{}", observation.json_line());
        }
        Some(ClientsAction::Learn { observations }) => {
            let observations = load_observations(&observations)?;
            let report = learn_client_contracts(&observations, &client_integrations());
            println!("{}", report.links_notation());
        }
    }
    Ok(())
}
