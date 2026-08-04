//! Checking that a global client configuration can actually start the client.
//!
//! `--global` used to report success as soon as it had written its exports,
//! so a client that still refused to start headlessly — gemini with no
//! *selected* auth type, qwen with an incomplete OpenAI triple — only failed
//! much later, with an error that named nothing about the configuration. Two
//! checks close that gap:
//!
//! * a readback check that re-reads the files just written and confirms every
//!   seed-declared headless requirement resolves to a non-empty value, and
//! * an opt-in (`--verify`) one-shot probe that starts the client
//!   non-interactively and fails on the seed-declared auth refusals.

use std::error::Error;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use toml_edit::DocumentMut;

use crate::seed::{ClientIntegration, ClientIntegrationGlobalConfig, ConfigFormat};

use super::command::{command_available, resolve_integration_command};
use super::{global_config_path, render_template, RenderContext};

/// How long a startup probe may run before it is stopped. The probe only needs
/// the client's first reaction to the configuration, not a full answer.
const PROBE_TIMEOUT: Duration = Duration::from_mins(1);

/// Prompt sent to the probed client. It is a plain word, not a task.
const PROBE_PROMPT: &str = "ping";

const TOOL_PLACEHOLDER: &str = concat!("{", "tool", "}");
const REQUIREMENT_PLACEHOLDER: &str = concat!("{", "requirement", "}");
const PATH_PLACEHOLDER: &str = concat!("{", "path", "}");
const OUTPUT_PLACEHOLDER: &str = concat!("{", "output", "}");
const COMMAND_PLACEHOLDER: &str = concat!("{", "command", "}");

/// A single seed-declared piece the configured client needs, resolved against
/// the file that is supposed to carry it.
struct Requirement {
    kind: String,
    target: String,
    /// File the requirement is read back from: the declaring node's own config
    /// file for `env`, the path named by the requirement for `json`/`toml`.
    source: PathBuf,
    tool: String,
}

impl Requirement {
    /// Human-readable form used in the failure message, for example
    /// `env:OPENAI_MODEL` or `json:.gemini/settings.json:security.auth.selectedType`.
    fn label(&self) -> String {
        format!("{}:{}", self.kind, self.target)
    }
}

/// Re-read what `--global` just wrote and fail when a declared headless
/// requirement is missing or empty.
pub(super) fn verify_written_config(
    integration: &ClientIntegration,
    config: &ClientIntegrationGlobalConfig,
    context: &RenderContext,
) -> Result<(), Box<dyn Error>> {
    for requirement in collect_requirements(integration, config, context)? {
        if !is_satisfied(&requirement)? {
            return Err(missing_requirement_error(&requirement).into());
        }
    }
    Ok(())
}

fn collect_requirements(
    integration: &ClientIntegration,
    config: &ClientIntegrationGlobalConfig,
    context: &RenderContext,
) -> Result<Vec<Requirement>, Box<dyn Error>> {
    let mut requirements = Vec::new();
    for node in std::iter::once(config).chain(config.companions.iter()) {
        let node_path = global_config_path(&node.path)?;
        for (kind, target) in &node.headless_requirements {
            let target = render_template(target, context);
            let source = if kind == "env" {
                node_path.clone()
            } else {
                let (relative, _) = split_file_target(&target);
                global_config_path(relative)?
            };
            requirements.push(Requirement {
                kind: kind.clone(),
                target,
                source,
                tool: integration.id.clone(),
            });
        }
    }
    Ok(requirements)
}

/// Split `<relative path>:<dotted key>` into its two halves.
fn split_file_target(target: &str) -> (&str, &str) {
    target.split_once(':').unwrap_or((target, ""))
}

fn is_satisfied(requirement: &Requirement) -> Result<bool, Box<dyn Error>> {
    let Ok(contents) = fs::read_to_string(&requirement.source) else {
        return Ok(false);
    };
    match requirement.kind.as_str() {
        "env" => Ok(
            exported_value(&contents, &requirement.tool, &requirement.target)
                .is_some_and(|value| !value.is_empty()),
        ),
        "json" => {
            let (_, key) = split_file_target(&requirement.target);
            let root: Value = serde_json::from_str(&contents)?;
            Ok(json_value_at(&root, key).is_some_and(is_present))
        }
        "toml" => {
            let (_, key) = split_file_target(&requirement.target);
            let document = contents.parse::<DocumentMut>()?;
            Ok(toml_string_at(&document, key).is_some_and(|value| !value.is_empty()))
        }
        _ => Ok(false),
    }
}

/// Read back the value a managed shell block exports for `name`.
///
/// Only the block this tool owns is inspected, so an unrelated export left by
/// the operator elsewhere in the profile cannot make a requirement look
/// satisfied.
fn exported_value(contents: &str, tool: &str, name: &str) -> Option<String> {
    let start = format!("# >>> formal-ai {tool}");
    let end = format!("# <<< formal-ai {tool}");
    let prefix = format!("export {name}=");
    let mut inside = false;
    for line in contents.lines() {
        if line == start {
            inside = true;
            continue;
        }
        if inside && line == end {
            inside = false;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(value) = line.strip_prefix(&prefix) {
            return Some(unquote(value.trim()));
        }
    }
    None
}

fn unquote(value: &str) -> String {
    let trimmed = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value);
    trimmed.replace("\\\"", "\"").replace("\\\\", "\\")
}

fn json_value_at<'a>(root: &'a Value, dotted_path: &str) -> Option<&'a Value> {
    let mut current = root;
    for part in dotted_path.split('.').filter(|part| !part.is_empty()) {
        current = current.as_object()?.get(part)?;
    }
    Some(current)
}

const fn is_present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(text) => !text.is_empty(),
        _ => true,
    }
}

fn toml_string_at(document: &DocumentMut, dotted_path: &str) -> Option<String> {
    let mut item = document.as_item();
    for part in dotted_path.split('.').filter(|part| !part.is_empty()) {
        item = item.as_table_like()?.get(part)?;
    }
    item.as_str().map(ToOwned::to_owned)
}

fn missing_requirement_error(requirement: &Requirement) -> String {
    seed_text("client_global_config_requirement_missing")
        .replace(TOOL_PLACEHOLDER, &requirement.tool)
        .replace(REQUIREMENT_PLACEHOLDER, &requirement.label())
        .replace(PATH_PLACEHOLDER, &requirement.source.display().to_string())
}

fn seed_text(intent: &str) -> String {
    crate::seed::response_for(intent, "en").unwrap_or_else(|| intent.to_owned())
}

/// Start the configured client once, non-interactively, and fail when it
/// answers with one of its seed-declared auth refusals.
pub(super) fn probe_headless_start(
    integration: &ClientIntegration,
    config: &ClientIntegrationGlobalConfig,
    context: &RenderContext,
) -> Result<(), Box<dyn Error>> {
    let command = resolve_integration_command(integration);
    // A client that is configured but not installed, or that is not a
    // command-line client at all, has nothing to probe — that is not a failure
    // of the configuration just written.
    if integration.verification.surface != "cli" || !command_available(&command) {
        println!("{}", probe_skipped_message(&integration.id, &command));
        return Ok(());
    }
    let mut arguments = integration
        .invocation
        .non_interactive_args
        .iter()
        .map(|argument| render_template(argument, context))
        .collect::<Vec<_>>();
    arguments.push(PROBE_PROMPT.to_owned());

    let profile = match config.format {
        ConfigFormat::ShellEnv => Some(global_config_path(&config.path)?),
        ConfigFormat::Json | ConfigFormat::Toml => None,
    };
    let output = run_probe(&command, &arguments, profile.as_deref())?;
    if let Some(refusal) = integration
        .verification
        .auth_refusals
        .iter()
        .find(|refusal| output.contains(refusal.as_str()))
    {
        return Err(seed_text("client_global_config_auth_refusal")
            .replace(TOOL_PLACEHOLDER, &integration.id)
            .replace(OUTPUT_PLACEHOLDER, refusal)
            .into());
    }
    println!(
        "{}",
        seed_text("client_global_config_probe_started")
            .replace(TOOL_PLACEHOLDER, &integration.id)
            .replace(
                PATH_PLACEHOLDER,
                &global_config_path(&config.path)?.display().to_string()
            )
    );
    Ok(())
}

fn probe_skipped_message(tool: &str, command: &Path) -> String {
    seed_text("client_global_config_probe_skipped")
        .replace(TOOL_PLACEHOLDER, tool)
        .replace(COMMAND_PLACEHOLDER, &command.display().to_string())
}

/// Run the probe and return its combined output, bounded by `PROBE_TIMEOUT`.
///
/// A shell-profile configuration is sourced first, so the probe sees exactly
/// what a fresh login shell would see rather than this process's environment.
fn run_probe(
    program: &Path,
    arguments: &[String],
    profile: Option<&Path>,
) -> Result<String, Box<dyn Error>> {
    let mut command = profile.map_or_else(
        || Command::new(program),
        |profile| {
            let mut shell = Command::new("sh");
            shell.arg("-c").arg(format!(
                ". {} >/dev/null 2>&1 || true; exec \"$0\" \"$@\"",
                shell_quote(&profile.display().to_string())
            ));
            shell.arg(program);
            shell
        },
    );
    command.args(arguments);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let deadline = Instant::now() + PROBE_TIMEOUT;
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let mut output = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        let _ = stdout.read_to_string(&mut output);
    }
    if let Some(mut stderr) = child.stderr.take() {
        let _ = stderr.read_to_string(&mut output);
    }
    Ok(output)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_block_exports_are_read_back() {
        let profile = concat!(
            "export OPENAI_MODEL=\"other\"\n",
            "# >>> formal-ai qwen\n",
            "export OPENAI_API_KEY=\"formal-ai\"\n",
            "export OPENAI_MODEL=\"formal-ai\"\n",
            "# <<< formal-ai qwen\n",
        );
        assert_eq!(
            exported_value(profile, "qwen", "OPENAI_MODEL").as_deref(),
            Some("formal-ai")
        );
        assert_eq!(exported_value(profile, "qwen", "OPENAI_BASE_URL"), None);
        // An export outside the managed block belongs to the operator and must
        // not stand in for a missing managed one.
        assert_eq!(exported_value(profile, "gemini", "OPENAI_MODEL"), None);
    }

    #[test]
    fn dotted_json_paths_resolve_to_present_values() {
        let root: Value = serde_json::from_str(
            r#"{"security":{"auth":{"selectedType":"gemini-api-key","empty":""}}}"#,
        )
        .expect("json");
        assert!(json_value_at(&root, "security.auth.selectedType").is_some_and(is_present));
        assert!(!json_value_at(&root, "security.auth.empty").is_some_and(is_present));
        assert!(json_value_at(&root, "security.auth.missing").is_none());
    }
}
