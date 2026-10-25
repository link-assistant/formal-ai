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
    work_hygiene_with_policy(paths, None)
}

fn work_hygiene_with_policy(
    paths: &[String],
    session_policy: Option<&serde_json::Value>,
) -> Option<PreparedWorkHygiene> {
    let root = hygiene_policy().children.first()?;
    let values = |name: &str| {
        root.children
            .iter()
            .filter(|node| node.name == name)
            .map(|node| node.id.clone())
            .collect::<Vec<_>>()
    };
    let flags = values("output-flag");
    let outputs = hygiene_outputs(&flags);
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
        "empty-evidence-suffixes":suffixes,"session-policy":session_policy,"session-contract":session_contract()?});
    let source_io = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/meta/workspace-source-io.lino"
    ));
    let mut source_lines = source_io
        .children
        .first()?
        .children
        .iter()
        .filter(|node| node.name == "source-line")
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    source_lines.extend(values("source-line"));
    let source = source_lines.join("\n") + "\n";
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
    let direct = serde_json::from_str::<serde_json::Value>(output).ok();
    let direct_feedback = direct.as_ref().is_some_and(|value| {
        value
            .get("comments")
            .is_some_and(serde_json::Value::is_array)
            && value
                .get("reviews")
                .is_some_and(serde_json::Value::is_array)
    });
    let exit = super::tool_result::reported_exit_code(output);
    if super::tool_result::harness_reported_failure(output)
        || super::tool_result::incomplete_receipt(output)
        || exit.is_some_and(|exit| exit != 0)
        || (!direct_feedback && exit != Some(0))
    {
        return true;
    }
    let text = if direct_feedback {
        output.to_owned()
    } else {
        super::tool_result::observed_payload(output).unwrap_or_else(|| output.to_owned())
    };
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

const COMMAND_PLACEHOLDER: &str = "{command}";
const OUTPUT_PLACEHOLDER: &str = "{output}";

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
                &[
                    (COMMAND_PLACEHOLDER, status_command),
                    (OUTPUT_PLACEHOLDER, output),
                ],
            )));
        }
        return Some(plan_one(
            run,
            json!({"command": status_command}).to_string(),
        ));
    };
    let current = prepared_work_paths(status_output);
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
    let commit = prepared_work_commit_command(&hygiene, reference)?;
    for command in [hygiene.command, commit, hygiene.clean_command] {
        if progress.successful_run_count_for(&command) > 0 {
            continue;
        }
        if let Some(output) = progress.latest_run_output_for(&command) {
            return Some(AgenticPlan::Final(super::work_item_steps::fill(
                "prepared_work_stopped_report",
                &[
                    (COMMAND_PLACEHOLDER, &command),
                    (OUTPUT_PLACEHOLDER, output),
                ],
            )));
        }
        return Some(plan_one(run, json!({"command": command}).to_string()));
    }
    // Repair of a working tree never certifies the issue, workflow, or output.
    None
}

fn hygiene_outputs(flags: &[String]) -> Vec<serde_json::Value> {
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
    outputs
}

/// Normal owned-only commit with staged-subset validation and physical branch push.
#[must_use]
pub(super) fn prepared_work_commit_command(
    hygiene: &PreparedWorkHygiene,
    reference: &str,
) -> Option<String> {
    let policy = hygiene_policy().children.first()?;
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
    Some(crate::seed::fill_template_once(
        policy.find_child_value("commit-command"),
        &[
            ("guard", &hygiene.index_guard_command),
            ("files", &files),
            ("subject", &subject),
            ("body", &body),
            ("branch", &branch),
        ],
    ))
}

fn session_contract() -> Option<serde_json::Value> {
    let installed = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/meta/prepared-artifact-inventory.lino"
    ));
    let root = installed.children.first()?;
    Some(
        json!({"rootPattern":root.find_child_value("evidence-pattern"),
        "pattern":root.find_child_value("session-pattern"),
        "maximumBytes":root.find_child_value("session-limit").parse::<usize>().ok()?,
        "filename":"session-id.txt"}),
    )
}

/// Only an independently command-qualified inventory may extend the current recipe scope.
#[must_use]
pub(super) fn prepared_recipe_hygiene_with_inventory(
    source_paths: &[String],
    changed: &[String],
    inventory: Option<&super::recipe_artifact_inventory::RecipeArtifactInventory>,
) -> Option<PreparedWorkHygiene> {
    let policy = hygiene_policy().children.first()?;
    let flags = policy
        .children
        .iter()
        .filter(|node| node.name == "output-flag")
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let outputs = hygiene_outputs(&flags)
        .into_iter()
        .filter(|entry| {
            entry
                .get("source")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|source| source_paths.iter().any(|path| path == source))
        })
        .collect::<Vec<_>>();
    let output_owned = |path: &str| {
        outputs
            .iter()
            .any(|entry| entry.get("output").and_then(serde_json::Value::as_str) == Some(path))
    };
    let mut combined = changed.to_vec();
    let mut session_policy = None;
    let mut evidence_members = Vec::new();
    if let Some(inventory) = inventory {
        let contract = session_contract()?;
        let root_pattern = contract.get("rootPattern")?.as_str()?;
        if inventory.source_paths.as_slice() != source_paths
            || inventory.owned_paths.len() > 256
            || regex::Regex::new(root_pattern)
                .ok()?
                .find(&inventory.evidence_root)?
                .as_str()
                != inventory.evidence_root
        {
            return None;
        }
        let expected = json!({"evidenceRoot":inventory.evidence_root,"filename":"session-id.txt",
            "path":format!("{}session-id.txt",inventory.evidence_root),"pattern":contract.get("pattern")?,
            "maximumBytes":contract.get("maximumBytes")?});
        if inventory.session_policy != expected {
            return None;
        }
        let installed = crate::seed::parser::parse_lino(include_str!(
            "../../embedded/data/meta/prepared-artifact-inventory.lino"
        ));
        evidence_members = installed
            .children
            .first()?
            .children
            .iter()
            .filter(|node| node.name == "stream")
            .map(|node| format!("{}{}", inventory.evidence_root, node.id))
            .collect();
        evidence_members.push(format!("{}session-id.txt", inventory.evidence_root));
        let mut observed = std::collections::BTreeSet::new();
        for path in &inventory.owned_paths {
            if !observed.insert(path) || (!output_owned(path) && !evidence_members.contains(path)) {
                return None;
            }
            if !combined.contains(path) {
                combined.push(path.clone());
            }
        }
        session_policy = Some(expected);
    }
    let known = changed.iter().all(|path| {
        source_paths.contains(path)
            || output_owned(path)
            || if inventory.is_some() {
                evidence_members.contains(path)
            } else {
                path.starts_with(policy.find_child_value("evidence-root"))
                    && policy
                        .children
                        .iter()
                        .filter(|node| node.name == "empty-evidence-suffix")
                        .any(|node| path.ends_with(node.id.as_str()))
            }
    });
    known
        .then(|| work_hygiene_with_policy(&combined, session_policy.as_ref()))
        .flatten()
}

/// Decode a freshly command-bound status payload without trimming its path bytes.
#[must_use]
pub(super) fn prepared_work_paths(output: &str) -> Option<Vec<String>> {
    if super::tool_result::incomplete_receipt(output) {
        return None;
    }
    porcelain_paths(output)
        .or_else(|| porcelain_paths(&super::tool_result::observed_payload(output)?))
}
