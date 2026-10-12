//! The trimmed Wikidata entity snapshot the importer caches (issue #660, R378).
//!
//! A live fetch keeps only the `type`, `id`, `labels`, `descriptions` and
//! `aliases` of an entity, restricted to the cached project languages, and
//! writes it in the field order and layout of the Python curation script so a
//! live write is byte-identical to the committed cache.

use std::fmt::Write as _;

use serde_json::Value;

use super::{cached_languages, diagnostic};

/// Trim a full entity document to the cached project languages, keeping the
/// `type`, `id`, `labels`, `descriptions`, and `aliases` fields in that order —
/// mirroring `scripts/ground-meanings.rs` so live writes match the committed
/// cache byte-for-byte.
pub(super) fn trim_entity(full: &Value, qid: &str) -> Result<Ordered, String> {
    let entity = full
        .get("entities")
        .and_then(|entities| entities.get(qid))
        .ok_or_else(|| diagnostic("lexeme_import_qid_absent_from_fetch", &[("qid", qid)]))?;
    let mut trimmed = Vec::new();
    if let Some(kind) = entity.get("type").and_then(Value::as_str) {
        trimmed.push(("type".to_string(), Ordered::Str(kind.to_string())));
    }
    trimmed.push(("id".to_string(), Ordered::Str(qid.to_string())));
    if let Some(section) = keep_languages(entity.get("labels")) {
        trimmed.push(("labels".to_string(), section));
    }
    if let Some(section) = keep_languages(entity.get("descriptions")) {
        trimmed.push(("descriptions".to_string(), section));
    }
    if let Some(section) = keep_language_arrays(entity.get("aliases")) {
        trimmed.push(("aliases".to_string(), section));
    }
    let entities = Ordered::Obj(vec![(qid.to_string(), Ordered::Obj(trimmed))]);
    Ok(Ordered::Obj(vec![
        ("entities".to_string(), entities),
        ("success".to_string(), Ordered::Int(1)),
    ]))
}

/// An insertion-ordered JSON value, used so the trimmed cache serialises in the
/// same field order as the Python curation script (`serde_json`'s map is sorted).
pub(super) enum Ordered {
    Str(String),
    Int(i64),
    Obj(Vec<(String, Self)>),
    Arr(Vec<Self>),
}

fn keep_languages(section: Option<&Value>) -> Option<Ordered> {
    let object = section?.as_object()?;
    let mut kept = Vec::new();
    for language in cached_languages() {
        if let Some(entry) = object.get(language) {
            kept.push((language.to_string(), value_to_ordered(entry)));
        }
    }
    (!kept.is_empty()).then_some(Ordered::Obj(kept))
}

fn keep_language_arrays(section: Option<&Value>) -> Option<Ordered> {
    let object = section?.as_object()?;
    let mut kept = Vec::new();
    for language in cached_languages() {
        if let Some(Value::Array(items)) = object.get(language)
            && !items.is_empty()
        {
            kept.push((
                language.to_string(),
                Ordered::Arr(items.iter().map(value_to_ordered).collect()),
            ));
        }
    }
    (!kept.is_empty()).then_some(Ordered::Obj(kept))
}

fn value_to_ordered(value: &Value) -> Ordered {
    match value {
        Value::String(text) => Ordered::Str(text.clone()),
        Value::Number(number) => Ordered::Int(number.as_i64().unwrap_or_default()),
        Value::Array(items) => Ordered::Arr(items.iter().map(value_to_ordered).collect()),
        Value::Object(object) => Ordered::Obj(
            object
                .iter()
                .map(|(key, inner)| (key.clone(), value_to_ordered(inner)))
                .collect(),
        ),
        Value::Bool(_) | Value::Null => Ordered::Str(String::new()),
    }
}

/// Serialise an [`Ordered`] value as pretty JSON matching Python's
/// `json.dump(ensure_ascii=False, indent=2)` (two-space indent, `": "`
/// separators, trailing newline) so live writes are byte-identical to the
/// committed cache.
pub(super) fn serialize_trimmed(value: &Ordered) -> String {
    let mut out = String::new();
    write_ordered(&mut out, value, 0);
    out.push('\n');
    out
}

fn write_ordered(out: &mut String, value: &Ordered, indent: usize) {
    match value {
        Ordered::Str(text) => {
            out.push('"');
            out.push_str(&escape_json(text));
            out.push('"');
        }
        Ordered::Int(number) => {
            let _ = write!(out, "{number}");
        }
        Ordered::Obj(entries) => {
            if entries.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (index, (key, inner)) in entries.iter().enumerate() {
                pad(out, indent + 2);
                out.push('"');
                out.push_str(&escape_json(key));
                out.push_str("\": ");
                write_ordered(out, inner, indent + 2);
                if index + 1 < entries.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            pad(out, indent);
            out.push('}');
        }
        Ordered::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (index, inner) in items.iter().enumerate() {
                pad(out, indent + 2);
                write_ordered(out, inner, indent + 2);
                if index + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            pad(out, indent);
            out.push(']');
        }
    }
}

fn pad(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn escape_json(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}
