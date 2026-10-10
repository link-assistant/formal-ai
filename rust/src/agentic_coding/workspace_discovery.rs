//! Source-owned workaround for carried upstream discovery functions.
//! Shared seed owns grammar; observations remain gaps and do not approve effects.

use super::capability_router::tool_for;
use super::planner::{AgenticPlan, Capability, plan_one};
use super::progress::Progress;
use super::tool_result::{
    harness_reported_failure, incomplete_receipt, observed_bytes_match, observed_payload,
    reported_exit_code,
};
use crate::engine::stable_id;
use serde_json::{Value, json};

/// Shared immutable source templates carry wording only; receipt qualification stays at each caller.
pub(super) fn contract_text(key: &str, arguments: &[&str]) -> String {
    let seed = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/seed/workspace-discovery-contracts.lino"
    ));
    let roots: Vec<_> = seed
        .children
        .iter()
        .filter(|root| root.name == "workspace-discovery-contracts")
        .collect();
    assert_eq!(roots.len(), 1, "MissingSourceTemplateRoot");
    let root = roots[0];
    let entries: Vec<_> = root
        .children
        .iter()
        .filter(|entry| entry.name == "template" && entry.id == key)
        .collect();
    assert_eq!(entries.len(), 1, "AmbiguousSourceTemplate");
    let texts: Vec<_> = entries[0]
        .children
        .iter()
        .filter(|field| field.name == "text")
        .collect();
    assert_eq!(texts.len(), 1, "AmbiguousSourceTemplateText");
    let pieces: Vec<_> = texts[0].id.split("{}").collect();
    assert_eq!(
        pieces.len(),
        arguments.len() + 1,
        "DifferentSourceTemplateArity"
    );
    let mut output = pieces[0].to_owned();
    for (argument, piece) in arguments.iter().zip(pieces.iter().skip(1)) {
        output.push_str(argument);
        output.push_str(piece);
    }
    output
}

const MAXIMUM_FILES: usize = 128;
const MAXIMUM_BYTES: usize = 65536;

pub(super) struct DiscoveryContract {
    pub(super) source: String,
    pub(super) subject: String,
    /// UTF-16 coordinates match the maintained JavaScript source contract.
    pub(super) subject_span: [usize; 2],
    pub(super) remaining_span: [usize; 2],
    words: Vec<String>,
}

fn utf16_boundary(source: &str, byte: usize) -> Option<usize> {
    source
        .get(..byte)
        .map(|prefix| prefix.encode_utf16().count())
}

pub(super) fn workspace_discovery_contract(task: &str) -> Option<DiscoveryContract> {
    let parsed = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/seed/workspace-discovery-grammar.lino"
    ));
    let grammar = parsed.children.first()?;
    if grammar.name != "workspace-discovery-grammar" {
        return None;
    }
    let mut actions: Vec<String> = crate::seed::lexicon()
        .role_word_forms("workspace_inspection_action")
        .into_iter()
        .filter(|form| form.slot() == crate::seed::Slot::Bare)
        .map(|form| regex::escape(&form.text))
        .collect();
    actions.sort_by_key(|action| std::cmp::Reverse(action.len()));
    let field = |name| grammar.children.iter().find(|node| node.name == name);
    let template = &field("pattern")?.id;
    let subject = &field("subject-pattern")?.id;
    if actions.is_empty()
        || template.matches("{action}").count() != 1
        || template.matches("{subject}").count() != 1
    {
        return None;
    }
    let pattern = template
        .replace("{action}", &format!("({})", actions.join("|")))
        .replace("{subject}", &format!("({subject})"));
    let expression = regex::RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()
        .ok()?;
    let captures = expression.captures(task)?;
    if captures.get(0)?.start() != 0 {
        return None;
    }
    let subject = captures.get(2)?;
    if !subject.as_str().is_ascii() {
        return None;
    }
    let words: Vec<String> = subject
        .as_str()
        .to_ascii_lowercase()
        .split([' ', '_', '-'])
        .map(str::to_owned)
        .collect();
    if words.is_empty() || words.iter().any(|word| word.len() < 2) {
        return None;
    }
    let end = utf16_boundary(task, subject.end())?;
    Some(DiscoveryContract {
        source: task.to_owned(),
        subject: subject.as_str().to_owned(),
        subject_span: [utf16_boundary(task, subject.start())?, end],
        remaining_span: [end, task.encode_utf16().count()],
        words,
    })
}

pub(super) fn workspace_discovery_command(need: &str) -> String {
    contract_text(
        "source-template-1",
        &[&(stable_id("workspace_discovery_need", need)).to_string()],
    )
}

fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_./-".contains(&byte))
        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}

fn candidate_read_command(need: &str, path: &str) -> Option<String> {
    safe_path(path).then(|| {
        contract_text(
            "source-template-2",
            &[
                &(stable_id("workspace_candidate_need", need)).to_string(),
                &(path).to_string(),
            ],
        )
    })
}

pub(super) enum DiscoveryStep {
    Calls(AgenticPlan),
    Observation { kind: &'static str, answer: String },
}

fn observation(kind: &'static str, answer: impl Into<String>) -> DiscoveryStep {
    DiscoveryStep::Observation {
        kind,
        answer: answer.into(),
    }
}

fn incomplete(raw: &str) -> bool {
    if incomplete_receipt(raw) {
        return true;
    }
    serde_json::from_str::<Value>(raw)
        .ok()
        .is_some_and(|value| {
            value
                .get("command_output_complete")
                .and_then(Value::as_bool)
                == Some(false)
                || value
                    .pointer("/stdout_capture/complete")
                    .and_then(Value::as_bool)
                    == Some(false)
        })
}

fn qualified_payload(
    attempt: &super::qualified_tool_observation::QualifiedToolAttempt,
) -> Option<String> {
    if !attempt.succeeded
        || incomplete(&attempt.detail)
        || harness_reported_failure(&attempt.detail)
        || reported_exit_code(&attempt.detail) != Some(0)
    {
        return None;
    }
    let payload = observed_payload(&attempt.detail)?;
    observed_bytes_match(&attempt.detail, &payload).then_some(payload)
}

pub(super) fn workspace_discovery_step(
    task: &str,
    progress: &Progress,
    tools: &[&str],
    need: &str,
) -> Option<DiscoveryStep> {
    let contract = workspace_discovery_contract(task)?;
    let command = workspace_discovery_command(need);
    let arguments = json!({"command": command});
    let Some(run) = progress.latest_attempt_for(Capability::Run, &arguments) else {
        let listing = json!({"path":"."});
        if let Some(tool) = tool_for(tools, Capability::ListDir)
            && progress
                .latest_attempt_for(Capability::ListDir, &listing)
                .is_none()
        {
            return Some(DiscoveryStep::Calls(plan_one(tool, listing.to_string())));
        }
        return Some(match tool_for(tools, Capability::Run) {
            Some(tool) => DiscoveryStep::Calls(plan_one(tool, arguments.to_string())),
            None => observation("unavailable", contract_text("source-template-8", &[])),
        });
    };
    let Some(payload) = qualified_payload(run) else {
        return Some(observation(
            "failed",
            contract_text("source-template-9", &[]),
        ));
    };
    if payload.len() > MAXIMUM_BYTES {
        return Some(observation(
            "unqualified",
            contract_text("source-template-10", &[]),
        ));
    }
    let mut lines: Vec<&str> = payload.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    if lines.first() != Some(&"workspace-discovery-v1")
        || lines.last() != Some(&"workspace-discovery-end")
        || lines.len() < 2
    {
        return Some(observation(
            "unqualified",
            contract_text("source-template-11", &[]),
        ));
    }
    let lines = &lines[1..lines.len() - 1];
    if lines.len() > MAXIMUM_FILES
        || lines
            .iter()
            .any(|path| !path.strip_prefix("./").is_some_and(safe_path))
    {
        return Some(observation(
            "unqualified",
            contract_text("source-template-12", &[]),
        ));
    }
    let mut unique = Vec::new();
    for path in lines {
        if !unique.contains(path) {
            unique.push(*path);
        }
    }
    let candidates: Vec<&str> = unique
        .iter()
        .copied()
        .filter(|path| {
            let terms: Vec<String> = path
                .split(['/', '_', '.', '-'])
                .map(str::to_ascii_lowercase)
                .collect();
            contract.words.iter().all(|word| terms.contains(word))
        })
        .collect();
    if candidates.is_empty() {
        return Some(observation(
            "no_candidate",
            contract_text(
                "source-template-3",
                &[
                    &(unique.len()).to_string(),
                    &(serde_json::to_string(&contract.words).ok()?).to_string(),
                ],
            ),
        ));
    }
    if candidates.len() != 1 {
        return Some(observation(
            "ambiguous",
            contract_text(
                "source-template-4",
                &[
                    &(candidates.len()).to_string(),
                    &(candidates.join(", ")).to_string(),
                ],
            ),
        ));
    }
    let path = candidates[0].strip_prefix("./")?;
    let arguments = json!({"command":candidate_read_command(need,path)?});
    let Some(read) = progress.latest_attempt_for(Capability::Run, &arguments) else {
        return Some(match tool_for(tools, Capability::Run) {
            Some(tool) => DiscoveryStep::Calls(plan_one(tool, arguments.to_string())),
            None => observation(
                "unavailable",
                contract_text("source-template-5", &[&(path).to_string()]),
            ),
        });
    };
    let source = qualified_payload(read).and_then(|payload| {
        payload
            .strip_prefix("workspace-source-v1\n")?
            .strip_suffix("\nworkspace-source-end\n")
            .map(str::to_owned)
    });
    let Some(source) = source else {
        return Some(observation(
            "unqualified_read",
            contract_text("source-template-6", &[&(path).to_string()]),
        ));
    };
    if source.len() > MAXIMUM_BYTES {
        return Some(observation(
            "unqualified_read",
            contract_text("source-template-13", &[]),
        ));
    }
    Some(observation(
        "candidate_read",
        contract_text(
            "source-template-7",
            &[
                &(path).to_string(),
                &(source.len()).to_string(),
                &(stable_id("observed_candidate", &source)).to_string(),
            ],
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_seed_retains_original_subject_and_unconsumed_unicode_tail() {
        for source in [
            contract_text("original-request-1", &[]),
            contract_text("original-request-2", &[]),
            contract_text("original-request-3", &[]),
            contract_text("original-request-4", &[]),
            contract_text("original-request-5", &[]),
        ] {
            let contract = workspace_discovery_contract(&source).expect("shared grammar");
            assert_eq!(contract.source, source);
            assert_eq!(contract.subject, "task model");
            assert_eq!(contract.subject_span, [12, 22]);
            assert_eq!(contract.remaining_span, [22, source.encode_utf16().count()]);
        }
        assert!(utf16_boundary("😀", 1).is_none());
        assert_eq!(utf16_boundary("😀", 4), Some(2));
        assert!(workspace_discovery_contract("Inspect ../task_model").is_none());
        assert!(workspace_discovery_contract("node_path=1.1.1.1.1").is_none());
    }
    #[test]
    fn partial_receipts_and_unsafe_candidate_paths_cannot_become_source_evidence() {
        for raw in [
            r#"{"command_output_complete":false}"#,
            r#"{"stdout_capture":{"complete":false}}"#,
            r#"{"truncated":true}"#,
        ] {
            assert!(incomplete(raw));
        }
        for path in [
            "../task_model",
            "task model",
            "a//task_model",
            "./task_model",
            "task_model;true",
        ] {
            assert!(!safe_path(path));
        }
    }
}
