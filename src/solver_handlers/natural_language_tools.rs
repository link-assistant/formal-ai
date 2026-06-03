//! Natural-language tool/API execution with explicit policy gates.

use crate::associative_package::{default_package_store, PackagePermissionDecision};
use crate::calculation::evaluate_calculation;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed;
use crate::solver_helpers::{
    extract_backticked, extract_javascript_program, extract_quoted_phrase,
};

use super::finalize_simple;
use super::web_requests::{WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K};

pub fn try_natural_language_tool_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    agent_mode: bool,
) -> Option<SymbolicAnswer> {
    if let Some(answer) = try_javascript_code_execution(prompt, log, agent_mode) {
        return Some(answer);
    }
    if let Some(answer) = try_calculator_api_call(prompt, normalized, log, agent_mode) {
        return Some(answer);
    }
    if let Some(answer) = try_web_search_api_call(prompt, normalized, log, agent_mode) {
        return Some(answer);
    }
    if let Some(answer) = try_local_shell_tool_call(prompt, normalized, log, agent_mode) {
        return Some(answer);
    }
    None
}

fn try_javascript_code_execution(
    prompt: &str,
    log: &mut EventLog,
    agent_mode: bool,
) -> Option<SymbolicAnswer> {
    let program = extract_javascript_program(prompt)?;
    if let Some(denial) = require_tool_permission(log, agent_mode, "javascript_execution") {
        log.append("execution_status", "javascript:refused".to_owned());
        log.append("execution_environment", "agent-permission-gate".to_owned());
        let body = format!("{denial}\n\nRequested source:\n```js\n{program}\n```");
        return Some(finalize_simple(
            prompt,
            log,
            "tool_call_refused",
            "response:tool_call_refused",
            &body,
            1.0,
        ));
    }

    log.append("tool_call", "javascript_execution".to_owned());
    log.append("tool_parameter", format!("source={program}"));
    log.append("execution:request", "javascript".to_owned());
    log.append("execution:source", program.clone());
    log.append(
        "execution_environment",
        "formal-ai deterministic javascript subset".to_owned(),
    );

    let (status, result) = execute_javascript_subset(&program);
    log.append("execution_status", format!("javascript:{status}"));
    log.append("tool_result", result.clone());
    let body =
        format!("Execution status: {status}.\nTool call: javascript_execution\nOutput: {result}");
    Some(finalize_simple(
        prompt,
        log,
        "javascript_execution",
        "response:javascript_execution",
        &body,
        0.9,
    ))
}

fn try_calculator_api_call(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    agent_mode: bool,
) -> Option<SymbolicAnswer> {
    if !is_explicit_tool_api_request(normalized, seed::ROLE_CALCULATOR_TOOL_NAME) {
        return None;
    }
    let expression = extract_argument(prompt, normalized)?;
    if let Some(denial) = require_tool_permission(log, agent_mode, "calculator") {
        log.append("execution_status", "calculator:refused".to_owned());
        log.append("execution_environment", "agent-permission-gate".to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            "tool_call_refused",
            "response:tool_call_refused",
            &denial,
            1.0,
        ));
    }
    log.append("tool_call", "calculator".to_owned());
    log.append("tool_parameter", format!("expression={expression}"));
    match evaluate_calculation(&expression) {
        Ok(evaluation) => {
            log.append("tool_result", evaluation.formatted.clone());
            log.append("execution_status", "calculator:executed".to_owned());
            log.append("calculation:engine", evaluation.engine.slug());
            let body = format!(
                "Execution status: executed.\nTool call: calculator\nInput: `{expression}`\nResult: {}",
                evaluation.formatted
            );
            Some(finalize_simple(
                prompt,
                log,
                "natural_language_api_call",
                "response:natural_language_api_call",
                &body,
                1.0,
            ))
        }
        Err(error) => {
            log.append("tool_result", format!("error={error}"));
            log.append("execution_status", "calculator:error".to_owned());
            let body = format!(
                "Execution status: failed.\nTool call: calculator\nInput: `{expression}`\nError: {error}"
            );
            Some(finalize_simple(
                prompt,
                log,
                "natural_language_api_call_failed",
                "response:natural_language_api_call_failed",
                &body,
                0.4,
            ))
        }
    }
}

fn try_web_search_api_call(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    agent_mode: bool,
) -> Option<SymbolicAnswer> {
    if !is_explicit_tool_api_request(normalized, seed::ROLE_WEB_SEARCH_TOOL_NAME) {
        return None;
    }
    let query = extract_argument(prompt, normalized)?;
    if let Some(denial) = require_tool_permission(log, agent_mode, "web_search") {
        log.append("execution_status", "web_search:refused".to_owned());
        log.append("execution_environment", "agent-permission-gate".to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            "tool_call_refused",
            "response:tool_call_refused",
            &denial,
            1.0,
        ));
    }

    log.append("tool_call", "web_search".to_owned());
    log.append("tool_parameter", format!("query={query}"));
    log.append("web_search:request", query.clone());
    log.append("web_search:query_kind", "direct_api_call".to_owned());
    for provider in WEB_SEARCH_PROVIDERS {
        log.append("web_search:provider", (*provider).to_owned());
    }
    log.append("web_search:combined", format!("rrf:k={WEB_SEARCH_RRF_K}"));
    log.append("tool_result", "search_plan_recorded".to_owned());
    log.append("execution_status", "web_search:executed".to_owned());
    let providers = WEB_SEARCH_PROVIDERS.join(", ");
    let body = format!(
        "Execution status: executed.\nTool call: web_search\nQuery: `{query}`\nResult: search plan recorded with providers {providers}; combined ranking uses reciprocal rank fusion (k = {WEB_SEARCH_RRF_K})."
    );
    Some(finalize_simple(
        prompt,
        log,
        "natural_language_api_call",
        "response:natural_language_api_call",
        &body,
        0.9,
    ))
}

fn try_local_shell_tool_call(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    agent_mode: bool,
) -> Option<SymbolicAnswer> {
    if !is_explicit_local_shell_request(normalized) {
        return None;
    }
    if let Some(denial) = require_tool_permission(log, agent_mode, "local_shell") {
        log.append("execution_status", "local_shell:refused".to_owned());
        log.append("execution_environment", "agent-permission-gate".to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            "tool_call_refused",
            "response:tool_call_refused",
            &denial,
            1.0,
        ));
    }
    log.append("execution_status", "local_shell:refused".to_owned());
    log.append(
        "execution_environment",
        "not-implemented-in-core".to_owned(),
    );
    Some(finalize_simple(
        prompt,
        log,
        "tool_call_refused",
        "response:tool_call_refused",
        "Execution status: refused. local_shell is permissioned, but this Rust core does not provide a shell executor.",
        1.0,
    ))
}

fn require_tool_permission(
    log: &mut EventLog,
    agent_mode: bool,
    tool_name: &str,
) -> Option<String> {
    let capability = format!("tool:{tool_name}");
    if !agent_mode {
        log.append("policy:agent_mode_required_for_tools", capability.clone());
        return Some(format!(
            "Execution status: refused. Natural-language tool calls require explicit agent mode before `{capability}` can run."
        ));
    }
    match default_package_store().permission_for_tool(tool_name) {
        PackagePermissionDecision::Allowed {
            package_id,
            permission_id,
            capability,
        } => {
            log.append(
                "tool_permission",
                format!("allowed:{capability}:{package_id}:{permission_id}"),
            );
            None
        }
        PackagePermissionDecision::Denied { capability, reason } => {
            log.append("policy:package_permission_required", capability.clone());
            Some(format!(
                "Execution status: refused. Tool calls are not allowed for `{capability}`: {reason}. Install or import an associative package that grants this capability before enabling the tool."
            ))
        }
    }
}

fn execute_javascript_subset(program: &str) -> (&'static str, String) {
    let Some(expression) = extract_console_log_expression(program) else {
        return ("executed", String::from("(no output)"));
    };
    match evaluate_calculation(&expression) {
        Ok(evaluation) => ("executed", evaluation.formatted),
        Err(error) => ("failed", error.to_string()),
    }
}

fn extract_console_log_expression(program: &str) -> Option<String> {
    let start = program.find("console.log(")? + "console.log(".len();
    let tail = &program[start..];
    let end = tail.find(')')?;
    Some(tail[..end].trim().to_owned())
}

fn is_explicit_tool_api_request(normalized: &str, tool_name_role: &str) -> bool {
    // The intent is two meanings together: a named tool (the `tool_name_role`
    // meaning — `calculator_tool` or `web_search_tool`) and a tool_invocation_cue
    // ("call", "invoke", "run", "api", "tool"). Both are matched as whole tokens
    // in every supported language through the lexicon, so no tool name or cue
    // word is spelled out in this code.
    let lexicon = seed::lexicon();
    lexicon.mentions_role(tool_name_role, normalized)
        && lexicon.mentions_role(seed::ROLE_TOOL_INVOCATION_CUE, normalized)
}

fn is_explicit_local_shell_request(normalized: &str) -> bool {
    // The local-shell request forms bundle the verb and the tool name into whole
    // phrases (e.g. "local shell tool", "invoke the shell tool"), so the
    // local_shell_request_cue meaning is decisive on its own — unlike the
    // calculator/web-search tools, no separate tool_invocation_cue is required.
    seed::lexicon().mentions_role(seed::ROLE_LOCAL_SHELL_REQUEST_CUE, normalized)
}

fn extract_argument(prompt: &str, normalized: &str) -> Option<String> {
    extract_backticked(prompt)
        .or_else(|| extract_quoted_phrase(prompt))
        .or_else(|| after_argument_marker(normalized))
        .map(|value| clean_argument(&value))
        .filter(|value| !value.is_empty())
}

fn after_argument_marker(normalized: &str) -> Option<String> {
    // When the argument is not already delimited by backticks or quotes, the
    // phrases that introduce it ("with query", "query", "with", "for") are
    // tool_argument_marker word forms. We reconstruct each English form as a
    // space-padded token, find the first one present in the lexicon's
    // declaration (priority) order — longer, more specific phrases first — and
    // take the text after it. Only the English forms drive this English-frame
    // heuristic; the other languages stay in the seed for self-description.
    for marker in
        seed::lexicon().words_for_role_in_languages(seed::ROLE_TOOL_ARGUMENT_MARKER, &["en"])
    {
        let delimiter = format!(" {marker} ");
        if let Some(index) = normalized.find(&delimiter) {
            return Some(normalized[index + delimiter.len()..].to_owned());
        }
    }
    None
}

fn clean_argument(value: &str) -> String {
    value
        .trim()
        .trim_matches(|c: char| {
            matches!(
                c,
                '`' | '"' | '\'' | '.' | ',' | ':' | ';' | '!' | '?' | '(' | ')' | '[' | ']'
            )
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
