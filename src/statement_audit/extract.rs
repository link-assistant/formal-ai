use std::path::Path;

use super::model::{Claim, RepositoryCorpus, SourceKind, SourceLocation};

const REGISTRY: &str = include_str!("../../data/meta/statement-audit.lino");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExtractedStatement {
    pub text: String,
    pub location: SourceLocation,
    pub claim: Option<Claim>,
}

#[derive(Debug, Default)]
struct RequirementRegistry {
    required: Vec<String>,
    forbidden: Vec<String>,
    resolution: String,
}

fn registry() -> RequirementRegistry {
    let mut registry = RequirementRegistry::default();
    for line in REGISTRY.lines().map(str::trim) {
        let Some((key, raw_value)) = line.split_once(' ') else {
            continue;
        };
        let value = raw_value.trim().trim_matches('"').to_owned();
        match key {
            "requirement_required" => registry.required.push(value),
            "requirement_forbidden" => registry.forbidden.push(value),
            "resolution_action" => registry.resolution = value,
            _ => {}
        }
    }
    registry
}

pub(super) fn proposed_resolution() -> String {
    registry().resolution
}

/// Convert a durable natural-language directive into an exclusive claim.
#[must_use]
pub fn requirement_claim(text: &str) -> Option<Claim> {
    let normalized = trim_statement(text).to_lowercase();
    let registry = registry();
    for (surfaces, value) in [
        (&registry.forbidden, "forbidden"),
        (&registry.required, "required"),
    ] {
        for surface in surfaces {
            if let Some(subject) = strip_surface(&normalized, surface) {
                return Some(Claim::exclusive(subject, "requirement_state", value));
            }
        }
    }
    None
}

fn strip_surface(text: &str, surface: &str) -> Option<String> {
    let remainder = text.strip_prefix(surface)?;
    if surface.is_ascii()
        && !remainder.is_empty()
        && !remainder.starts_with(char::is_whitespace)
        && surface.chars().last().is_some_and(char::is_alphanumeric)
        && remainder.chars().next().is_some_and(char::is_alphanumeric)
    {
        return None;
    }
    let subject = trim_statement(remainder);
    (!subject.is_empty()).then_some(subject)
}

pub(super) fn extract_corpus(corpus: &RepositoryCorpus) -> Vec<ExtractedStatement> {
    let mut extracted = Vec::new();
    for document in &corpus.documents {
        if is_prose_path(&document.path) {
            extract_prose(&document.path, &document.content, &mut extracted);
        } else if is_structured_path(&document.path) {
            extract_structured(&document.path, &document.content, &mut extracted);
        } else if is_code_path(&document.path) {
            extract_code_comments(&document.path, &document.content, &mut extracted);
        }
    }
    extracted
}

fn extract_prose(path: &str, content: &str, extracted: &mut Vec<ExtractedStatement>) {
    let mut in_fence = false;
    for (index, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        let tilde_fence = trimmed.starts_with("~~~");
        let tick_fence = trimmed.as_bytes().get(..3) == Some(&[96, 96, 96]);
        if tilde_fence || tick_fence {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.starts_with('#') || trimmed.starts_with('|') {
            continue;
        }
        let text = strip_list_marker(trimmed);
        push_statement(path, index + 1, SourceKind::Prose, text, None, extracted);
    }
}

fn extract_structured(path: &str, content: &str, extracted: &mut Vec<ExtractedStatement>) {
    for (index, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim().trim_end_matches(',');
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
            continue;
        }
        let pair = trimmed.split_once('=').or_else(|| trimmed.split_once(':'));
        let Some((raw_key, raw_value)) = pair else {
            continue;
        };
        let key = raw_key.trim().trim_matches('"');
        let value = trim_statement(raw_value.trim().trim_matches('"'));
        if key.is_empty() || value.is_empty() || matches!(value.as_str(), "{" | "[") {
            continue;
        }
        let claim = Claim::exclusive(path, key, value);
        push_statement(
            path,
            index + 1,
            SourceKind::Structured,
            trimmed,
            Some(claim),
            extracted,
        );
    }
}

fn extract_code_comments(path: &str, content: &str, extracted: &mut Vec<ExtractedStatement>) {
    let mut in_block = false;
    for (index, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        let comment = if in_block {
            let ends = trimmed.contains("*/") || trimmed.contains("-->");
            let text = trimmed
                .trim_start_matches('*')
                .split("*/")
                .next()
                .unwrap_or_default();
            in_block = !ends;
            Some(text)
        } else if let Some(text) = leading_comment(trimmed) {
            Some(text)
        } else if let Some(text) = trimmed.strip_prefix("/*") {
            in_block = !text.contains("*/");
            Some(text.split("*/").next().unwrap_or_default())
        } else if let Some(text) = trimmed.strip_prefix("<!--") {
            in_block = !text.contains("-->");
            Some(text.split("-->").next().unwrap_or_default())
        } else {
            None
        };
        if let Some(text) = comment {
            push_statement(
                path,
                index + 1,
                SourceKind::CodeComment,
                text,
                None,
                extracted,
            );
        }
    }
}

fn leading_comment(line: &str) -> Option<&str> {
    for marker in ["///", "//!", "//", "#", "--", ";"] {
        if let Some(text) = line.strip_prefix(marker) {
            return Some(text);
        }
    }
    None
}

fn push_statement(
    path: &str,
    line: usize,
    kind: SourceKind,
    raw_text: &str,
    explicit_claim: Option<Claim>,
    extracted: &mut Vec<ExtractedStatement>,
) {
    let text = raw_text.trim().to_owned();
    if text.is_empty() {
        return;
    }
    let claim = explicit_claim
        .or_else(|| requirement_claim(&text))
        .or_else(|| path_claim(&text));
    extracted.push(ExtractedStatement {
        text,
        location: SourceLocation {
            path: path.to_owned(),
            line,
            kind,
        },
        claim,
    });
}

fn path_claim(text: &str) -> Option<Claim> {
    text.split_whitespace().find_map(|token| {
        let candidate = token.trim_matches(|character: char| {
            matches!(
                character,
                '.' | ',' | ';' | ':' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\''
            )
        });
        (candidate.contains('/')
            && !candidate.contains("://")
            && Path::new(candidate).extension().is_some())
        .then(|| Claim::exclusive(candidate, "path_exists", "true"))
    })
}

fn strip_list_marker(text: &str) -> &str {
    text.strip_prefix("- ")
        .or_else(|| text.strip_prefix("* "))
        .or_else(|| text.strip_prefix("+ "))
        .unwrap_or(text)
}

fn trim_statement(text: &str) -> String {
    text.trim()
        .trim_end_matches(['.', ',', ';', ':'])
        .trim()
        .to_owned()
}

fn extension(path: &str) -> &str {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
}

fn is_prose_path(path: &str) -> bool {
    matches!(extension(path), "md" | "markdown" | "txt" | "rst" | "adoc")
}

fn is_structured_path(path: &str) -> bool {
    matches!(extension(path), "toml" | "yaml" | "yml" | "json")
}

fn is_code_path(path: &str) -> bool {
    matches!(
        extension(path),
        "rs" | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "py"
            | "rb"
            | "go"
            | "java"
            | "kt"
            | "kts"
            | "c"
            | "h"
            | "cc"
            | "cpp"
            | "hpp"
            | "cs"
            | "swift"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "sql"
            | "lua"
            | "html"
            | "css"
            | "scss"
    )
}
