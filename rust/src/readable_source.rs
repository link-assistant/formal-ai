//! Lossless readable source content. Positions and link identifiers never persist.
mod network;
mod notation;
use serde_json::{Value, json};

const RULES: &str = include_str!("../embedded/data/meta/source-model/syntax-rules.lino");

pub(super) fn field<'a>(node: &'a [Value], name: &str) -> Result<Option<&'a [Value]>, String> {
    let mut matches = node
        .iter()
        .filter_map(Value::as_array)
        .filter(|item| item.first().and_then(Value::as_str) == Some(name));
    let result = matches.next().map(Vec::as_slice);
    if matches.next().is_some() {
        return Err(format!("AmbiguousSourceField:{name}"));
    }
    Ok(result)
}

pub(super) fn field_text<'a>(node: &'a [Value], name: &str) -> Result<&'a str, String> {
    let fields = field(node, name)?.ok_or_else(|| format!("MissingSourceField:{name}"))?;
    if fields.len() != 2 {
        return Err(format!("MalformedSourceField:{name}"));
    }
    fields[1]
        .as_object()
        .filter(|object| object.len() == 1)
        .and_then(|object| object.get("quoted"))
        .and_then(Value::as_str)
        .ok_or_else(|| format!("SourceTextRequired:{name}"))
}

pub(super) fn supported(language: &str) -> bool {
    matches!(language, "JavaScript" | "TypeScript" | "Rust")
}

pub(super) fn rules() -> Result<Vec<(String, String, String)>, String> {
    let documents = notation::parse(RULES)?;
    let document = documents
        .first()
        .and_then(Value::as_array)
        .filter(|document| {
            documents.len() == 1
                && document.first().and_then(Value::as_str) == Some("source-syntax-rules")
        })
        .ok_or_else(|| "SourceSyntaxRulesRequired".to_owned())?;
    let mut rules = Vec::new();
    for row in document.iter().skip(1) {
        let row = row
            .as_array()
            .filter(|row| row.len() == 4 && row[0].as_str() == Some("parse-rule"))
            .ok_or_else(|| "UnsupportedSourceSyntaxRule".to_owned())?;
        let language = field_text(row, "language")?;
        let kind = field_text(row, "kind")?;
        let concept = field_text(row, "concept")?;
        if !supported(language)
            || rules
                .iter()
                .any(|(old_language, old_kind, _)| old_language == language && old_kind == kind)
        {
            return Err("AmbiguousSourceSyntaxRule".to_owned());
        }
        rules.push((language.to_owned(), kind.to_owned(), concept.to_owned()));
    }
    Ok(rules)
}

fn render_node(value: &Value, depth: usize) -> Result<String, String> {
    if depth > 512 {
        return Err("SourceModelDepthExceeded".to_owned());
    }
    let node = value
        .as_array()
        .filter(|node| node.first().is_some_and(Value::is_string))
        .ok_or_else(|| "MalformedSourceNode".to_owned())?;
    if node.iter().skip(1).any(|value| !value.is_array()) {
        return Err("UnexpectedSourcePayload".to_owned());
    }
    field_text(node, "form")?;
    let children = node
        .iter()
        .skip(1)
        .filter(|value| {
            !matches!(
                value
                    .as_array()
                    .and_then(|node| node.first())
                    .and_then(Value::as_str),
                Some("form" | "role" | "meaning" | "semantic-gap" | "text")
            )
        })
        .collect::<Vec<_>>();
    if field(node, "text")?.is_some() {
        if !children.is_empty() {
            return Err("AmbiguousSourceLeaf".to_owned());
        }
        return field_text(node, "text").map(str::to_owned);
    }
    children
        .into_iter()
        .map(|child| render_node(child, depth + 1))
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.concat())
}

/// Attach structural ownership without granting any read or write authority.
pub fn owned_source_links(source_path: &str, document: &Value) -> Result<Value, String> {
    source_from_links(document)?;
    let wrapped = json!([
        "owned-source",
        text_field("path", source_path),
        source_content(document)?
    ]);
    source_content(&wrapped)?;
    Ok(wrapped)
}

/// Return the validated readable owner, when the document declares one.
pub fn source_owner(document: &Value) -> Result<Option<&str>, String> {
    let Some(node) = document
        .as_array()
        .filter(|node| node.first().and_then(Value::as_str) == Some("owned-source"))
    else {
        return Ok(None);
    };
    source_content(document)?;
    field_text(node, "path").map(Some)
}

fn source_content(document: &Value) -> Result<&Value, String> {
    let Some(node) = document
        .as_array()
        .filter(|node| node.first().and_then(Value::as_str) == Some("owned-source"))
    else {
        return Ok(document);
    };
    let source_path = field_text(node, "path")?;
    if source_path.is_empty()
        || source_path.chars().next() == Some('/')
        || source_path.contains('\\')
        || source_path.contains(':')
        || source_path
            .split('/')
            .any(|part| matches!(part, "" | "." | ".."))
    {
        return Err("InvalidSourceOwner".to_owned());
    }
    let contents = node
        .iter()
        .skip(1)
        .filter(|value| {
            value
                .as_array()
                .and_then(|row| row.first())
                .and_then(Value::as_str)
                == Some("source")
        })
        .collect::<Vec<_>>();
    if node.len() != 3 || contents.len() != 1 {
        return Err("AmbiguousSourceOwnership".to_owned());
    }
    Ok(contents[0])
}

/// Reconstruct solely from recursive model text, without a source file or cache.
pub fn source_from_links(document: &Value) -> Result<String, String> {
    let document = source_content(document)?;
    let node = document
        .as_array()
        .filter(|node| node.first().and_then(Value::as_str) == Some("source"))
        .ok_or_else(|| "SourceDocumentRequired".to_owned())?;
    if !supported(field_text(node, "language")?) {
        return Err("UnsupportedSourceLanguage".to_owned());
    }
    let children = node.iter().skip(1).filter(|value| {
        value
            .as_array()
            .and_then(|row| row.first())
            .and_then(Value::as_str)
            != Some("language")
    });
    let mut contents = Vec::new();
    for child in children {
        contents.push(render_node(child, 0)?);
    }
    if contents.is_empty() {
        return Err("MissingSourceTree".to_owned());
    }
    Ok(contents.concat())
}

/// Parse the pinned grammar and retain every syntax, field and trivia text leaf.
pub fn source_to_links(source: &str, language: &str) -> Result<Value, String> {
    if !supported(language) {
        return Err("UnsupportedSourceLanguage".to_owned());
    }
    if source.len() > 4 * 1024 * 1024 {
        return Err("SourceInputTooLarge".to_owned());
    }
    network::convert(source, language, &rules()?)
}

/// Read the same readable grammar used by the JavaScript source model.
pub fn read_source_links(notation: &str) -> Result<Value, String> {
    let mut documents = notation::parse(notation)?;
    if documents.len() != 1 {
        return Err("OneSourceDocumentRequired".to_owned());
    }
    let document = documents.remove(0);
    source_from_links(&document)?;
    Ok(document)
}

/// Serialize ordinary UTF-8 text, not numeric IDs or encoded source buffers.
pub fn write_source_links(document: &Value) -> Result<String, String> {
    source_from_links(document)?;
    let notation = notation::write(document, 0)? + "\n";
    if read_source_links(&notation)? != *document {
        return Err("SourceNotationMismatch".to_owned());
    }
    Ok(notation)
}

fn concrete(value: &Value) -> Value {
    value.as_array().map_or_else(
        || value.clone(),
        |values| {
            Value::Array(
                values
                    .iter()
                    .filter(|value| {
                        !matches!(
                            value
                                .as_array()
                                .and_then(|row| row.first())
                                .and_then(Value::as_str),
                            Some("meaning" | "semantic-gap")
                        )
                    })
                    .map(concrete)
                    .collect(),
            )
        },
    )
}

/// Independently reparse restored text; stored labels cannot authenticate themselves.
pub fn verify_source_links(document: &Value) -> Result<(), String> {
    let document = source_content(document)?;
    let node = document
        .as_array()
        .ok_or_else(|| "SourceDocumentRequired".to_owned())?;
    let language = field_text(node, "language")?;
    let fresh = source_to_links(&source_from_links(document)?, language)?;
    if concrete(&fresh) != concrete(document) {
        return Err("SourceStructureMismatch".to_owned());
    }
    Ok(())
}

pub(super) fn text_field(name: &str, text: &str) -> Value {
    json!([name, {"quoted": text}])
}
