//! Finite JSON declaration claims over actual current qualified Read evidence.
use super::super::progress::Progress;
use super::super::shell_command_policy::sentences;
use super::super::source_manifest_facts::observed_source_json;
use crate::normal_markov::{quote_fault, quoted_segment_spans};
use crate::seed::parser::parse_lino;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
use std::sync::OnceLock;

struct QuestionGrammar {
    copulas: Vec<String>,
    subjects: Vec<String>,
    negations: Vec<String>,
    ends: Vec<String>,
}
fn question_grammar() -> Option<&'static QuestionGrammar> {
    static GRAMMAR: OnceLock<Option<QuestionGrammar>> = OnceLock::new();
    GRAMMAR
        .get_or_init(|| {
            let root = parse_lino(include_str!(
                "../../../embedded/data/meta/source-scalar-questions.lino"
            ));
            let node = root
                .children
                .iter()
                .find(|node| node.name == "source-scalar-questions")?;
            if node.find_child_value("schema") != "source-scalar-questions/v1" {
                return None;
            }
            let values = |name: &str| {
                node.children
                    .iter()
                    .filter(|child| child.name == name)
                    .map(|child| child.id.clone())
                    .collect::<Vec<_>>()
            };
            Some(QuestionGrammar {
                copulas: values("copula"),
                subjects: values("subject"),
                negations: values("negation"),
                ends: values("terminator"),
            })
        })
        .as_ref()
}
fn scalar(value: &Value) -> bool {
    !value.is_array() && !value.is_object()
}

/// Full question ownership; offsets are UTF-8 byte spans into the request.
pub(super) fn source_scalar_question(request: &str, key: &str) -> Option<Option<Value>> {
    let grammar = question_grammar()?;
    if request.len() > 16_384 {
        return None;
    }
    let mut masked = request.to_owned();
    for quoted in quoted_segment_spans(request).iter().rev() {
        masked.replace_range(
            quoted.start..quoted.end,
            &" ".repeat(quoted.end - quoted.start),
        );
    }
    let candidates = sentences(&masked)
        .into_iter()
        .filter(|sentence| {
            let text = masked[sentence.span.clone()].to_lowercase();
            grammar
                .subjects
                .iter()
                .any(|subject| text.contains(&subject.to_lowercase()))
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() != 1 || quote_fault(request).is_some() {
        return Some(None);
    }
    let sentence = &candidates[0];
    let raw = &request[sentence.span.clone()];
    let leading = raw.len() - raw.trim_start().len();
    let text = raw.trim();
    let start = sentence.span.start + leading;
    let mut claims = Vec::new();
    for copula in &grammar.copulas {
        for subject in &grammar.subjects {
            let prefix = format!("{copula} {subject} {key} ");
            if !text.to_lowercase().starts_with(&prefix.to_lowercase()) {
                continue;
            }
            let key_start = start + copula.len() + 1 + subject.len() + 1;
            if request.get(key_start..key_start + key.len()) != Some(key) {
                continue;
            }
            for ending in &grammar.ends {
                if !text.ends_with(ending) {
                    continue;
                }
                let operand_start = start + prefix.len();
                let Some(rest) = text.get(prefix.len()..text.len() - ending.len()) else {
                    continue;
                };
                for marker in
                    std::iter::once("").chain(grammar.negations.iter().map(String::as_str))
                {
                    let lead = if marker.is_empty() {
                        String::new()
                    } else {
                        format!("{marker} ")
                    };
                    if !rest.to_lowercase().starts_with(&lead.to_lowercase()) {
                        continue;
                    }
                    let Some(tail) = rest.get(lead.len()..) else {
                        continue;
                    };
                    let value_text = tail.trim();
                    let value_start =
                        operand_start + lead.len() + tail.len() - tail.trim_start().len();
                    let Ok(expected) = observed_source_json(value_text) else {
                        continue;
                    };
                    if !scalar(&expected) {
                        continue;
                    }
                    let polarity = if marker.is_empty() {
                        "positive"
                    } else {
                        "negative"
                    };
                    let polarity_span = if marker.is_empty() {
                        [start, start + text.len()]
                    } else {
                        [operand_start, operand_start + marker.len()]
                    };
                    claims.push(json!({
                        "key": key, "expected": expected, "polarity": polarity,
                        "requestIdentity": sha256_hex(request.as_bytes()),
                        "claimSpan": [start,start+text.len()],
                        "keySpan": [key_start,key_start+key.len()],
                        "expectedSpan": [value_start,value_start+value_text.len()],
                        "polaritySpan": polarity_span
                    }));
                }
            }
        }
    }
    Some(if claims.len() == 1 {
        claims.pop()
    } else {
        None
    })
}
fn scalar_evidence(value: &Value) -> Value {
    if let Some(text) = value.as_str().filter(|text| text.len() > 512) {
        json!({"type":"string","bytes":text.len(),"sha256":sha256_hex(text.as_bytes())})
    } else {
        value.clone()
    }
}
fn unresolved(mut report: Value, reason: &str) -> Value {
    report["unresolved"] = json!([reason]);
    report
}
fn scalar_equal(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(left), Value::Number(right)) => left.as_f64() == right.as_f64(),
        _ => actual == expected,
    }
}

/// No legacy/supplied/Grep observation or prior probability grants source authority.
pub(super) fn source_scalar_assessment(
    request: &str,
    path: &str,
    key: &str,
    progress: &Progress,
) -> Option<Value> {
    let claim = source_scalar_question(request, key)?;
    let mut displayed_claim = claim.clone();
    if let Some(value) = displayed_claim.as_mut() {
        value["expected"] = scalar_evidence(&value["expected"]);
    }
    let mut report = json!({"schema":"source-scalar-assessment/v1","status":"unknown",
        "scope":"declared-json-scalar-only","claim":displayed_claim,"evidence":[],"unresolved":[]});
    let Some(read) = progress.fresh_source_read_for(path) else {
        return Some(unresolved(report, "fresh-complete-source-read-unavailable"));
    };
    let Some(source) = read.source.as_deref() else {
        return Some(unresolved(report, "fresh-complete-source-read-unavailable"));
    };
    let Ok(document) = observed_source_json(source) else {
        return Some(unresolved(report, "strict-json-observation-unavailable"));
    };
    let Some(actual) = document.as_object().and_then(|document| document.get(key)) else {
        return Some(unresolved(report, "declared-key-unavailable"));
    };
    if !scalar(actual) {
        return Some(unresolved(report, "declared-value-not-scalar"));
    }
    report["evidence"] = json!([{"owner":path,"sourceIdentity":sha256_hex(source.as_bytes()),
        "documentSpan":[0,source.len()],"spanScope":"complete-document","key":key,
        "actual":scalar_evidence(actual)}]);
    let Some(claim) = claim else {
        return Some(unresolved(report, "comparison-frame-not-owned"));
    };
    let equal = scalar_equal(actual, &claim["expected"]);
    let supported = if claim["polarity"] == "positive" {
        equal
    } else {
        !equal
    };
    report["status"] = json!(if supported {
        "supported"
    } else {
        "contradicted"
    });
    Some(report)
}

#[cfg(test)]
#[path = "../../../tests/unit/agentic-coding/source_scalar_claims.rs"]
mod tests;
