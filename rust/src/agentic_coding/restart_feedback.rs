//! Resume Hive Mind's existing working tree instead of reimplementing its issue.
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use crate::protocol::ChatMessage;
use serde_json::json;

/// The bounded filesystem preparation and exact commit operands for prepared work.
#[derive(Debug, Clone)]
pub struct PreparedWorkHygiene {
    pub command: String,
    pub commit_files: Vec<String>,
    pub clean_command: String,
    pub generated_index_removals: Vec<String>,
    pub index_guard_command: String,
}

fn hygiene_policy() -> &'static crate::seed::parser::LinoNode {
    static POLICY: std::sync::OnceLock<crate::seed::parser::LinoNode> = std::sync::OnceLock::new();
    POLICY.get_or_init(|| {
        crate::seed::parser::parse_lino(include_str!(
            "../../embedded/data/meta/restart-hygiene.lino"
        ))
    })
}

/// Derive generated operands from installed compiler recipes, preserving physical bytes.
///
/// Unknown binary paths remain blocked by the source-owned validation command;
/// this does not grant authority to remove arbitrary files or execute a compiler.
#[must_use]
pub fn prepared_work_hygiene(paths: &[String]) -> Option<PreparedWorkHygiene> {
    let root = hygiene_policy().children.first()?;
    let values = |name: &str| {
        root.children
            .iter()
            .filter(|node| node.name == name)
            .map(|node| node.id.clone())
            .collect::<Vec<_>>()
    };
    let flags = values("output-flag");
    let mut outputs = Vec::new();
    let portable = |value: &str| {
        !value.is_empty()
            && !value.starts_with('/')
            && !value.split('/').any(|part| part == "..")
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_./-".contains(&byte))
    };
    for language in crate::coding::program_languages() {
        let source = language.save_as.as_ref();
        let words = language
            .execution
            .check_command
            .as_deref()
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>();
        if !portable(source) || !words.contains(&source) {
            continue;
        }
        for pair in words.windows(2) {
            if flags.iter().any(|flag| flag == pair[0]) && portable(pair[1]) && pair[1] != source {
                let entry = json!({"source": source, "output": pair[1]});
                if !outputs.contains(&entry) {
                    outputs.push(entry);
                }
            }
        }
    }
    let evidence_root = root.find_child_value("evidence-root");
    let suffixes = values("empty-evidence-suffix");
    let ignored = paths
        .iter()
        .filter(|path| {
            outputs.iter().any(|entry| {
                entry.get("output").and_then(serde_json::Value::as_str) == Some(path.as_str())
            }) || (path.starts_with(evidence_root)
                && suffixes
                    .iter()
                    .any(|suffix| path.ends_with(suffix.as_str())))
        })
        .cloned()
        .collect::<Vec<_>>();
    let plan = json!({"files":paths, "outputs":outputs, "ignored":ignored,
        "binary-prefixes":values("binary-prefix"), "evidence-root":evidence_root,
        "empty-evidence-suffixes":suffixes});
    let source = values("source-line").join("\n") + "\n";
    let quote = super::git_commit::shell_quote;
    let excluded = paths
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(".gitignore"))
        .map(|path| quote(&format!(":(exclude,literal){path}")))
        .collect::<Vec<_>>()
        .join(" ");
    let index_guard_command = crate::seed::fill_template_once(
        root.find_child_value("index-guard-command"),
        &[("exclude", &excluded)],
    );
    let quoted_source = quote(&source);
    let quoted_plan = quote(&plan.to_string());
    let command = crate::seed::fill_template_once(
        root.find_child_value("command"),
        &[("source", &quoted_source), ("plan", &quoted_plan)],
    );
    let mut command = format!("{index_guard_command} && {command}");
    if !ignored.is_empty() {
        command.push_str(" && ");
        command.push_str(
            &root.find_child_value("untrack-command").replace(
                "{files}",
                &ignored
                    .iter()
                    .map(|value| quote(&format!(":(literal){value}")))
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
        );
    }
    let mut commit_files = paths
        .iter()
        .filter(|path| !ignored.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    if !ignored.is_empty() && !commit_files.iter().any(|path| path == ".gitignore") {
        commit_files.push(".gitignore".to_owned());
    }
    Some(PreparedWorkHygiene {
        command,
        commit_files,
        generated_index_removals: ignored,
        index_guard_command,
        clean_command: root.find_child_value("clean-command").to_owned(),
    })
}

fn porcelain_paths(status: &str) -> Option<Vec<String>> {
    if status.len() > 65536 {
        return None;
    }
    let mut paths = Vec::new();
    for line in status.lines().filter(|line| !line.trim().is_empty()) {
        if line.len() < 4 || line.as_bytes()[2] != b' ' || !line.is_char_boundary(3) {
            return None;
        }
        let status = &line[..2];
        if !status.bytes().all(|value| b" MADRCU?!".contains(&value)) {
            return None;
        }
        let path = &line[3..];
        // Quoted and rename records need a real porcelain parser; never guess.
        if path.is_empty()
            || path.starts_with('"')
            || path.contains(" -> ")
            || path.contains('\0')
            || std::path::Path::new(path).is_absolute()
            || std::path::Path::new(path)
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return None;
        }
        if !paths.iter().any(|existing| existing == path) {
            paths.push(path.to_owned());
            if paths.len() > 256 {
                return None;
            }
        }
    }
    Some(paths)
}

/// Only the explicitly marked fenced porcelain listing is authoritative.
#[must_use]
pub fn changed_paths(task: &str) -> Option<Vec<String>> {
    let (_, rest) = task.split_once("UNCOMMITTED CHANGES DETECTED")?;
    let (_, fenced) = rest.split_once("```")?;
    let (_, body) = fenced.split_once('\n')?;
    let (body, _) = body.split_once("```")?;
    let paths = porcelain_paths(body)?;
    (!paths.is_empty()).then_some(paths)
}

/// Whether a pull request's comment/review feedback still asks for changes;
/// unreadable feedback counts as asking, so readiness is never assumed.
#[must_use]
pub fn feedback_needs_changes(output: &str) -> bool {
    let text =
        super::tool_result::shell_step(output).map_or_else(|| output.to_owned(), |step| step.text);
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return true;
    };
    if !value
        .get("comments")
        .is_some_and(serde_json::Value::is_array)
        || !value
            .get("reviews")
            .is_some_and(serde_json::Value::is_array)
    {
        return true;
    }
    value
        .get("comments")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| {
            items.iter().any(|item| {
                item.get("body")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|body| !body.trim().is_empty())
            })
        })
        || value
            .get("reviews")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("state").and_then(serde_json::Value::as_str)
                        == Some("CHANGES_REQUESTED")
                })
            })
}

pub(super) fn plan_restart(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
) -> Option<AgenticPlan> {
    let listed = changed_paths(task)?;
    let reference = task
        .split_whitespace()
        .find(|word| word.starts_with("https://github.com/") && word.contains("/pull/"))?
        .trim_matches(|character| matches!(character, '`' | ')' | ',' | '.'));
    let progress = Progress::scan(messages);
    let run = tool_for(tools, Capability::Run)?;
    let policy = hygiene_policy().children.first()?;
    let status_command = policy.find_child_value("status-command");
    let Some(status_output) = progress.latest_successful_run_output_for(status_command) else {
        if let Some(output) = progress.latest_run_output_for(status_command) {
            return Some(AgenticPlan::Final(super::work_item_steps::fill(
                "prepared_work_stopped_report",
                &[("{command}", status_command), ("{output}", output)],
            )));
        }
        return Some(plan_one(
            run,
            json!({"command": status_command}).to_string(),
        ));
    };
    let status = super::tool_result::observed_payload(status_output)
        .unwrap_or_else(|| status_output.to_owned());
    let current = porcelain_paths(&status);
    let Some(current) = current.filter(|paths| {
        paths.iter().all(|path| {
            listed.iter().any(|scope| {
                path == scope || (scope.ends_with('/') && path.starts_with(scope.as_str()))
            })
        })
    }) else {
        return Some(AgenticPlan::Final(
            policy.find_child_value("scope-refusal").to_owned(),
        ));
    };
    if current.is_empty() {
        return None;
    }
    let hygiene = prepared_work_hygiene(&current)?;
    let quote = super::git_commit::shell_quote;
    let files = hygiene
        .commit_files
        .iter()
        .map(|path| quote(&format!(":(literal){path}")))
        .collect::<Vec<_>>()
        .join(" ");
    let subject = quote(&crate::seed::agentic_message(
        "restart_feedback_commit_subject",
        &[],
    ));
    let body = quote(&super::git_commit::resolves_body(reference));
    let branch = quote("HEAD");
    let commit = crate::seed::fill_template_once(
        policy.find_child_value("commit-command"),
        &[
            ("guard", &hygiene.index_guard_command),
            ("files", &files),
            ("subject", &subject),
            ("body", &body),
            ("branch", &branch),
        ],
    );
    for command in [hygiene.command, commit, hygiene.clean_command] {
        if progress.successful_run_count_for(&command) > 0 {
            continue;
        }
        if let Some(output) = progress.latest_run_output_for(&command) {
            return Some(AgenticPlan::Final(super::work_item_steps::fill(
                "prepared_work_stopped_report",
                &[("{command}", &command), ("{output}", output)],
            )));
        }
        return Some(plan_one(run, json!({"command": command}).to_string()));
    }
    // Repair of a working tree never certifies the issue, workflow, or output.
    None
}
