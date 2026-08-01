use std::path::Path;

use crate::engine::stable_id;

use super::model::{Claim, RepositoryCorpus, SourceKind, SourceLocation};

const REGISTRY: &str = include_str!("../../data/meta/statement-audit.lino");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExtractedStatement {
    pub id: String,
    pub text: String,
    pub resolved_text: String,
    pub location: SourceLocation,
    pub claim: Option<Claim>,
    pub references: Vec<ExtractedReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExtractedReference {
    pub surface: String,
    pub antecedent_statement_id: String,
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
    resolve_document_references(&mut extracted);
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
    let location = SourceLocation {
        path: path.to_owned(),
        line,
        kind,
    };
    extracted.push(ExtractedStatement {
        id: statement_id(&location, &text),
        resolved_text: text.clone(),
        text,
        location,
        claim,
        references: Vec::new(),
    });
}

fn statement_id(location: &SourceLocation, text: &str) -> String {
    stable_id(
        "audited_statement",
        &format!("{}:{}:{}", location.path, location.line, text),
    )
}

#[derive(Debug, Clone)]
struct Referent {
    subject: String,
    statement_id: String,
}

fn resolve_document_references(statements: &mut [ExtractedStatement]) {
    let mut document = String::new();
    let mut referent: Option<Referent> = None;
    let mut previous_statement_complete = false;
    for statement in statements {
        if statement.location.path != document {
            document.clone_from(&statement.location.path);
            referent = None;
            previous_statement_complete = false;
        }
        if statement.location.kind != SourceKind::Prose {
            continue;
        }

        let reference = leading_reference(&statement.text);
        let resolution = previous_statement_complete
            .then(|| reference.zip(referent.as_ref()))
            .flatten();
        if let Some(((surface, possessive), antecedent)) = resolution {
            let replacement = if possessive {
                possessive_form(&antecedent.subject)
            } else {
                antecedent.subject.clone()
            };
            statement.resolved_text = format!("{replacement}{}", &statement.text[surface.len()..]);
            statement.references.push(ExtractedReference {
                surface: surface.to_owned(),
                antecedent_statement_id: antecedent.statement_id.clone(),
            });
            referent = Some(Referent {
                subject: antecedent.subject.clone(),
                statement_id: statement.id.clone(),
            });
        } else if reference.is_some() {
            // A leading reference after an unterminated Markdown line is most
            // likely a soft-wrapped continuation. Do not invent an antecedent,
            // and do not let an older referent leak past the ambiguous line.
            referent = None;
        } else if let Some(subject) = extract_subject(&statement.text) {
            referent = Some(Referent {
                subject,
                statement_id: statement.id.clone(),
            });
        }
        previous_statement_complete = ends_sentence(&statement.text);
    }
}

fn ends_sentence(text: &str) -> bool {
    text.trim_end()
        .trim_end_matches(['"', '\'', ')', ']', '*', '_', '`'])
        .chars()
        .next_back()
        .is_some_and(|character| matches!(character, '.' | '!' | '?'))
}

fn leading_reference(text: &str) -> Option<(&str, bool)> {
    const REFERENCES: [(&str, bool, bool); 12] = [
        ("themselves", false, false),
        ("itself", false, false),
        ("theirs", true, false),
        ("their", true, false),
        ("they", false, false),
        ("them", false, false),
        ("its", true, false),
        ("it", false, false),
        ("these", false, true),
        ("those", false, true),
        ("this", false, true),
        ("that", false, true),
    ];
    REFERENCES
        .iter()
        .find_map(|(reference, possessive, demonstrative)| {
            let surface = text.get(..reference.len())?;
            let remainder = text.get(reference.len()..)?.trim_start();
            (surface.eq_ignore_ascii_case(reference)
                && text
                    .as_bytes()
                    .get(reference.len())
                    .is_some_and(u8::is_ascii_whitespace)
                && (!demonstrative || begins_with_predicate(remainder)))
            .then_some((surface, *possessive))
        })
}

fn begins_with_predicate(text: &str) -> bool {
    const PREDICATES: [&str; 16] = [
        "is ", "are ", "was ", "were ", "has ", "have ", "does ", "do ", "can ", "cannot ",
        "must ", "may ", "might ", "will ", "should ", "would ",
    ];
    let lower = text.to_ascii_lowercase();
    PREDICATES
        .iter()
        .any(|predicate| lower.starts_with(predicate))
}

fn possessive_form(subject: &str) -> String {
    if subject.ends_with('s') || subject.ends_with('S') {
        format!("{subject}'")
    } else {
        format!("{subject}'s")
    }
}

fn extract_subject(text: &str) -> Option<String> {
    const PREDICATES: [&str; 26] = [
        " is ",
        " are ",
        " was ",
        " were ",
        " has ",
        " have ",
        " had ",
        " does ",
        " do ",
        " did ",
        " can ",
        " cannot ",
        " must ",
        " may ",
        " might ",
        " will ",
        " should ",
        " would ",
        " uses ",
        " use ",
        " contains ",
        " includes ",
        " exposes ",
        " supports ",
        " remains ",
        " requires ",
    ];
    if leading_reference(text).is_some() {
        return None;
    }
    let lower = text.to_ascii_lowercase();
    let boundary = PREDICATES
        .iter()
        .filter_map(|predicate| lower.find(predicate))
        .min()?;
    let subject = text[..boundary]
        .trim()
        .trim_matches(|character| matches!(character, '*' | '_' | '`' | '[' | ']'));
    let word_count = subject.split_whitespace().count();
    (word_count > 0 && word_count <= 12).then(|| subject.to_owned())
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
