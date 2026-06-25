//! Repository-file formalization and summarization.
//!
//! This layer adapts the existing statement summarizer to whole files. It keeps
//! file metadata, optional meta-language parse evidence, and Markdown fenced
//! code blocks as separate formalized records before rendering a short prose
//! summary.

use std::fmt::Write as _;
use std::path::Path;

use meta_language::{LinkNetwork, LinkType, NetworkProjection, ParseConfiguration};

use crate::links_format::sanitize_lino_value;

use super::{
    deformalize, formalize, formalize_markdown, summarize, Statement, StatementKind,
    SummarizationConfig,
};

/// meta-language parse evidence for a repository file or embedded grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaLanguageFormalization {
    pub label: String,
    pub syntax_link_count: usize,
    pub total_link_count: usize,
    pub has_error: bool,
    pub text_preserved: bool,
}

impl MetaLanguageFormalization {
    /// `true` when the upstream grammar produced syntax links and round-tripped
    /// the source text without parse errors.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        !self.has_error && self.syntax_link_count > 0 && self.text_preserved
    }
}

/// A fenced code block or other embedded grammar inside a Markdown file.
#[derive(Debug, Clone)]
pub struct EmbeddedGrammarFormalization {
    pub language: String,
    pub line_count: usize,
    pub statement_count: usize,
    pub meta_language: Option<MetaLanguageFormalization>,
}

/// Link-native representation of one repository file prepared for summary.
#[derive(Debug, Clone)]
pub struct RepositoryFileFormalization {
    pub path: String,
    pub format: String,
    pub line_count: usize,
    pub byte_count: usize,
    pub statements: Vec<Statement>,
    pub embedded_grammars: Vec<EmbeddedGrammarFormalization>,
    pub meta_language: Option<MetaLanguageFormalization>,
}

impl RepositoryFileFormalization {
    /// Render the formalized file as compact indented Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("repository_file\n");
        push_field(&mut out, 1, "path", &self.path);
        push_field(&mut out, 1, "format", &self.format);
        push_field(&mut out, 1, "line_count", &self.line_count.to_string());
        push_field(&mut out, 1, "byte_count", &self.byte_count.to_string());
        push_field(
            &mut out,
            1,
            "statement_count",
            &self.statements.len().to_string(),
        );
        if let Some(meta) = &self.meta_language {
            push_meta_language(&mut out, 1, meta);
        }
        for statement in &self.statements {
            let _ = writeln!(out, "  statement");
            push_field(&mut out, 2, "kind", statement_kind_label(statement.kind));
            push_field(&mut out, 2, "weight", &statement.weight.to_string());
            push_field(&mut out, 2, "text", &statement.text);
        }
        for embedded in &self.embedded_grammars {
            let _ = writeln!(out, "  embedded_grammar");
            push_field(&mut out, 2, "language", &embedded.language);
            push_field(&mut out, 2, "line_count", &embedded.line_count.to_string());
            push_field(
                &mut out,
                2,
                "statement_count",
                &embedded.statement_count.to_string(),
            );
            if let Some(meta) = &embedded.meta_language {
                push_meta_language(&mut out, 2, meta);
            }
        }
        out.trim_end().to_owned()
    }
}

/// Formalize an arbitrary repository file into metadata, statements, and
/// optional embedded grammar records.
#[must_use]
pub fn formalize_repository_file(path: &str, content: &str) -> RepositoryFileFormalization {
    let format = detect_repository_file_format(path);
    let meta_language = meta_language_label_for_format(format)
        .map(|label| parse_with_meta_language(label, content));
    let embedded_grammars = if format == "markdown" {
        formalize_markdown_embedded_grammars(content)
    } else {
        Vec::new()
    };
    let mut statements = statements_for_file(path, content, format);
    if statements.is_empty() {
        statements.push(Statement::new(
            format!("{path} is an empty {} file", display_file_format(format)),
            StatementKind::Identity,
            90,
        ));
    }

    RepositoryFileFormalization {
        path: path.to_owned(),
        format: format.to_owned(),
        line_count: line_count(content),
        byte_count: content.len(),
        statements,
        embedded_grammars,
        meta_language,
    }
}

/// Summarize any repository file with the existing summarization configuration.
#[must_use]
pub fn summarize_repository_file(
    path: &str,
    content: &str,
    config: &SummarizationConfig,
) -> String {
    let formalized = formalize_repository_file(path, content);
    render_repository_file_summary(&formalized, config)
}

fn render_repository_file_summary(
    formalized: &RepositoryFileFormalization,
    config: &SummarizationConfig,
) -> String {
    let mut parts = Vec::new();
    parts.push(format!(
        "{} is a {} file with {} lines and {} bytes.",
        formalized.path,
        display_file_format(&formalized.format),
        formalized.line_count,
        formalized.byte_count
    ));
    if let Some(meta) = &formalized.meta_language {
        if meta.is_valid() {
            parts.push(format!(
                "meta-language parsed it as {} with {} syntax links.",
                meta.label, meta.syntax_link_count
            ));
        }
    }
    if !formalized.embedded_grammars.is_empty() {
        parts.push(format!(
            "It has embedded grammar blocks: {}.",
            embedded_language_list(&formalized.embedded_grammars)
        ));
    }
    let summarized = summarize(&formalized.statements, config);
    let content_summary = deformalize(&summarized);
    if !content_summary.is_empty() {
        parts.push(format!("Key content: {content_summary}"));
    }
    parts.join(" ")
}

fn statements_for_file(path: &str, content: &str, format: &str) -> Vec<Statement> {
    if format == "markdown" {
        return markdown_file_statements(content);
    }
    if is_code_format(format) {
        return code_statements(path, content, format);
    }
    if is_structured_format(format) {
        return structured_statements(path, content, format);
    }
    formalize(content)
}

fn markdown_file_statements(content: &str) -> Vec<Statement> {
    let mut statements = formalize_markdown(content);
    for statement in &mut statements {
        if looks_like_heading_fragment(&statement.text) {
            statement.weight = statement.weight.saturating_sub(15);
        }
    }
    statements
}

fn code_statements(path: &str, content: &str, format: &str) -> Vec<Statement> {
    let mut statements = vec![Statement::new(
        format!(
            "{path} is a {} source file",
            display_file_format(format).to_lowercase()
        ),
        StatementKind::Identity,
        90,
    )];
    for symbol in extract_code_symbols(content, format).into_iter().take(8) {
        statements.push(Statement::new(
            format!("Defines {symbol}."),
            StatementKind::Feature,
            70,
        ));
    }
    statements
}

fn structured_statements(path: &str, content: &str, format: &str) -> Vec<Statement> {
    let mut statements = vec![Statement::new(
        format!("{path} is a {} data file", display_file_format(format)),
        StatementKind::Identity,
        90,
    )];
    let keys = extract_structural_keys(content);
    if !keys.is_empty() {
        statements.push(Statement::new(
            format!("Top-level keys: {}.", keys.join(", ")),
            StatementKind::Feature,
            70,
        ));
    }
    statements
}

fn formalize_markdown_embedded_grammars(markdown: &str) -> Vec<EmbeddedGrammarFormalization> {
    let mut blocks = Vec::new();
    let mut active: Option<FencedBlock> = None;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if let Some(block) = active.take() {
            if is_closing_fence(trimmed, &block.marker) {
                blocks.push(formalize_fenced_block(&block));
            } else {
                let mut block = block;
                block.source.push_str(line);
                block.source.push('\n');
                active = Some(block);
            }
            continue;
        }
        if let Some(marker) = opening_fence_marker(trimmed) {
            active = Some(FencedBlock {
                marker,
                language: fence_language(trimmed),
                source: String::new(),
            });
        }
    }
    if let Some(block) = active {
        blocks.push(formalize_fenced_block(&block));
    }
    blocks
}

fn formalize_fenced_block(block: &FencedBlock) -> EmbeddedGrammarFormalization {
    let language = normalize_language_label(&block.language);
    let statements = if is_code_format(&language) {
        extract_code_symbols(&block.source, &language)
    } else {
        Vec::new()
    };
    let meta_language = meta_language_label_for_format(&language)
        .map(|label| parse_with_meta_language(label, &block.source));
    EmbeddedGrammarFormalization {
        language,
        line_count: line_count(&block.source),
        statement_count: statements.len(),
        meta_language,
    }
}

#[derive(Debug, Clone)]
struct FencedBlock {
    marker: FencedMarker,
    language: String,
    source: String,
}

#[derive(Debug, Clone, Copy)]
struct FencedMarker {
    ch: char,
    len: usize,
}

fn opening_fence_marker(trimmed_line: &str) -> Option<FencedMarker> {
    let ch = trimmed_line.chars().next()?;
    if ch != '`' && ch != '~' {
        return None;
    }
    let len = trimmed_line
        .chars()
        .take_while(|candidate| *candidate == ch)
        .count();
    (len >= 3).then_some(FencedMarker { ch, len })
}

fn is_closing_fence(trimmed_line: &str, opening: &FencedMarker) -> bool {
    let Some(closing) = opening_fence_marker(trimmed_line) else {
        return false;
    };
    if closing.ch != opening.ch || closing.len < opening.len {
        return false;
    }
    let rest = &trimmed_line[closing.len..];
    rest.trim().is_empty()
}

fn fence_language(trimmed_line: &str) -> String {
    let Some(marker) = opening_fence_marker(trimmed_line) else {
        return "text".to_owned();
    };
    let without_marker = &trimmed_line[marker.len..];
    let info_string = without_marker.trim();
    if marker.ch == '`' && info_string.contains('`') {
        "text".to_owned()
    } else {
        info_string
            .split(|ch: char| ch.is_whitespace() || matches!(ch, ',' | ';' | '{'))
            .next()
            .filter(|language| !language.is_empty())
            .unwrap_or("text")
            .to_owned()
    }
}

fn parse_with_meta_language(label: &str, source: &str) -> MetaLanguageFormalization {
    let network = LinkNetwork::parse(source, label, ParseConfiguration::default());
    let verification = network.verify_full_match(None);
    let syntax_link_count = network
        .projected_links(NetworkProjection::ConcreteSyntax)
        .filter(|link| link.metadata().link_type() == Some(LinkType::Syntax))
        .count();
    MetaLanguageFormalization {
        label: label.to_owned(),
        syntax_link_count,
        total_link_count: network.len(),
        has_error: !verification.is_clean(),
        text_preserved: network.reconstruct_text() == source,
    }
}

fn detect_repository_file_format(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    let file_name = Path::new(&lower)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    match file_name {
        "cargo.toml" | "pyproject.toml" => return "toml",
        "package.json" | "tsconfig.json" | "package-lock.json" | "bun.lock" => return "json",
        "dockerfile" => return "dockerfile",
        _ => {}
    }
    let extension = Path::new(&lower)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    match extension {
        "md" | "markdown" | "mdown" => "markdown",
        "rs" => "rust",
        "js" | "mjs" | "cjs" | "jsx" => "javascript",
        "ts" | "tsx" => "typescript",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "html" | "htm" => "html",
        "css" => "css",
        "xml" | "svg" => "xml",
        "ini" => "ini",
        "sh" | "bash" => "shell",
        "lino" => "links_notation",
        _ => "text",
    }
}

fn normalize_language_label(label: &str) -> String {
    match label.trim().to_ascii_lowercase().as_str() {
        "rs" => "rust",
        "js" | "mjs" | "cjs" | "jsx" => "javascript",
        "ts" | "tsx" => "typescript",
        "py" => "python",
        "c++" => "cpp",
        "c#" | "cs" => "csharp",
        "md" => "markdown",
        "" => "text",
        other => other,
    }
    .to_owned()
}

fn meta_language_label_for_format(format: &str) -> Option<&'static str> {
    match format {
        "rust" => Some("rust"),
        "javascript" => Some("javascript"),
        "typescript" => Some("typescript"),
        "python" => Some("python"),
        "go" => Some("go"),
        "java" => Some("java"),
        "c" => Some("c"),
        "cpp" => Some("cpp"),
        "csharp" => Some("csharp"),
        "ruby" => Some("ruby"),
        "json" => Some("json"),
        "yaml" => Some("yaml"),
        "toml" => Some("toml"),
        "html" => Some("html"),
        "css" => Some("css"),
        "xml" => Some("xml"),
        "ini" => Some("ini"),
        _ => None,
    }
}

fn is_code_format(format: &str) -> bool {
    matches!(
        format,
        "rust"
            | "javascript"
            | "typescript"
            | "python"
            | "go"
            | "java"
            | "c"
            | "cpp"
            | "csharp"
            | "ruby"
    )
}

fn is_structured_format(format: &str) -> bool {
    matches!(
        format,
        "json" | "yaml" | "toml" | "ini" | "links_notation" | "xml" | "html" | "css"
    )
}

fn display_file_format(format: &str) -> &'static str {
    match format {
        "markdown" => "Markdown",
        "rust" => "Rust",
        "javascript" => "JavaScript",
        "typescript" => "TypeScript",
        "python" => "Python",
        "go" => "Go",
        "java" => "Java",
        "c" => "C",
        "cpp" => "C++",
        "csharp" => "C#",
        "ruby" => "Ruby",
        "json" => "JSON",
        "yaml" => "YAML",
        "toml" => "TOML",
        "html" => "HTML",
        "css" => "CSS",
        "xml" => "XML",
        "ini" => "INI",
        "shell" => "shell",
        "links_notation" => "Links Notation",
        "dockerfile" => "Dockerfile",
        _ => "text",
    }
}

fn extract_code_symbols(content: &str, format: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with('*') {
            continue;
        }
        if let Some(symbol) = symbol_from_line(trimmed, format) {
            push_unique(&mut symbols, symbol);
        }
    }
    symbols
}

fn symbol_from_line(line: &str, format: &str) -> Option<String> {
    match format {
        "rust" => rust_symbol(line),
        "javascript" | "typescript" => js_symbol(line),
        "python" => prefixed_symbol(line, "def ")
            .or_else(|| prefixed_symbol(line, "class "))
            .map(|name| format!("python symbol {name}")),
        "go" => prefixed_symbol(line, "func ").map(|name| format!("go function {name}")),
        "java" | "csharp" => class_like_symbol(line),
        "c" | "cpp" => c_like_symbol(line),
        "ruby" => prefixed_symbol(line, "def ")
            .or_else(|| prefixed_symbol(line, "class "))
            .map(|name| format!("ruby symbol {name}")),
        _ => None,
    }
}

fn rust_symbol(line: &str) -> Option<String> {
    let stripped = strip_leading_words(line, &["pub", "async", "unsafe", "const"]);
    for (keyword, label) in [
        ("fn ", "function"),
        ("struct ", "struct"),
        ("enum ", "enum"),
        ("trait ", "trait"),
        ("mod ", "module"),
        ("type ", "type"),
        ("const ", "constant"),
        ("static ", "static"),
    ] {
        if let Some(name) = prefixed_symbol(stripped, keyword) {
            return Some(format!("rust {label} {name}"));
        }
    }
    stripped
        .strip_prefix("impl ")
        .and_then(first_identifier)
        .map(|name| format!("rust impl {name}"))
}

fn js_symbol(line: &str) -> Option<String> {
    let stripped = strip_leading_words(line, &["export", "default", "async"]);
    prefixed_symbol(stripped, "function ")
        .map(|name| format!("javascript function {name}"))
        .or_else(|| {
            prefixed_symbol(stripped, "class ").map(|name| format!("javascript class {name}"))
        })
        .or_else(|| {
            prefixed_symbol(stripped, "const ")
                .or_else(|| prefixed_symbol(stripped, "let "))
                .or_else(|| prefixed_symbol(stripped, "var "))
                .map(|name| format!("javascript binding {name}"))
        })
}

fn class_like_symbol(line: &str) -> Option<String> {
    let stripped = strip_leading_words(
        line,
        &[
            "public",
            "private",
            "protected",
            "internal",
            "static",
            "sealed",
        ],
    );
    prefixed_symbol(stripped, "class ")
        .map(|name| format!("class {name}"))
        .or_else(|| prefixed_symbol(stripped, "interface ").map(|name| format!("interface {name}")))
        .or_else(|| prefixed_symbol(stripped, "enum ").map(|name| format!("enum {name}")))
}

fn c_like_symbol(line: &str) -> Option<String> {
    if !line.contains('(') || line.ends_with(';') {
        return None;
    }
    let before_paren = line.split_once('(')?.0.trim_end();
    let name = before_paren.split_whitespace().last()?;
    is_identifier(name).then(|| format!("function {name}"))
}

fn prefixed_symbol(line: &str, prefix: &str) -> Option<String> {
    line.strip_prefix(prefix).and_then(first_identifier)
}

fn first_identifier(text: &str) -> Option<String> {
    let candidate: String = text
        .chars()
        .skip_while(|ch| !is_identifier_start(*ch))
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect();
    (!candidate.is_empty()).then_some(candidate)
}

fn strip_leading_words<'a>(line: &'a str, words: &[&str]) -> &'a str {
    let mut rest = line.trim_start();
    loop {
        let mut changed = false;
        for word in words {
            if let Some(after_word) = rest.strip_prefix(word) {
                if after_word.starts_with(char::is_whitespace) {
                    rest = after_word.trim_start();
                    changed = true;
                }
            }
        }
        if !changed {
            return rest;
        }
    }
}

fn extract_structural_keys(content: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(key) = quoted_key(trimmed).or_else(|| bare_key(trimmed)) {
            push_unique(&mut keys, key);
        }
        if keys.len() >= 8 {
            break;
        }
    }
    keys
}

fn quoted_key(line: &str) -> Option<String> {
    let rest = line.strip_prefix('"')?;
    let (key, after_key) = rest.split_once('"')?;
    after_key
        .trim_start()
        .starts_with(':')
        .then(|| key.to_owned())
}

fn bare_key(line: &str) -> Option<String> {
    let (key, _) = line.split_once([':', '='])?;
    let trimmed = key.trim();
    (!trimmed.is_empty()
        && trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.')))
    .then(|| trimmed.to_owned())
}

fn embedded_language_list(blocks: &[EmbeddedGrammarFormalization]) -> String {
    let mut languages = Vec::new();
    for block in blocks {
        push_unique(&mut languages, block.language.clone());
    }
    languages.join(", ")
}

fn push_unique(values: &mut Vec<String>, candidate: String) {
    if !values.iter().any(|value| value == &candidate) {
        values.push(candidate);
    }
}

fn line_count(content: &str) -> usize {
    if content.is_empty() {
        0
    } else {
        content.lines().count()
    }
}

fn looks_like_heading_fragment(text: &str) -> bool {
    text.split_whitespace().count() <= 6 && !has_terminal_punctuation(text)
}

fn has_terminal_punctuation(text: &str) -> bool {
    text.chars()
        .last()
        .is_some_and(|ch| matches!(ch, '.' | '!' | '?' | ':' | '。' | '…'))
}

fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some_and(is_identifier_start)
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

const fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

const fn statement_kind_label(kind: StatementKind) -> &'static str {
    match kind {
        StatementKind::Identity => "identity",
        StatementKind::Purpose => "purpose",
        StatementKind::Language => "language",
        StatementKind::Stars => "stars",
        StatementKind::Feature => "feature",
        StatementKind::UseCase => "use_case",
        StatementKind::Install => "install",
        StatementKind::Example => "example",
        StatementKind::Misc => "misc",
    }
}

fn push_meta_language(out: &mut String, indent: usize, meta: &MetaLanguageFormalization) {
    write_indent(out, indent);
    let _ = writeln!(out, "meta_language");
    push_field(out, indent + 1, "label", &meta.label);
    push_field(
        out,
        indent + 1,
        "syntax_link_count",
        &meta.syntax_link_count.to_string(),
    );
    push_field(
        out,
        indent + 1,
        "total_link_count",
        &meta.total_link_count.to_string(),
    );
    push_field(out, indent + 1, "has_error", &meta.has_error.to_string());
    push_field(
        out,
        indent + 1,
        "text_preserved",
        &meta.text_preserved.to_string(),
    );
}

fn push_field(out: &mut String, indent: usize, name: &str, value: &str) {
    write_indent(out, indent);
    let _ = writeln!(out, "{name} {}", sanitize_lino_value(value));
}

fn write_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}
