//! Observable completion checks and machine-readable output for coding clients.

use std::error::Error;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Map, Value};

use crate::orchestration::workspace::{changes, snapshot, Snapshot, WorkspaceChange};
use crate::seed::{self, ClientCompletionContract};

use super::session_files::{native_session_id, newest_changed_session_file, SessionSnapshot};
use super::{build_invocation_args, ClientIntegration, InvocationOptions, RenderContext};

#[derive(Debug)]
pub(super) struct CapturedOutcome {
    pub(super) status_success: bool,
    pub(super) status_label: String,
    pub(super) completion_passed: bool,
}

pub(super) fn require_completed(
    completion_passed: Option<bool>,
    status_success: bool,
    status_label: &str,
    command: &Path,
) -> Result<(), Box<dyn Error>> {
    const COMMAND_PLACEHOLDER: &str = concat!("{", "command", "}");
    const STATUS_PLACEHOLDER: &str = concat!("{", "status", "}");
    if completion_passed.unwrap_or(status_success) {
        return Ok(());
    }
    let contract = seed::software_authoring_completion_contract()
        .ok_or("software-authoring completion contract is unavailable")?;
    let template = if completion_passed.is_some() && status_success {
        &contract.incomplete_error
    } else {
        &contract.process_error
    };
    Err(template
        .replace(COMMAND_PLACEHOLDER, &command.display().to_string())
        .replace(STATUS_PLACEHOLDER, status_label)
        .into())
}

#[derive(Debug)]
pub(super) struct AuthoringRun {
    pub(super) before: Snapshot,
    pub(super) workspace: PathBuf,
    pub(super) contract: ClientCompletionContract,
    pub(super) attempts: usize,
    pub(super) input_tokens: u64,
    pub(super) output_tokens: u64,
    pub(super) endpoint_mismatch: Option<String>,
}

impl AuthoringRun {
    pub(super) fn for_invocation(
        user_args: &[String],
        options: &mut InvocationOptions,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        if options.force_interactive {
            return Ok(None);
        }
        let run = Self::start(&user_args.join(" "))?;
        if run.is_some() {
            options.orchestration = true;
        }
        Ok(run)
    }

    pub(super) fn start(task: &str) -> Result<Option<Self>, Box<dyn Error>> {
        if !requires_workspace_effect(task) {
            return Ok(None);
        }
        let contract = seed::software_authoring_completion_contract()
            .ok_or("software-authoring completion contract is unavailable")?;
        if contract.observable_postcondition != "workspace_effect" || contract.max_attempts == 0 {
            return Err("software-authoring completion contract is invalid".into());
        }
        let workspace = std::env::current_dir()?;
        ignore_scratch_directory(&workspace, &contract.scratch_directory)?;
        let before = snapshot(&workspace)?;
        Ok(Some(Self {
            before,
            workspace,
            contract,
            attempts: 0,
            input_tokens: 0,
            output_tokens: 0,
            endpoint_mismatch: None,
        }))
    }

    pub(super) fn observe_output(
        &mut self,
        output: &Output,
        expected_endpoint: &str,
    ) -> Result<(), Box<dyn Error>> {
        self.attempts += 1;
        let records = normalize_json_stream(&output.stdout);
        for record in &records {
            collect_token_usage(record, &mut self.input_tokens, &mut self.output_tokens);
            println!("{}", serde_json::to_string(record)?);
        }
        std::io::stdout().flush()?;
        std::io::stderr().write_all(&output.stderr)?;
        self.endpoint_mismatch = self
            .endpoint_mismatch
            .take()
            .or_else(|| detect_vendor_endpoint(&output.stdout, &output.stderr, expected_endpoint));
        Ok(())
    }

    pub(super) fn changes(&self) -> Result<Vec<WorkspaceChange>, Box<dyn Error>> {
        let after = snapshot(&self.workspace)?;
        Ok(changes(&self.before, &after))
    }

    pub(super) const fn can_retry(&self) -> bool {
        self.attempts < self.contract.max_attempts
    }

    pub(super) fn completion_prompt(task: &str) -> Result<String, Box<dyn Error>> {
        const TASK_PLACEHOLDER: &str = concat!("{", "task", "}");
        const CLAIM_PLACEHOLDER: &str = concat!("{", "claim", "}");
        const EVIDENCE_PLACEHOLDER: &str = concat!("{", "evidence", "}");
        let language = crate::language::detect(task).slug();
        let template = seed::response_for("orchestration_correction_prompt", language)
            .or_else(|| seed::response_for("orchestration_correction_prompt", "en"))
            .ok_or("orchestration correction prompt is unavailable")?;
        Ok(template
            .replace(TASK_PLACEHOLDER, task)
            .replace(CLAIM_PLACEHOLDER, "workspace_effect_present=true")
            .replace(EVIDENCE_PLACEHOLDER, "workspace_effect_count=0"))
    }

    pub(super) fn print_completion(
        &self,
        context: &RenderContext,
        status_success: bool,
        workspace_changes: &[WorkspaceChange],
    ) -> Result<bool, Box<dyn Error>> {
        let (completion_state, reason, passed) = if self.endpoint_mismatch.is_some() {
            ("failed", "endpoint_mismatch", false)
        } else if !status_success {
            ("failed", "client_failed", false)
        } else if workspace_changes.is_empty() {
            (
                "incomplete",
                self.contract.incomplete_reason.as_str(),
                false,
            )
        } else {
            ("complete", "workspace_effect_observed", true)
        };
        let paths = workspace_changes
            .iter()
            .map(|change| change.path.as_str())
            .collect::<Vec<_>>();
        let record = json!({
            "type": "formal_ai_completion",
            "completion_state": completion_state,
            "reason": reason,
            "attempts": self.attempts,
            "observable_postcondition": {
                "kind": self.contract.observable_postcondition,
                "expected": true,
                "observed": !workspace_changes.is_empty(),
                "paths": paths,
            },
            "rawMetadata": {
                "formalai": {
                    "model": context.model,
                    "endpoint": context.endpoint_base_url,
                    "input_tokens": self.input_tokens,
                    "output_tokens": self.output_tokens,
                    "actual_endpoint": self.endpoint_mismatch,
                }
            }
        });
        println!("{}", serde_json::to_string(&record)?);
        Ok(passed)
    }
}

#[derive(Clone, Copy)]
pub(super) struct CompletionInvocation<'a> {
    pub(super) command: &'a Command,
    pub(super) integration: &'a ClientIntegration,
    pub(super) user_args: &'a [String],
    pub(super) context: &'a RenderContext,
    pub(super) options: InvocationOptions,
    pub(super) session_root: Option<&'a Path>,
    pub(super) session_before: &'a SessionSnapshot,
    pub(super) initial_resume: Option<&'a str>,
}

pub(super) fn run_to_completion(
    authoring: &mut AuthoringRun,
    invocation: CompletionInvocation<'_>,
) -> Result<CapturedOutcome, Box<dyn Error>> {
    let CompletionInvocation {
        command,
        integration,
        user_args,
        context,
        options,
        session_root,
        session_before,
        initial_resume,
    } = invocation;
    let task = user_args.join(" ");
    let initial_args =
        build_invocation_args(integration, user_args, context, options, initial_resume);
    let mut output = clone_command(command, &initial_args).output()?;
    authoring.observe_output(&output, &context.endpoint_base_url)?;
    let mut workspace_changes = authoring.changes()?;
    if output.status.success()
        && authoring.endpoint_mismatch.is_none()
        && workspace_changes.is_empty()
        && authoring.can_retry()
    {
        let initial_session_file = session_root.and_then(|root| {
            newest_changed_session_file(
                root,
                &integration.invocation.session_file_suffix,
                session_before,
            )
        });
        let resume_id = initial_session_file
            .as_deref()
            .and_then(|path| native_session_id(integration, path))
            .or_else(|| initial_resume.map(str::to_owned));
        let correction = AuthoringRun::completion_prompt(&task)?;
        let retry_args = build_invocation_args(
            integration,
            &[correction],
            context,
            options,
            resume_id.as_deref(),
        );
        output = clone_command(command, &retry_args).output()?;
        authoring.observe_output(&output, &context.endpoint_base_url)?;
        workspace_changes = authoring.changes()?;
    }
    let status_success = output.status.success();
    let status_label = output
        .status
        .code()
        .map_or_else(|| String::from("signal"), |code| code.to_string());
    let completion_passed =
        authoring.print_completion(context, status_success, &workspace_changes)?;
    Ok(CapturedOutcome {
        status_success,
        status_label,
        completion_passed,
    })
}

fn clone_command(template: &Command, args: &[String]) -> Command {
    let mut command = Command::new(template.get_program());
    for (key, value) in template.get_envs() {
        match value {
            Some(value) => {
                command.env(key, value);
            }
            None => {
                command.env_remove(key);
            }
        }
    }
    if let Some(directory) = template.get_current_dir() {
        command.current_dir(directory);
    }
    command.args(args);
    command
}

fn requires_workspace_effect(task: &str) -> bool {
    let normalized = crate::engine::normalize_prompt(task);
    let lexicon = seed::lexicon();
    lexicon.mentions_role(seed::ROLE_SOFTWARE_AUTHORING_ACTION, &normalized)
        || lexicon.mentions_role_raw(seed::ROLE_SOFTWARE_AUTHORING_ACTION, &normalized)
}

fn ignore_scratch_directory(workspace: &Path, scratch: &str) -> Result<(), Box<dyn Error>> {
    if scratch.is_empty() || Path::new(scratch).is_absolute() || scratch.contains("..") {
        return Err("completion scratch directory must be a safe relative path".into());
    }
    let output = Command::new("git")
        .args(["rev-parse", "--git-path", "info/exclude"])
        .current_dir(workspace)
        .output()?;
    if !output.status.success() {
        return Ok(());
    }
    let raw_path = String::from_utf8(output.stdout)?;
    let raw_path = raw_path.trim();
    if raw_path.is_empty() {
        return Ok(());
    }
    let path = if Path::new(raw_path).is_absolute() {
        PathBuf::from(raw_path)
    } else {
        workspace.join(raw_path)
    };
    let pattern = format!("{}/", scratch.trim_end_matches('/'));
    let existing = fs::read_to_string(&path).unwrap_or_default();
    if existing.lines().any(|line| line.trim() == pattern) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    if !existing.is_empty() && !existing.ends_with('\n') {
        writeln!(file)?;
    }
    writeln!(file, "{pattern}")?;
    Ok(())
}

fn normalize_json_stream(bytes: &[u8]) -> Vec<Value> {
    let text = String::from_utf8_lossy(bytes);
    let mut records = Vec::new();
    let mut remaining = text.as_ref();
    while !remaining.trim_start().is_empty() {
        remaining = remaining.trim_start();
        let mut stream = serde_json::Deserializer::from_str(remaining).into_iter::<Value>();
        if let Some(Ok(value)) = stream.next() {
            let consumed = stream.byte_offset();
            records.push(value);
            remaining = &remaining[consumed..];
        } else {
            let split = remaining.find('\n').unwrap_or(remaining.len());
            let line = remaining[..split].trim_end_matches('\r');
            if !line.is_empty() {
                records.push(json!({
                    "type": "formal_ai_client_output",
                    "stream": "stdout",
                    "text": line,
                }));
            }
            remaining = remaining.get(split + 1..).unwrap_or_default();
        }
    }
    records
}

fn collect_token_usage(value: &Value, input: &mut u64, output: &mut u64) {
    match value {
        Value::Object(object) => {
            collect_object_token_usage(object, input, output);
            for (key, child) in object {
                if matches!(
                    key.as_str(),
                    "tokens" | "token_usage" | "tokenUsage" | "tokensBreakdown"
                ) {
                    collect_short_token_usage(child, input, output);
                }
                collect_token_usage(child, input, output);
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_token_usage(child, input, output);
            }
        }
        Value::String(text) => {
            if let Ok(nested) = serde_json::from_str::<Value>(text) {
                collect_token_usage(&nested, input, output);
            } else {
                for nested in normalize_json_stream(text.as_bytes()) {
                    if nested["type"] != "formal_ai_client_output" {
                        collect_token_usage(&nested, input, output);
                    }
                }
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn collect_short_token_usage(value: &Value, input: &mut u64, output: &mut u64) {
    let Value::Object(object) = value else {
        return;
    };
    if let Some(value) = object.get("input").and_then(Value::as_u64) {
        *input = (*input).max(value);
    }
    if let Some(value) = object.get("output").and_then(Value::as_u64) {
        *output = (*output).max(value);
    }
}

fn collect_object_token_usage(object: &Map<String, Value>, input: &mut u64, output: &mut u64) {
    for key in [
        "input_tokens",
        "inputTokens",
        "prompt_tokens",
        "promptTokens",
    ] {
        if let Some(value) = object.get(key).and_then(Value::as_u64) {
            *input = (*input).max(value);
        }
    }
    for key in [
        "output_tokens",
        "outputTokens",
        "completion_tokens",
        "completionTokens",
    ] {
        if let Some(value) = object.get(key).and_then(Value::as_u64) {
            *output = (*output).max(value);
        }
    }
}

fn detect_vendor_endpoint(stdout: &[u8], stderr: &[u8], expected: &str) -> Option<String> {
    const VENDOR_ENDPOINTS: [&str; 3] = [
        "https://api.openai.com",
        "https://api.anthropic.com",
        "https://generativelanguage.googleapis.com",
    ];
    let output = format!(
        "{}\n{}",
        String::from_utf8_lossy(stdout),
        String::from_utf8_lossy(stderr)
    );
    VENDOR_ENDPOINTS
        .into_iter()
        .find(|endpoint| !expected.starts_with(endpoint) && output.contains(endpoint))
        .map(str::to_owned)
}
