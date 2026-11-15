//! Independent acceptance of observed documentation (issue #1483).
//! JavaScript twin: js/agentic/documentation_acceptance.mjs.
//!
//! The derivation that wrote a documentation candidate never accepts it. These
//! checks read only the task text, the raw observed sources and the readback
//! artifact: a version the task claims must be the declared one, every derived
//! fact must be anchored in its source and present in the artifact, and the
//! guide's earlier lines must survive the edit. Requirements are verified one
//! by one from the code spans they name; anything else stays unverified.

use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::sync::OnceLock;

const MAXIMUM_DIFF_LINES: usize = 400;

fn pattern(cell: &'static OnceLock<regex::Regex>, source: &str) -> &'static regex::Regex {
    cell.get_or_init(|| regex::Regex::new(source).expect("StaticAcceptancePattern"))
}
fn code_span() -> &'static regex::Regex {
    static CELL: OnceLock<regex::Regex> = OnceLock::new();
    pattern(&CELL, r"`([^`\n]{1,200})`")
}
fn claim_pattern() -> &'static regex::Regex {
    static CELL: OnceLock<regex::Regex> = OnceLock::new();
    pattern(
        &CELL,
        r#"^\s*"?([@A-Za-z0-9_./-]+)"?\s*[=:]\s*"([^"\n]+)"\s*$"#,
    )
}
fn word_pattern() -> &'static regex::Regex {
    static CELL: OnceLock<regex::Regex> = OnceLock::new();
    pattern(&CELL, r"[\p{L}\p{N}']+")
}
fn url_pattern() -> &'static regex::Regex {
    static CELL: OnceLock<regex::Regex> = OnceLock::new();
    pattern(&CELL, r"https?://\S+")
}
fn compound_pattern() -> &'static regex::Regex {
    static CELL: OnceLock<regex::Regex> = OnceLock::new();
    pattern(&CELL, r"[@A-Za-z0-9_][@A-Za-z0-9_./:-]*[A-Za-z0-9_]")
}
fn compound_joint() -> &'static regex::Regex {
    static CELL: OnceLock<regex::Regex> = OnceLock::new();
    pattern(&CELL, r"[-_/.]|::")
}
fn owned_start() -> &'static regex::Regex {
    static CELL: OnceLock<regex::Regex> = OnceLock::new();
    pattern(&CELL, r"^<!-- formal-ai-[a-z-]+-start: \S+ -->$")
}

/// Code spans of `text`, in order.
#[must_use]
pub(super) fn code_spans(text: &str) -> Vec<String> {
    code_span()
        .captures_iter(text)
        .map(|captures| captures[1].trim().to_owned())
        .filter(|span| !span.is_empty())
        .collect()
}

/// Version claims written as `name = "version"` (or `"name": "version"`) in the task.
#[must_use]
pub(super) fn documentation_claims(task: &str) -> Vec<Value> {
    let mut claims: Vec<Value> = Vec::new();
    for reference in code_spans(task) {
        if let Some(captures) = claim_pattern().captures(&reference)
            && !claims.iter().any(|claim| claim["reference"] == reference)
        {
            claims.push(json!({"reference":reference,"name":&captures[1],"version":&captures[2]}));
        }
    }
    claims
}

fn as_rows(value: &Value) -> &[Value] {
    value.as_array().map_or(&[][..], Vec::as_slice)
}

/// Each claim against the declarations: `confirmed` when a declaration has the
/// name and the version, `contradicted` when declarations of the name exist
/// but none has the version, `unobserved` when no declaration has the name.
#[must_use]
pub(super) fn judged_claims(claims: &[Value], facts: &Value) -> Vec<Value> {
    claims
        .iter()
        .map(|claim| {
            let named = as_rows(facts)
                .iter()
                .filter(|fact| {
                    fact["kind"] == "manifest-declaration"
                        && fact["name"].is_string()
                        && fact["name"] == claim["name"]
                })
                .collect::<Vec<_>>();
            let mut observed: Vec<&str> = Vec::new();
            for version in named.iter().filter_map(|fact| fact["version"].as_str()) {
                if !observed.contains(&version) {
                    observed.push(version);
                }
            }
            let status = if named.is_empty() {
                "unobserved"
            } else if claim["version"]
                .as_str()
                .is_some_and(|version| observed.contains(&version))
            {
                "confirmed"
            } else {
                "contradicted"
            };
            let mut judged = claim.clone();
            judged["status"] = json!(status);
            judged["observed"] = json!(observed);
            judged
        })
        .collect()
}

/// The cells of a table row with surrounding backticks removed; empty for any other line.
fn cells(line: &str) -> Vec<String> {
    if !line.trim().starts_with('|') {
        return Vec::new();
    }
    line.split('|')
        .map(|cell| cell.trim().trim_matches('`').trim().to_owned())
        .collect()
}

/// Every derived fact is anchored in its raw source and appears in the
/// artifact. A fact of a kind this check cannot read makes it unavailable.
#[must_use]
pub(super) fn source_anchored_facts(facts: &Value, sources: &[Value], artifact: &str) -> Value {
    let source_of = |path: &Value| {
        sources
            .iter()
            .find(|row| row["path"] == *path)
            .and_then(|row| row["source"].as_str())
    };
    let rows = artifact
        .split('\n')
        .map(cells)
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();
    let mut missing: Vec<Value> = Vec::new();
    let mut unavailable = false;
    for fact in as_rows(facts) {
        if fact["kind"] == "manifest-declaration" {
            // A declaration without a literal version (a path or workspace
            // dependency) is anchored by its name; its row still needs the section.
            let source = source_of(&fact["manifest"]);
            let anchors = [&fact["name"], &fact["version"]]
                .into_iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>();
            let mut values = anchors.clone();
            let place = if source.is_none_or(|source| {
                fact["name"].as_str().is_none()
                    || !anchors.iter().all(|value| source.contains(value))
            }) {
                Some("source")
            } else if fact["section"].as_str().is_none_or(|section| {
                values.push(section);
                !rows.iter().any(|row| {
                    values
                        .iter()
                        .all(|value| row.iter().any(|cell| cell == value))
                })
            }) {
                Some("artifact")
            } else {
                None
            };
            if let Some(place) = place {
                missing.push(json!({"kind":fact["kind"],"path":fact["manifest"],"name":fact["name"],"place":place}));
            }
        } else if fact["kind"] == "observed-source-usage" {
            let source = source_of(&fact["owner"]);
            for call in as_rows(&fact["calls"]) {
                let text = call["source"].as_str();
                match (source, text) {
                    (Some(source), Some(text)) if source.contains(text) => {
                        if !artifact.contains(text) {
                            missing.push(json!({"kind":fact["kind"],"path":fact["owner"],"name":call["imported"],"place":"artifact"}));
                        }
                    }
                    _ => missing.push(json!({"kind":fact["kind"],"path":fact["owner"],"name":call["imported"],"place":"source"})),
                }
            }
        } else {
            unavailable = true;
        }
    }
    let checked = as_rows(facts).len();
    let status = if !missing.is_empty() {
        "failed"
    } else if unavailable || checked == 0 {
        "unavailable"
    } else {
        "passed"
    };
    json!({"kind":"source-anchored-facts","status":status,"checked":checked,"missing":missing})
}

/// Non-blank preimage lines in order inside the postimage, and the added
/// lines. Lines of a block the guide marks as generated
/// (`<!-- formal-ai-…-start: … -->` to its end marker) are owned by the
/// derivation and may be regenerated.
#[must_use]
pub(super) fn preserved_preimage(preimage: &str, postimage: &str) -> (Value, Vec<String>) {
    let mut before: Vec<&str> = Vec::new();
    let mut owner: Option<String> = None;
    let mut owned = 0_usize;
    for line in preimage.split('\n') {
        if owner.is_none() && owned_start().is_match(line) {
            // The kind holds no colon, so the first `-start:` is the marker's.
            owner = Some(line.replacen("-start:", "-end:", 1));
        }
        if let Some(end) = &owner {
            owned += 1;
            if line == end {
                owner = None;
            }
        } else if !line.trim().is_empty() {
            before.push(line);
        }
    }
    let mut added: Vec<String> = Vec::new();
    let mut next = 0_usize;
    for line in postimage.split('\n') {
        if next < before.len() && line == before[next] {
            next += 1;
        } else if !line.trim().is_empty() {
            added.push(line.to_owned());
        }
    }
    let status = if next == before.len() && owner.is_none() {
        "passed"
    } else {
        "failed"
    };
    (
        json!({"kind":"preimage-preserved","status":status,"kept":next,"expected":before.len(),"owned":owned}),
        added,
    )
}

/// The bounded added-line diff of a candidate edit.
#[must_use]
pub(super) fn added_lines_diff(added: &[String]) -> Value {
    let shown = added
        .iter()
        .take(MAXIMUM_DIFF_LINES)
        .map(|line| format!("+{line}"))
        .collect::<Vec<_>>();
    json!({"added":added.len(),"truncated":added.len() > shown.len(),"lines":shown})
}

/// The terms a requirement names: its code spans, or, when it has none, its
/// compound tokens (`ribbon-core`, `docs/guide.md`, `Path::item`) outside links.
#[must_use]
pub(super) fn requirement_terms(text: &str) -> Vec<String> {
    let spans = code_spans(text)
        .into_iter()
        .map(|span| span.strip_suffix("()").map_or(span.clone(), str::to_owned))
        .collect::<Vec<_>>();
    if !spans.is_empty() {
        return spans;
    }
    let unlinked = url_pattern().replace_all(text, " ");
    let mut terms: Vec<String> = Vec::new();
    for token in compound_pattern()
        .find_iter(&unlinked)
        .map(|found| found.as_str())
    {
        if token
            .chars()
            .any(|character| character.is_ascii_alphabetic())
            && compound_joint().is_match(token)
            && !terms.iter().any(|term| term == token)
        {
            terms.push(token.to_owned());
        }
    }
    terms
}

/// Terms a passed fact check establishes: names, owners, manifests, modules and imported bindings.
fn established_terms(facts: &Value, claims: &[Value], target: &str) -> BTreeSet<String> {
    let mut terms = claims
        .iter()
        .filter(|claim| claim["status"] == "confirmed")
        .filter_map(|claim| claim["reference"].as_str().map(str::to_owned))
        .collect::<BTreeSet<_>>();
    terms.insert(target.to_owned());
    for fact in as_rows(facts) {
        for value in [&fact["name"], &fact["manifest"], &fact["owner"]] {
            if let Some(value) = value.as_str() {
                terms.insert(value.to_owned());
            }
        }
        for entry in as_rows(&fact["imports"]) {
            if let Some(module) = entry["module"].as_str() {
                terms.insert(module.to_owned());
            }
            for binding in as_rows(&entry["bindings"]) {
                for value in [&binding["imported"], &binding["local"]] {
                    if let Some(value) = value.as_str() {
                        terms.insert(value.to_owned());
                    }
                }
            }
        }
    }
    terms
}

/// Whether `text` asks to keep existing content: a preserve cue not directly negated.
#[must_use]
pub(super) fn preserve_cue(text: &str, preserve: &[String], negations: &[String]) -> bool {
    let lowered = text.to_lowercase();
    let words = word_pattern()
        .find_iter(&lowered)
        .map(|found| found.as_str())
        .collect::<Vec<_>>();
    words.iter().enumerate().any(|(index, word)| {
        preserve.iter().any(|cue| cue == word)
            && !index.checked_sub(1).is_some_and(|previous| {
                negations.iter().any(|negation| negation == words[previous])
            })
    })
}

/// What a requirement is judged against.
pub(super) struct RequirementEvidence<'a> {
    pub facts: &'a Value,
    pub claims: &'a [Value],
    pub target: &'a str,
    pub facts_passed: bool,
    pub preservation: Option<&'a str>,
    pub preserve: &'a [String],
    pub negations: &'a [String],
}

/// The status of each requirement. A requirement is verified when it names at
/// least one term and every term is established by the passed fact check (a
/// fact's name, manifest, owner, module or binding, a confirmed claim, or the
/// accepted target), or names none and asks to keep existing content; a
/// preserve cue always needs the preservation check. A requirement that
/// names a contradicted claim is contradicted; any other stays unverified.
#[must_use]
pub(super) fn judged_requirements(
    requirements: &[Value],
    evidence: &RequirementEvidence<'_>,
) -> Vec<Value> {
    let terms = if evidence.facts_passed {
        established_terms(evidence.facts, evidence.claims, evidence.target)
    } else {
        BTreeSet::new()
    };
    let contradicted = evidence
        .claims
        .iter()
        .filter(|claim| claim["status"] == "contradicted")
        .filter_map(|claim| claim["reference"].as_str())
        .collect::<BTreeSet<_>>();
    requirements
        .iter()
        .map(|requirement| {
            let text = requirement["text"].as_str().unwrap_or_default();
            let spans = requirement_terms(text);
            let mut judged = requirement.clone();
            if spans
                .iter()
                .any(|span| contradicted.contains(span.as_str()))
            {
                judged["status"] = json!("contradicted");
                return judged;
            }
            let cue = preserve_cue(text, evidence.preserve, evidence.negations);
            let established = if spans.is_empty() {
                cue
            } else {
                spans.iter().all(|span| terms.contains(span))
            };
            let kept = !cue || evidence.preservation == Some("passed");
            if established && kept {
                judged["status"] = json!("verified");
                judged["evidence"] = if spans.is_empty() {
                    json!(["preimage-preserved"])
                } else {
                    json!(spans)
                };
            }
            judged
        })
        .collect()
}
