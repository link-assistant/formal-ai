//! Shared grammar-backed readable Links codec; JSON decodes only scalar text.
use meta_language::{FeatureParseOptions, LeafText, SyntaxTree};
use serde_json::{Value, json};

const GRAMMAR: &str = include_str!("../../embedded/data/meta/source-model/readable-links.grammar");
const DIAGNOSTICS: &str = include_str!("../../embedded/data/meta/source-model/diagnostics.json");

fn diagnostic(key: &str, values: &[(&str, String)]) -> String {
    let messages: Value = serde_json::from_str(DIAGNOSTICS).expect("InvalidSourceDiagnosticData");
    let mut text = messages[key]
        .as_str()
        .expect("MissingSourceDiagnostic")
        .to_owned();
    for (name, value) in values {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

fn decode(tree: &SyntaxTree, depth: usize) -> Result<Option<Value>, String> {
    if depth > 512 {
        return Err("SourceModelDepthExceeded".to_owned());
    }
    match tree {
        SyntaxTree::Node { kind, children, .. } if matches!(kind.as_str(), "document" | "link") => {
            let mut values = Vec::new();
            for child in children {
                if let Some(value) = decode(child, depth + 1)? {
                    values.push(value);
                }
            }
            Ok(Some(Value::Array(values)))
        }
        SyntaxTree::Token {
            kind,
            text: LeafText::Text(text),
            ..
        } => match kind.as_deref() {
            Some("quoted") => serde_json::from_str::<String>(text)
                .map(|text| Some(json!({"quoted": text})))
                .map_err(|error| error.to_string()),
            Some("word") => Ok(Some(Value::String(text.clone()))),
            Some("layout") | None => Ok(None),
            _ => Err("UnsupportedSourceNotationToken".to_owned()),
        },
        _ => Err("UnrepresentedSourceNotation".to_owned()),
    }
}

pub(super) fn parse(notation: &str) -> Result<Vec<Value>, String> {
    if notation.len() > 64 * 1024 * 1024 {
        return Err("SourceModelTooLarge".to_owned());
    }
    let grammar =
        meta_language::parse_native_grammar(GRAMMAR).map_err(|error| error.to_string())?;
    let options = FeatureParseOptions::default();
    let parser =
        meta_language::compile_feature_grammar(&grammar, None, FeatureParseOptions::default())
            .map_err(|error| error.to_string())?;
    let outcome = parser
        .parse_tree(notation.as_bytes(), &options)
        .map_err(|error| error.to_string())?;
    if !outcome.ok {
        if let Some(rejection) = outcome.rejection {
            return Err(diagnostic(
                "rejected-source-notation",
                &[
                    ("reason", rejection.reason.to_string()),
                    ("offset", format!("{:?}", rejection.offset)),
                    ("line", format!("{:?}", rejection.line)),
                    ("column", format!("{:?}", rejection.column)),
                    ("limit", format!("{:?}", rejection.limit)),
                ],
            ));
        }
        return Err(diagnostic("rejection-detail-unavailable", &[]));
    }
    let tree = outcome
        .tree
        .ok_or_else(|| "SourceNotationTreeRequired".to_owned())?;
    let decoded = decode(&tree, 0)?.ok_or_else(|| "SourceNotationDocumentRequired".to_owned())?;
    decoded
        .as_array()
        .cloned()
        .ok_or_else(|| "SourceNotationDocumentRequired".to_owned())
}

pub(super) fn write(value: &Value, depth: usize) -> Result<String, String> {
    if depth > 512 {
        return Err("SourceModelDepthExceeded".to_owned());
    }
    if let Some(word) = value.as_str() {
        if word.is_empty()
            || !word.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
        {
            return Err("InvalidSourceConcept".to_owned());
        }
        return Ok(word.to_owned());
    }
    if let Some(text) = value
        .as_object()
        .filter(|object| object.len() == 1)
        .and_then(|object| object.get("quoted"))
        .and_then(Value::as_str)
    {
        return serde_json::to_string(text).map_err(|error| error.to_string());
    }
    let values = value
        .as_array()
        .ok_or_else(|| "UnexpectedSourcePayload".to_owned())?;
    let pieces = values
        .iter()
        .map(|value| write(value, depth + 1))
        .collect::<Result<Vec<_>, _>>()?;
    let indent = "  ".repeat(depth + 1);
    let compact = values.iter().skip(1).all(|value| !value.is_array())
        || values.iter().any(|value| {
            value
                .as_array()
                .is_some_and(|fields| fields.first().and_then(Value::as_str) == Some("text"))
        });
    Ok(if compact {
        format!("({})", pieces.join(" "))
    } else {
        let (head, tail) = pieces
            .split_first()
            .ok_or_else(|| "EmptySourceNode".to_owned())?;
        format!("({head}\n{indent}{})", tail.join(&format!("\n{indent}")))
    })
}
