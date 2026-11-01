//! Manual native projection of the bounded source-documentation controller.
//! Source only: no native compilation or verified translation credit claimed.
use super::final_result::FinalResult;
use super::observed_documentation::plan_observed_documentation;
use super::planner::AgenticPlan;
use super::shell_command_policy::sentences;
use super::source_manifest_facts::observed_source_json;
use crate::normal_markov::{quote_fault, quoted_segment_spans};
use crate::protocol::ChatMessage;
use crate::seed::parser::parse_lino;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};

const GRAMMAR: &str = include_str!("../../embedded/data/meta/source-documentation.lino");

pub(super) use super::manifest_documentation::derive_manifest_documentation;

pub(super) fn source_documentation_policy() -> Option<Value> {
    let parsed = parse_lino(GRAMMAR);
    let root = parsed
        .children
        .iter()
        .find(|node| node.name == "source-documentation")?;
    let values = |name: &str| {
        root.children
            .iter()
            .filter(|node| node.name == name)
            .map(|node| node.id.clone())
            .collect::<Vec<_>>()
    };
    let mut policy = json!({"actions":values("action"),"preserve":values("preserve"),"negations":values("negation"),"prefixes":values("prefix"),"template":values("template").first()?,"dependencyCues":values("dependency-cue"),"versionCues":values("version-cue"),"sectionCues":values("section-cue")});
    policy["manifestStart"] = values("manifest-start")
        .first()
        .map_or(Value::Null, |value| json!(value));
    policy["manifestEnd"] = values("manifest-end")
        .first()
        .map_or(Value::Null, |value| json!(value));
    policy["manifestTitle"] = values("manifest-title")
        .first()
        .map_or(Value::Null, |value| json!(value));
    policy["manifestNotice"] = values("manifest-notice")
        .first()
        .map_or(Value::Null, |value| json!(value));
    policy["manifestHeader"] = values("manifest-header")
        .first()
        .map_or(Value::Null, |value| json!(value));
    policy["manifestSeparator"] = values("manifest-separator")
        .first()
        .map_or(Value::Null, |value| json!(value));
    policy["manifestRow"] = values("manifest-row")
        .first()
        .map_or(Value::Null, |value| json!(value));
    policy["manifestAbsentVersion"] = values("manifest-absent-version")
        .first()
        .map_or(Value::Null, |value| json!(value));
    policy["manifestUnqualifiedTarget"] = values("manifest-unqualified-target")
        .first()
        .map_or(Value::Null, |value| json!(value));
    policy["sourceUsage"] = json!({"heading":values("source-usage-heading").first().cloned(),"call":values("source-usage-call").first().cloned(),"reference":values("source-usage-reference").first().cloned(),"codeOpen":values("source-usage-code-open").first().cloned(),"codeClose":values("source-usage-code-close").first().cloned()});
    Some(policy)
}

pub(super) fn concept_words(text: &str) -> Vec<String> {
    let camel = regex::Regex::new(r"([a-z0-9])([A-Z])").expect("StaticConceptBoundary");
    let words = regex::Regex::new(r"[^\p{L}\p{N}]+").expect("StaticConceptWords");
    let normalized = camel.replace_all(text, "$1 $2").to_lowercase();
    words
        .split(&normalized)
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

pub(super) fn documentation_authority(
    task: &str,
    guide_path: &str,
    actions: &[String],
    preserve: &[String],
    negations: &[String],
    prefixes: &[String],
    require_target_mention: bool,
    require_preservation: bool,
) -> Option<&'static str> {
    if quote_fault(task).is_some() {
        return Some("DocumentationActionUnowned");
    }
    let mut masked = task.as_bytes().to_vec();
    for span in quoted_segment_spans(task) {
        for byte in &mut masked[span.start..span.end] {
            if !matches!(*byte, b'\r' | b'\n') {
                *byte = b' ';
            }
        }
    }
    let masked = String::from_utf8(masked).expect("MaskedQuoteUtf8");
    if require_preservation
        && !preserve
            .iter()
            .any(|cue| concept_words(&masked).contains(cue))
    {
        return None;
    }
    let mut mentioned = false;
    for clause in sentences(&masked) {
        if require_target_mention && !task[clause.span.clone()].contains(guide_path) {
            continue;
        }
        let words = concept_words(clause.text);
        for action in actions {
            let Some(index) = words.iter().position(|word| word == action) else {
                continue;
            };
            mentioned = true;
            let before = words[..index].join(" ");
            if negations.iter().any(|negation| {
                format!(" {before} ").contains(&format!(" {} ", concept_words(negation).join(" ")))
            }) {
                return Some("NegatedDocumentationAction");
            }
            if index == 0 || prefixes.contains(&before) {
                return Some("owned");
            }
        }
    }
    let raw = concept_words(task);
    (mentioned || actions.iter().any(|action| raw.contains(action)))
        .then_some("DocumentationActionUnowned")
}

fn property_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn manifest_declaration_request(task: &str) -> bool {
    let Some(policy) = source_documentation_policy() else {
        return false;
    };
    if quote_fault(task).is_some() {
        return false;
    }
    let mut masked = task.as_bytes().to_vec();
    for span in quoted_segment_spans(task) {
        for byte in &mut masked[span.start..span.end] {
            if !matches!(*byte, b'\r' | b'\n') {
                *byte = b' ';
            }
        }
    }
    let masked = String::from_utf8(masked).expect("MaskedManifestRequestUtf8");
    sentences(&masked).iter().any(|clause| {
        let words = concept_words(clause.text);
        let phrase = format!(" {} ", words.join(" "));
        if policy["negations"].as_array().is_some_and(|cues| {
            cues.iter()
                .filter_map(Value::as_str)
                .any(|cue| phrase.contains(&format!(" {} ", concept_words(cue).join(" "))))
        }) {
            return false;
        }
        ["dependencyCues", "versionCues", "sectionCues"]
            .iter()
            .all(|key| {
                policy[*key].as_array().is_some_and(|cues| {
                    cues.iter()
                        .filter_map(Value::as_str)
                        .any(|cue| words.iter().any(|word| word == cue))
                })
            })
    })
}

/// Bind one requested concept to a scalar source field; ambiguity remains unresolved.
// Literal named template slots are replaced below; these are not Rust format strings.
#[expect(
    clippy::literal_string_with_formatting_args,
    reason = "literal_template_slots_are_not_format_strings"
)]
pub(super) fn derive_source_documentation(
    task: &str,
    source_path: &str,
    source: &str,
    guide: &str,
    template: &str,
) -> Value {
    let Ok(value) = observed_source_json(source) else {
        return json!({"gap":"InvalidSourceJson"});
    };
    let Some(object) = value.as_object() else {
        return json!({"gap":"SourceObjectRequired"});
    };
    let requested = concept_words(task);
    let bindings: Vec<_> = object
        .iter()
        .filter(|(key, value)| {
            let words = concept_words(key);
            !words.is_empty()
                && words.iter().all(|word| requested.contains(word))
                && property_text(value).is_some()
        })
        .collect();
    if manifest_declaration_request(task) {
        if !bindings.is_empty() {
            return json!({"gap":"AmbiguousDocumentationBinding"});
        }
        return derive_manifest_documentation(task, source_path, source, guide, None);
    }
    if bindings.len() != 1 {
        return json!({"gap":if bindings.is_empty() {"MissingPropertyBinding"} else {"AmbiguousPropertyBinding"}});
    }
    let (key, value) = bindings[0];
    let text = property_text(value).expect("ScalarBinding");
    if text.contains(['\r', '\n', '`']) || source_path.contains(['\r', '\n', '`']) {
        return json!({"gap":"UnsupportedDocumentationValue"});
    }
    let paragraph = template
        .replace("{source}", source_path)
        .replace("{concept}", &concept_words(key).join(" "))
        .replace("{value}", &text);
    let content = if guide.contains(&paragraph) {
        guide.to_owned()
    } else {
        format!(
            "{}\n\n{paragraph}\n",
            guide.strip_suffix('\n').unwrap_or(guide)
        )
    };
    json!({"key":key,"value":value,"paragraph":paragraph,"sourceIdentity":sha256_hex(source.as_bytes()),"content":content})
}

/// Same read/candidate/edit/readback and no-op contract as the JavaScript route.
pub(super) fn plan_source_documentation_step(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    work_item: Option<&str>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let policy = source_documentation_policy()?;
    let values = |name: &str| {
        policy[name]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    let template = policy["template"].as_str()?;
    let paths = super::work_item_evidence::work_item_evidence_paths(task);
    let sources = paths
        .iter()
        .copied()
        .filter(|path| path.ends_with(".json"))
        .collect::<Vec<_>>();
    let guides = paths
        .iter()
        .copied()
        .filter(|path| path.ends_with(".md"))
        .collect::<Vec<_>>();
    if paths.len() != 2 || sources.len() != 1 || guides.len() != 1 {
        return None;
    }
    let source_path = sources[0];
    let target = guides[0];
    let authority = documentation_authority(
        task,
        target,
        &values("actions"),
        &values("preserve"),
        &values("negations"),
        &values("prefixes"),
        true,
        true,
    )?;
    Some(plan_observed_documentation(
        task,
        messages,
        tools,
        &json!({"sources":sources,"target":target,"authority":authority,"workItem":work_item}),
        |observed, guide| {
            observed
                .first()
                .and_then(|row| row["source"].as_str())
                .map_or_else(
                    || json!({"gap":"EditPreconditionObservationMissing"}),
                    |source| {
                        derive_source_documentation(task, source_path, source, guide, template)
                    },
                )
        },
        result,
    ))
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/source_documentation.rs"]
mod tests;
