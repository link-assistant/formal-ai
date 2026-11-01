//! Manual native projection; source only, not verified translation credit.
use super::source_documentation::source_documentation_policy;
use super::source_lexical_policy::source_lexical_rule;
use super::source_manifest_facts::observed_manifest_facts;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::OnceLock;
fn gap(reason: &str) -> Value {
    json!({"gap":reason})
}
fn identifier(character: char) -> bool {
    static PATTERN: OnceLock<regex::Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| regex::Regex::new(r"[\p{L}\p{N}_$@/.-]").expect("StaticManifestIdentifier"))
        .is_match(&character.to_string())
}
fn mentions(text: &str, name: &str) -> bool {
    !name.is_empty()
        && text.match_indices(name).any(|(offset, _)| {
            text[..offset]
                .chars()
                .next_back()
                .is_none_or(|character| !identifier(character))
                && text[offset + name.len()..]
                    .chars()
                    .next()
                    .is_none_or(|character| !identifier(character))
        })
}
fn canonical(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(canonical).collect()),
        Value::Object(object) => {
            let ordered: BTreeMap<_, _> = object
                .iter()
                .map(|(key, value)| (key.clone(), canonical(value)))
                .collect();
            serde_json::to_value(ordered).expect("CanonicalManifestOptions")
        }
        value => value.clone(),
    }
}
fn safe(text: &str) -> bool {
    static PATTERN: OnceLock<regex::Regex> = OnceLock::new();
    text.len() <= 2048
        && !PATTERN
            .get_or_init(|| regex::Regex::new(r"[`|\p{Cc}\p{Cf}]").expect("StaticManifestCell"))
            .is_match(text)
}
fn fill(template: &str, bindings: &[(&str, &str)]) -> Result<String, ()> {
    let tokens = regex::Regex::new(r"\{([a-z]+)\}").expect("StaticManifestTemplate");
    let mut output = String::new();
    let mut end = 0;
    for captures in tokens.captures_iter(template) {
        let whole = captures.get(0).ok_or(())?;
        output.push_str(&template[end..whole.start()]);
        let key = captures.get(1).ok_or(())?.as_str();
        output.push_str(bindings.iter().find(|(name, _)| *name == key).ok_or(())?.1);
        end = whole.end();
    }
    output.push_str(&template[end..]);
    Ok(output)
}
struct OwnedBlock {
    owner: String,
    start: usize,
    end: usize,
}
fn owned_blocks(guide: &str, policy: &Value) -> Result<Vec<OwnedBlock>, ()> {
    let formats: Vec<Vec<&str>> = ["manifestStart", "manifestEnd"]
        .iter()
        .map(|key| {
            policy[*key]
                .as_str()
                .unwrap_or("")
                .split("{source}")
                .collect()
        })
        .collect();
    if formats.iter().any(|parts| parts.len() != 2) {
        return Err(());
    }
    let owner_pattern =
        regex::Regex::new(r"^[A-Za-z0-9_][A-Za-z0-9_./-]*$").expect("StaticManifestOwner");
    let mut blocks = Vec::new();
    let mut owners = std::collections::BTreeSet::new();
    let mut pending: Option<(String, usize)> = None;
    let mut offset = 0;
    for raw in guide.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        for (kind, parts) in formats.iter().enumerate() {
            let (prefix, suffix) = (parts[0], parts[1]);
            if !line.contains(prefix) {
                continue;
            }
            let Some(owner) = line
                .strip_prefix(prefix)
                .and_then(|text| text.strip_suffix(suffix))
            else {
                return Err(());
            };
            if !owner_pattern.is_match(owner)
                || source_lexical_rule(owner, "html-comment-forbidden") != Some(false)
                || owner
                    .split('/')
                    .any(|part| part.is_empty() || part == "." || part == "..")
            {
                return Err(());
            }
            if kind == 0 {
                if pending.is_some() || !owners.insert(owner.to_owned()) {
                    return Err(());
                }
                pending = Some((owner.to_owned(), offset));
            } else {
                let Some((prior, start)) = pending.take() else {
                    return Err(());
                };
                if prior != owner {
                    return Err(());
                }
                blocks.push(OwnedBlock {
                    owner: prior,
                    start,
                    end: offset + line.len(),
                });
            }
        }
        offset += raw.len() + 1;
    }
    if pending.is_some() {
        return Err(());
    }
    Ok(blocks)
}
/// Render literal manifest declarations, never executable-use claims.
pub(super) fn derive_manifest_documentation(
    task: &str,
    source_path: &str,
    source: &str,
    guide: &str,
    forms: Option<&Value>,
) -> Value {
    if source.len().saturating_add(guide.len()) > 262_144 || task.len() > 262_144 {
        return gap("ManifestDocumentationByteLimit");
    }
    let fallback = source_documentation_policy();
    let Some(policy) = forms.or(fallback.as_ref()) else {
        return gap("MissingManifestDocumentationForms");
    };
    if [
        "manifestStart",
        "manifestEnd",
        "manifestTitle",
        "manifestNotice",
        "manifestHeader",
        "manifestSeparator",
        "manifestRow",
        "manifestAbsentVersion",
        "manifestUnqualifiedTarget",
    ]
    .iter()
    .any(|key| !policy[*key].is_string())
    {
        return gap("MissingManifestDocumentationForms");
    }
    let Ok(facts) = observed_manifest_facts(source_path, source) else {
        return gap("UnsupportedManifestDocumentationSource");
    };
    let declarations: Vec<_> = facts
        .into_iter()
        .filter(|fact| fact["kind"] == "manifest-declaration")
        .collect();
    let named: Vec<_> = declarations
        .iter()
        .filter(|fact| {
            fact["name"]
                .as_str()
                .is_some_and(|name| mentions(task, name))
                || fact["options"]["package"]
                    .as_str()
                    .is_some_and(|name| mentions(task, name))
        })
        .cloned()
        .collect();
    let selected = if named.is_empty() {
        declarations
    } else {
        named
    };
    if selected.is_empty() {
        return gap("MissingManifestDeclarations");
    }
    if selected.len() > 32 {
        return gap("ManifestDocumentationRowLimit");
    }
    if !safe(source_path)
        || source_lexical_rule(source_path, "html-comment-forbidden") != Some(false)
    {
        return gap("UnsupportedManifestDocumentationValue");
    }
    let field = |key: &str| policy[key].as_str().ok_or(());
    let rendered = (|| -> Result<(String, String, String), ()> {
        let start = fill(field("manifestStart")?, &[("source", source_path)])?;
        let end = fill(field("manifestEnd")?, &[("source", source_path)])?;
        let mut lines = vec![
            start.clone(),
            fill(field("manifestTitle")?, &[("source", source_path)])?,
            field("manifestNotice")?.to_owned(),
            field("manifestHeader")?.to_owned(),
            field("manifestSeparator")?.to_owned(),
        ];
        for fact in &selected {
            let name = fact["name"].as_str().ok_or(())?;
            let section = fact["section"].as_str().ok_or(())?;
            let version = fact["version"]
                .as_str()
                .unwrap_or(field("manifestAbsentVersion")?);
            let target = fact["target"]
                .as_str()
                .unwrap_or(field("manifestUnqualifiedTarget")?);
            let options = canonical(fact.get("options").unwrap_or(&json!({}))).to_string();
            if name.is_empty()
                || ![name, section, version, target, &options]
                    .iter()
                    .all(|text| safe(text))
            {
                return Err(());
            }
            lines.push(fill(
                field("manifestRow")?,
                &[
                    ("name", name),
                    ("version", version),
                    ("section", section),
                    ("target", target),
                    ("options", &options),
                ],
            )?);
        }
        lines.push(end.clone());
        Ok((start, end, lines.join("\n")))
    })();
    let (start, end, paragraph) = match rendered {
        Ok(value) => value,
        Err(()) => return gap("UnsupportedManifestDocumentationValue"),
    };
    if start.is_empty() || end.is_empty() || start == end {
        return gap("MissingManifestDocumentationForms");
    }
    let blocks = match owned_blocks(guide, policy) {
        Ok(blocks) => blocks,
        Err(()) => return gap("AmbiguousManifestDocumentationBlock"),
    };
    let current = blocks.iter().find(|block| block.owner == source_path);
    let mut unowned = guide.to_owned();
    for block in blocks.iter().rev() {
        unowned.replace_range(block.start..block.end, "");
    }
    for line in unowned.lines() {
        if !line.contains('|') {
            continue;
        }
        let cells: Vec<_> = line
            .split('|')
            .map(|cell| cell.trim().trim_start_matches('`').trim_end_matches('`'))
            .collect();
        let matching: Vec<_> = selected
            .iter()
            .filter(|fact| {
                fact["name"]
                    .as_str()
                    .is_some_and(|name| cells.contains(&name))
            })
            .collect();
        if !matching.is_empty()
            && !matching.iter().any(|fact| {
                let version = fact["version"]
                    .as_str()
                    .unwrap_or_else(|| policy["manifestAbsentVersion"].as_str().unwrap_or(""));
                fact["section"]
                    .as_str()
                    .is_some_and(|section| cells.contains(&version) && cells.contains(&section))
            })
        {
            return gap("ConflictingManifestDocumentation");
        }
    }
    let content = match current {
        Some(block) => format!(
            "{}{paragraph}{}",
            &guide[..block.start],
            &guide[block.end..]
        ),
        None => format!(
            "{guide}{}{paragraph}\n",
            if guide.ends_with("\n\n") {
                ""
            } else if guide.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            }
        ),
    };
    if content.len() > 262_144 {
        return gap("ManifestDocumentationByteLimit");
    }
    json!({"key":"manifest-declarations","value":selected,"facts":selected,"paragraph":paragraph,"sourceIdentity":sha256_hex(source.as_bytes()),"content":content})
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/manifest_documentation.rs"]
mod tests;
