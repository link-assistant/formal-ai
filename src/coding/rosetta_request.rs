//! Sourced example and bounded-execution requests backed by Rosetta Code.

use crate::agent::{AgentRunStatus, AgentWorkspace, AgentWorkspaceConfig};
use crate::coding::function_catalog::rosetta_code::{
    RosettaExample, fetch_example, page_title_for_task, page_title_from_url,
};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};

use crate::solver_handlers::finalize_simple;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RequestKind {
    Example,
    Execute,
}

pub fn try_rosetta_code_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    let cache_dir = std::env::var("FORMAL_AI_SOURCE_CACHE_DIR")
        .or_else(|_| std::env::var("FORMAL_AI_CACHE_DIR"))
        .unwrap_or_else(|_| String::from("data"));
    let client = CachedSourceClient::new(cache_dir, CurlSourceTransport).with_online(!offline);
    try_rosetta_code_request_with_client(prompt, normalized, log, &client)
}

pub fn try_rosetta_code_request_with_client<T: SourceTransport>(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    let explicit_page = page_title_from_url(prompt);
    let kind = request_kind(normalized, explicit_page.is_some())?;
    let language = crate::coding::program_language_by_alias(normalized)?;
    let page_title = explicit_page.or_else(|| {
        crate::coding::program_task_by_alias(normalized).map(|task| page_title_for_task(task.slug))
    })?;
    let example = match fetch_example(client, &page_title, language.slug) {
        Ok(example) => example,
        Err(error) => {
            log.append("synthesis:source_miss", error.to_string());
            return None;
        }
    };
    log.append(
        "source:http",
        format!(
            "url={};fetched_at={};sha256={};cached={}",
            example.source_url, example.fetched_at, example.sha256, example.cached
        ),
    );
    if example.cached {
        log.append("cache_hit", example.source_url.clone());
    }
    let response_language = crate::language::detect(prompt).slug();
    let (intent, execution) = match kind {
        RequestKind::Example => (
            "coding_example",
            response("coding_rosetta_not_requested", response_language, &[]),
        ),
        RequestKind::Execute => (
            "execute_coding_example",
            execute_example(prompt, response_language, &example, log),
        ),
    };
    let body = render_example(response_language, &example, &execution);
    Some(finalize_simple(
        prompt,
        log,
        intent,
        "response:coding_example:rosetta_code",
        &body,
        1.0,
    ))
}

fn request_kind(normalized: &str, has_rosetta_url: bool) -> Option<RequestKind> {
    let lexicon = crate::seed::lexicon();
    if has_rosetta_url && lexicon.mentions_role(crate::seed::ROLE_EXECUTE_URL_REQUEST, normalized) {
        return Some(RequestKind::Execute);
    }
    lexicon
        .mentions_role(crate::seed::ROLE_EXAMPLE_REQUEST, normalized)
        .then_some(RequestKind::Example)
}

fn execute_example(
    prompt: &str,
    response_language: &str,
    example: &RosettaExample,
    log: &mut EventLog,
) -> String {
    if example.language != "rust" {
        return response(
            "coding_rosetta_rust_only",
            response_language,
            &[("language", example.language.as_str())],
        );
    }
    let Ok(mut workspace) = AgentWorkspace::for_prompt(prompt, &AgentWorkspaceConfig::default())
    else {
        return response(
            "coding_rosetta_workspace_unavailable",
            response_language,
            &[],
        );
    };
    workspace.create_file("main.rs", &example.code);
    let compile_command = if cfg!(windows) {
        "rustc main.rs -o main.exe"
    } else {
        "rustc main.rs -C linker=/usr/bin/cc -o main"
    };
    workspace.run_command(compile_command);
    let compiled = workspace
        .last_command_result()
        .is_some_and(|result| result.status_code == Some(0) && !result.timed_out);
    if compiled {
        workspace.run_command(if cfg!(windows) { "main.exe" } else { "./main" });
    }
    let run = workspace.finish();
    for action in &run.actions {
        log.append(action.event_kind(), action.evidence_payload());
    }
    if run.status == AgentRunStatus::Completed && compiled {
        log.append("execution_status", "tests passed".to_owned());
        response("coding_rosetta_passed", response_language, &[])
    } else {
        log.append("execution_status", "execution unavailable".to_owned());
        response("coding_rosetta_failed", response_language, &[])
    }
}

fn render_example(response_language: &str, example: &RosettaExample, execution: &str) -> String {
    let capture_intent = if example.cached {
        "coding_rosetta_capture_cached"
    } else {
        "coding_rosetta_capture_fetched"
    };
    let capture = response(
        capture_intent,
        response_language,
        &[("sha256", example.sha256.as_str())],
    );
    response(
        "coding_rosetta_answer",
        response_language,
        &[
            ("language", example.language.as_str()),
            ("task", example.task.as_str()),
            ("code", example.code.as_str()),
            ("source", example.source_url.as_str()),
            ("license", example.license.as_str()),
            ("execution", execution),
            ("capture", capture.as_str()),
        ],
    )
}

fn response(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    crate::seed::render_response(intent, language, values)
        .or_else(|| crate::seed::render_response(intent, "en", values))
        .unwrap_or_default()
}
