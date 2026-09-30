//! The internet as formal knowledge (issue #1163, E128).
//!
//! Web search exists, but its results stay "observations": strings the
//! solver shows and forgets. This module is the generic page formalizer that
//! turns any fetched resource -- HTML, Markdown, plain text, JSON, or
//! PDF-extracted text -- into a [`LinkNetwork`] the solver can reason over:
//! headings become section links, paragraphs become statement links, lists
//! and tables become list and row links, and every code block becomes a code
//! node tagged with its language.
//!
//! All behavior that could differ between deployments lives in
//! `data/seed/page-formalization-rules.lino` (mime hints, sniffing order,
//! language-tag resolution, shebang and extension heuristics, command
//! verbs) and `data/seed/source-trust-weights.lino` (the trust features and
//! their weights). The Rust here only interprets those rules.
//!
//! The six bespoke extractors in `concept_lookup.rs` stay as faster paths;
//! [`covers_statements`] proves the generic formalizer yields every
//! statement they yield on the same bytes, so each is a cache, not a
//! dependency.
//!
//! Nothing here fetches anything: bytes arrive through the existing
//! `source_fetch` capture boundary, and [`annotate_capture`] records the
//! rediscovery procedure (query, rank, URL, hash, timestamp) so a cached
//! network can be dropped under storage pressure and re-fetched
//! deterministically.
#![cfg(feature = "meta-language")]

use std::collections::BTreeMap;

use meta_language::{Link, LinkId, LinkNetwork};
use serde_json::Value;

use crate::seed::parser::parse_lino;
use crate::source_fetch::SourceCapture;

/// The formalization rules seed, mirrored into the embedded bundle.
const RULES_TEXT: &str = include_str!("../embedded/data/seed/page-formalization-rules.lino");
/// The trust-weight seed, mirrored into the embedded bundle.
const TRUST_TEXT: &str = include_str!("../embedded/data/seed/source-trust-weights.lino");

/// Mime categories the formalizer recognises; `Unknown` falls back to
/// plain-text parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageMime {
    /// Tagged markup.
    Html,
    /// Fence-and-heading markup.
    Markdown,
    /// Unstructured prose.
    PlainText,
    /// A JSON payload.
    Json,
    /// Text already extracted from a PDF.
    PdfText,
    /// Anything else; parsed as plain text.
    Unknown,
}

impl PageMime {
    /// Resolve the mime kind from a `Content-Type` style hint, falling back
    /// to plain text (sniffing needs the bytes; see
    /// [`PageMime::from_hint_or_sniff`]).
    #[must_use]
    pub fn from_hint(hint: Option<&str>) -> Self {
        let rules = FormalizationRules::load();
        Self::from_hint_or_sniff(hint, &[], &rules)
    }

    /// [`PageMime::from_hint`] with the payload available: a missing or
    /// unrecognized hint is resolved by the seed's sniff rules over the
    /// first bytes.
    #[must_use]
    pub fn from_hint_or_sniff(hint: Option<&str>, bytes: &[u8], rules: &FormalizationRules) -> Self {
        if let Some(hint) = hint {
            let normalized = hint.split(';').next().unwrap_or(hint).trim().to_ascii_lowercase();
            for mime in &rules.mime_hints {
                if mime.hint == normalized {
                    return mime.kind();
                }
            }
        }
        sniff(rules, bytes)
    }
}

/// One structural block of a page, before it becomes a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageBlock {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        text: String,
        command: bool,
    },
    ListItem {
        text: String,
    },
    TableRow {
        cells: String,
    },
    CodeBlock {
        language: String,
        text: String,
    },
}

/// The seed-driven rules the formalizer interprets (issue #1163).
#[derive(Debug, Clone)]
struct FormalizationRules {
    mime_hints: Vec<MimeHint>,
    sniffs: Vec<SniffRule>,
    tag_sources: Vec<String>,
    tag_fallback: String,
    class_prefixes: Vec<String>,
    shebangs: Vec<(String, String)>,
    extensions: Vec<(String, String)>,
    command_verbs: Vec<String>,
}

/// `hint text/html` / `kind html` pair from the seed.
#[derive(Debug, Clone)]
struct MimeHint {
    hint: String,
    kind: String,
}

/// One sniff rule: a byte-shape test and the kind it implies.
#[derive(Debug, Clone)]
struct SniffRule {
    lower_prefix: Option<String>,
    first_char: Option<String>,
    line_start: Option<String>,
    kind: String,
}

impl FormalizationRules {
    fn load() -> Self {
        let tree = parse_lino(RULES_TEXT);
        let mut rules = Self {
            mime_hints: Vec::new(),
            sniffs: Vec::new(),
            tag_sources: Vec::new(),
            tag_fallback: "unknown".to_owned(),
            class_prefixes: Vec::new(),
            shebangs: Vec::new(),
            extensions: Vec::new(),
            command_verbs: Vec::new(),
        };
        for record in &tree.children {
            match record.name.as_str() {
                "mime" => {
                    let hint = record.find_child_value("hint").to_owned();
                    let kind = record.find_child_value("kind").to_owned();
                    if !hint.is_empty() && !kind.is_empty() {
                        rules.mime_hints.push(MimeHint { hint, kind });
                    }
                }
                "sniff" => {
                    let kind = record.find_child_value("kind").to_owned();
                    if kind.is_empty() {
                        continue;
                    }
                    rules.sniffs.push(SniffRule {
                        lower_prefix: non_empty(record.find_child_value("lower_prefix")),
                        first_char: non_empty(record.find_child_value("first_char")),
                        line_start: non_empty(record.find_child_value("line_start")),
                        kind,
                    });
                }
                "language_tag" => {
                    for child in &record.children {
                        match child.name.as_str() {
                            "tag_source" if !child.id.is_empty() => {
                                rules.tag_sources.push(child.id.clone());
                            }
                            "html_class_prefix" if !child.id.is_empty() => {
                                rules.class_prefixes.push(child.id.clone());
                            }
                            _ => {}
                        }
                    }
                    let fallback = record.find_child_value("fallback").to_owned();
                    if !fallback.is_empty() {
                        rules.tag_fallback = fallback;
                    }
                }
                "shebang" => {
                    let token = record.find_child_value("token").to_owned();
                    let language = record.find_child_value("language").to_owned();
                    if !token.is_empty() && !language.is_empty() {
                        rules.shebangs.push((token, language));
                    }
                }
                "extension" => {
                    let suffix = record.find_child_value("suffix").to_owned();
                    let language = record.find_child_value("language").to_owned();
                    if !suffix.is_empty() && !language.is_empty() {
                        rules.extensions.push((suffix, language));
                    }
                }
                "command_verb" => {
                    if !record.id.is_empty() {
                        rules.command_verbs.push(record.id.clone());
                    }
                }
                _ => {}
            }
        }
        rules
    }
}

/// A field value that is absent in the seed stays absent in the rule.
fn non_empty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

impl MimeHint {
    fn kind(&self) -> PageMime {
        kind_of(&self.kind)
    }
}

/// Map the seed's kind spelling onto [`PageMime`].
fn kind_of(kind: &str) -> PageMime {
    match kind {
        "html" => PageMime::Html,
        "markdown" => PageMime::Markdown,
        "json" => PageMime::Json,
        "pdf_text" => PageMime::PdfText,
        "plain_text" => PageMime::PlainText,
        _ => PageMime::Unknown,
    }
}

/// Apply the seed's sniff rules to the payload bytes.
fn sniff(rules: &FormalizationRules, bytes: &[u8]) -> PageMime {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]);
    let lower_head = head.to_ascii_lowercase();
    for rule in &rules.sniffs {
        let matched = rule
            .lower_prefix
            .as_ref()
            .is_some_and(|prefix| lower_head.starts_with(prefix.as_str()))
            || rule
                .first_char
                .as_ref()
                .is_some_and(|character| head.starts_with(character.as_str()))
            || rule
                .line_start
                .as_ref()
                .is_some_and(|marker| head.lines().any(|line| line.starts_with(marker.as_str())));
        if matched {
            return kind_of(&rule.kind);
        }
    }
    PageMime::Unknown
}

/// Host of a URL, lowercased without a port or leading `www.`.
#[must_use]
pub fn url_domain(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = rest.split('/').next().unwrap_or(rest);
    let host = host.split(':').next().unwrap_or(host);
    let host = host.trim().to_ascii_lowercase();
    host.strip_prefix("www.").unwrap_or(&host).to_owned()
}

/// Convert raw fetched bytes into a [`LinkNetwork`] (issue #1163 R1).
///
/// Headings become section links, paragraphs become statement links, lists
/// and tables become list and row links, and fenced code blocks become code
/// links carrying a `language` field. The mime kind comes from the hint or,
/// when the hint is missing, from the seed's sniff rules.
#[must_use]
pub fn formalize_page(bytes: &[u8], mime_hint: Option<&str>) -> LinkNetwork {
    formalize_page_with_context(bytes, mime_hint, None).0
}

/// [`formalize_page`] with a URL the language-tag heuristic may consult
/// (extension in the path). Returns the network and each block's node id,
/// so callers can index blocks without walking the link graph.
#[must_use]
pub fn formalize_page_with_context(
    bytes: &[u8],
    mime_hint: Option<&str>,
    url_hint: Option<&str>,
) -> (LinkNetwork, Vec<(LinkId, PageBlock)>) {
    let rules = FormalizationRules::load();
    let mime = PageMime::from_hint_or_sniff(mime_hint, bytes, &rules);
    let text = String::from_utf8_lossy(bytes).to_string();
    let blocks = match mime {
        PageMime::Html => html_blocks(&text, &rules),
        PageMime::Markdown => markdown_blocks(&text, &rules, url_hint),
        PageMime::Json => {
            let mut blocks = Vec::new();
            if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
                json_blocks(&value, &mut blocks);
            }
            blocks
        }
        PageMime::PlainText | PageMime::PdfText | PageMime::Unknown => plain_blocks(&text, &rules),
    };
    (blocks_to_network(&blocks, url_hint), blocks)
}

/// Build the page network from parsed blocks: a `page` root with one child
/// object per block, each carrying its text (and language/level/command
/// fields where they apply). Block node ids come back in document order.
fn blocks_to_network(blocks: &[PageBlock], url_hint: Option<&str>) -> (LinkNetwork, Vec<(LinkId, PageBlock)>) {
    let mut network = LinkNetwork::new();
    let page = network.insert_object("page");
    if let Some(url) = url_hint {
        let domain = network.insert_point(&url_domain(url));
        network.insert_field(page, "domain", domain);
    }
    let mut nodes = Vec::new();
    for block in blocks {
        let (kind, text) = match block {
            PageBlock::Heading { text, .. } => ("heading", text.as_str()),
            PageBlock::Paragraph { text, .. } => ("paragraph", text.as_str()),
            PageBlock::ListItem { text } => ("list_item", text.as_str()),
            PageBlock::TableRow { cells } => ("table_row", cells.as_str()),
            PageBlock::CodeBlock { text, .. } => ("code_block", text.as_str()),
        };
        let node = network.insert_object(kind);
        let text_id = network.insert_point(text);
        network.insert_field(node, "text", text_id);
        match block {
            PageBlock::Heading { level, .. } => {
                let level_id = network.insert_point(&level.to_string());
                network.insert_field(node, "level", level_id);
            }
            PageBlock::CodeBlock { language, .. } => {
                let language_id = network.insert_point(language);
                network.insert_field(node, "language", language_id);
            }
            PageBlock::Paragraph { command: true, .. } => {
                let flag = network.insert_point("true");
                network.insert_field(node, "command", flag);
            }
            _ => {}
        }
        network.insert_field(page, "block", node);
        nodes.push((node, block.clone()));
    }
    (network, nodes)
}

/// HTML walking without an HTML dependency: for each opening tag of
/// interest, take the text up to its matching close tag.
fn html_blocks(text: &str, rules: &FormalizationRules) -> Vec<PageBlock> {
    let lower = text.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = lower[cursor..].find('<') {
        let open = cursor + rel;
        let Some(tag_end_rel) = lower[open..].find('>') else { break };
        let tag_end = open + tag_end_rel;
        let inner_tag = &text[open + 1..tag_end];
        let closing = inner_tag.starts_with('/');
        let name = inner_tag
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if closing || name.is_empty() {
            cursor = tag_end + 1;
            continue;
        }
        let close = format!("</{name}");
        let inner_end = lower[tag_end + 1..].find(&close).map(|offset| tag_end + 1 + offset);
        let block = match name.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => inner_end.map(|end| {
                let level = name.as_bytes()[1] - b'0';
                PageBlock::Heading {
                    level,
                    text: decode_entities(&strip_tags(&text[tag_end + 1..end])),
                }
            }),
            "p" => inner_end.map(|end| paragraph_block(&strip_tags(&text[tag_end + 1..end]), rules)),
            "li" => inner_end.map(|end| PageBlock::ListItem {
                text: decode_entities(&strip_tags(&text[tag_end + 1..end])),
            }),
            "tr" => inner_end.map(|end| {
                let cells = cell_texts(&text[tag_end + 1..end]).join(" | ");
                PageBlock::TableRow {
                    cells: decode_entities(&cells),
                }
            }),
            "pre" => inner_end.map(|end| {
                let raw = &text[tag_end + 1..end];
                let attrs = tag_attr(inner_tag, "class");
                let code_attrs = raw.find("<code").and_then(|code_open| {
                    let after = &raw[code_open..];
                    let tag_close = after.find('>').map(|offset| code_open + offset)?;
                    Some(tag_attr(&raw[code_open + 1..tag_close], "class"))
                });
                PageBlock::CodeBlock {
                    language: resolve_language(rules, None, attrs.or(code_attrs), raw, None),
                    text: decode_entities(&strip_tags_keep_lines(raw)),
                }
            }),
            _ => None,
        };
        if let Some(block) = block {
            out.push(block);
        }
        cursor = inner_end.map_or(tag_end + 1, |end| end + close.len());
    }
    out
}

/// Markdown walking: ATX headings, fenced code, list items, table rows,
/// blank-line-separated paragraphs.
fn markdown_blocks(text: &str, rules: &FormalizationRules, url_hint: Option<&str>) -> Vec<PageBlock> {
    let mut out = Vec::new();
    let mut paragraph = String::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            flush_paragraph(&mut paragraph, rules, &mut out);
            let fence = trimmed.chars().next().unwrap_or('`');
            let info = trimmed.trim_start_matches(fence).trim().to_owned();
            let mut code = String::new();
            for code_line in lines.by_ref() {
                let code_trimmed = code_line.trim_start();
                if code_trimmed.starts_with(fence)
                    && code_trimmed.chars().filter(|character| *character == fence).count() >= 3
                {
                    break;
                }
                code.push_str(code_line);
                code.push('\n');
            }
            out.push(PageBlock::CodeBlock {
                language: resolve_language(rules, Some(&info), None, &code, url_hint),
                text: code.trim_end().to_owned(),
            });
            continue;
        }
        let heading_level = trimmed.chars().take_while(|character| *character == '#').count();
        if heading_level > 0 && trimmed[heading_level..].starts_with(' ') {
            flush_paragraph(&mut paragraph, rules, &mut out);
            out.push(PageBlock::Heading {
                level: heading_level.min(6) as u8,
                text: trimmed[heading_level + 1..].trim().to_owned(),
            });
            continue;
        }
        if (trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ "))
            && trimmed.len() > 2
        {
            flush_paragraph(&mut paragraph, rules, &mut out);
            out.push(PageBlock::ListItem {
                text: trimmed[2..].trim().to_owned(),
            });
            continue;
        }
        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 1 {
            flush_paragraph(&mut paragraph, rules, &mut out);
            out.push(PageBlock::TableRow {
                cells: trimmed
                    .trim_matches('|')
                    .split('|')
                    .map(str::trim)
                    .collect::<Vec<_>>()
                    .join(" | "),
            });
            continue;
        }
        if line.trim().is_empty() {
            flush_paragraph(&mut paragraph, rules, &mut out);
            continue;
        }
        paragraph.push_str(line.trim());
        paragraph.push(' ');
    }
    flush_paragraph(&mut paragraph, rules, &mut out);
    out
}

/// Walk a JSON payload: every scalar leaf becomes a statement paragraph, so
/// the definitions a bespoke extractor reads are statements here too.
fn json_blocks(value: &Value, out: &mut Vec<PageBlock>) {
    match value {
        Value::String(text) => out.push(PageBlock::Paragraph {
            text: text.clone(),
            command: false,
        }),
        Value::Number(number) => out.push(PageBlock::Paragraph {
            text: number.to_string(),
            command: false,
        }),
        Value::Bool(flag) => out.push(PageBlock::Paragraph {
            text: flag.to_string(),
            command: false,
        }),
        Value::Array(items) => {
            for item in items {
                json_blocks(item, out);
            }
        }
        Value::Object(fields) => {
            for field in fields.values() {
                json_blocks(field, out);
            }
        }
        Value::Null => {}
    }
}

/// Plain text (and PDF-extracted text): blank-line-separated paragraphs,
/// with short colon-ended single lines as headings.
fn plain_blocks(text: &str, rules: &FormalizationRules) -> Vec<PageBlock> {
    let mut out = Vec::new();
    for chunk in text.split("\n\n") {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            continue;
        }
        let lines = chunk.lines().collect::<Vec<_>>();
        if lines.len() == 1 && lines[0].ends_with(':') && lines[0].len() < 80 {
            out.push(PageBlock::Heading {
                level: 2,
                text: lines[0].trim_end_matches(':').trim().to_owned(),
            });
            continue;
        }
        out.push(paragraph_block(&lines.join(" "), rules));
    }
    out
}

/// A paragraph, tagged as a command paragraph when its first token is one
/// of the seed's command verbs.
fn paragraph_block(text: &str, rules: &FormalizationRules) -> PageBlock {
    let text = decode_entities(text);
    let trimmed = text.trim();
    let command = trimmed
        .split_whitespace()
        .next()
        .is_some_and(|first| rules.command_verbs.iter().any(|verb| verb == first));
    PageBlock::Paragraph {
        text: trimmed.to_owned(),
        command,
    }
}

/// Empty the paragraph accumulator into a block.
fn flush_paragraph(paragraph: &mut String, rules: &FormalizationRules, out: &mut Vec<PageBlock>) {
    if !paragraph.trim().is_empty() {
        out.push(paragraph_block(paragraph, rules));
    }
    paragraph.clear();
}

/// Resolve a code block's language tag in the seed's order: fence
/// annotation, HTML class prefix, then heuristic (shebang, URL extension);
/// an unrecognized block gets the seed's fallback (`unknown`), never an
/// empty tag (issue #1163 R2).
fn resolve_language(
    rules: &FormalizationRules,
    fence_info: Option<&str>,
    class_attr: Option<&str>,
    code_text: &str,
    url_hint: Option<&str>,
) -> String {
    for source in &rules.tag_sources {
        match source.as_str() {
            "fence" => {
                if let Some(info) = fence_info {
                    let token = info.split_whitespace().next().unwrap_or_default();
                    if !token.is_empty() {
                        return token.to_owned();
                    }
                }
            }
            "html_class" => {
                if let Some(class_value) = class_attr {
                    for prefix in &rules.class_prefixes {
                        if let Some(rest) = class_value
                            .find(prefix.as_str())
                            .map(|at| &class_value[at + prefix.len()..])
                        {
                            let token = rest.split_whitespace().next().unwrap_or_default();
                            let token = token
                                .strip_prefix("highlight-")
                                .unwrap_or(token)
                                .strip_prefix("source-")
                                .unwrap_or(token);
                            if !token.is_empty() {
                                return token.to_owned();
                            }
                        }
                    }
                }
            }
            "heuristic" => {
                if let Some(language) = shebang_language(rules, code_text) {
                    return language;
                }
                if let Some(url) = url_hint {
                    if let Some(language) = extension_language(rules, url) {
                        return language;
                    }
                }
            }
            _ => {}
        }
    }
    rules.tag_fallback.clone()
}

/// The seed's shebang table applied to a code block's first line.
fn shebang_language(rules: &FormalizationRules, code_text: &str) -> Option<String> {
    let first = code_text.lines().next().unwrap_or_default().trim();
    rules
        .shebangs
        .iter()
        .find(|(token, _)| first == *token)
        .map(|(_, language)| language.clone())
}

/// The seed's extension table applied to a URL's path.
fn extension_language(rules: &FormalizationRules, url: &str) -> Option<String> {
    rules
        .extensions
        .iter()
        .find(|(suffix, _)| url.contains(suffix.as_str()))
        .map(|(_, language)| language.clone())
}

/// Value of `name="..."` in a tag's attribute text, if present.
fn tag_attr(inner_tag: &str, name: &str) -> Option<String> {
    let lower = inner_tag.to_ascii_lowercase();
    let needle = format!("{name}=");
    let at = lower.find(&needle)?;
    let rest = &inner_tag[at + needle.len()..];
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let end = rest[1..].find(quote)? + 1;
        Some(rest[1..end].to_owned())
    } else {
        Some(rest.split_whitespace().next().unwrap_or_default().to_owned())
    }
}

/// Strip every tag from an HTML fragment.
fn strip_tags(fragment: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for character in fragment.chars() {
        match character {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(character),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip tags while keeping line structure, for code blocks.
fn strip_tags_keep_lines(fragment: &str) -> String {
    fragment
        .lines()
        .map(strip_tags)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Text of each `<td>`/`<th>` cell in a table row.
fn cell_texts(row: &str) -> Vec<String> {
    let lower = row.to_ascii_lowercase();
    let mut cells = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = lower[cursor..].find("<t") {
        let open = cursor + rel;
        let Some(tag_end) = lower[open..].find('>') else { break };
        let tag_end = open + tag_end;
        let name = lower[open + 1..tag_end]
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_owned();
        if name == "td" || name == "th" {
            if let Some(close) = lower[tag_end + 1..].find("</t") {
                cells.push(strip_tags(&row[tag_end + 1..tag_end + 1 + close]));
            }
        }
        cursor = tag_end + 1;
    }
    cells
}

/// Decode the handful of entities real pages use.
fn decode_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// The trust features the seed weighs (issue #1163 R7).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustFeatures {
    /// The page is on the subject's official website (Wikidata P856).
    pub official_site: bool,
    /// The page was served over HTTPS.
    pub https: bool,
    /// The registry primacy of a matching registered source.
    pub primacy: Option<String>,
    /// The page declares an open license.
    pub open_license: bool,
    /// How many independently fetched pages agree on the statement.
    pub agreement_pages: u32,
}

/// Compute a page's trust score in 0..=100 from the seed's weights. The
/// score ranks sources; it never excludes one.
#[must_use]
pub fn trust_score(features: &TrustFeatures) -> u8 {
    let tree = parse_lino(TRUST_TEXT);
    let mut weights: BTreeMap<String, f64> = BTreeMap::new();
    let mut grades: BTreeMap<String, f64> = BTreeMap::new();
    let mut agreement_scale = 1.0f64;
    for record in &tree.children {
        if record.name != "feature" {
            continue;
        }
        let name = record.find_child_value("name").to_owned();
        if name.is_empty() {
            continue;
        }
        if let Ok(weight) = record.find_child_value("weight").parse::<f64>() {
            weights.insert(name.clone(), weight);
        }
        if name == "cross_page_agreement" {
            if let Ok(scale) = record.find_child_value("agreement_scale").parse::<f64>() {
                agreement_scale = scale.max(1.0);
            }
        }
        for grade in &record.children {
            if grade.name != "grade" {
                continue;
            }
            let grade_name = grade.find_child_value("name").to_owned();
            if let Ok(factor) = grade.find_child_value("factor").parse::<f64>() {
                if !grade_name.is_empty() {
                    grades.insert(grade_name, factor);
                }
            }
        }
    }
    let mut score = 0.0f64;
    if features.official_site {
        score += weights.get("official_site").copied().unwrap_or(0.0);
    }
    if features.https {
        score += weights.get("https").copied().unwrap_or(0.0);
    }
    if let Some(primacy) = &features.primacy {
        let factor = grades.get(primacy).copied().unwrap_or(0.0);
        score += weights.get("primacy").copied().unwrap_or(0.0) * factor;
    }
    if features.open_license {
        score += weights.get("open_license").copied().unwrap_or(0.0);
    }
    let agreement = features.agreement_pages.min(agreement_scale as u32) as f64 / agreement_scale;
    score += weights.get("cross_page_agreement").copied().unwrap_or(0.0) * agreement;
    (score.clamp(0.0, 1.0) * 100.0).round() as u8
}

/// Attach a trust score and the rediscovery procedure to a just-formalized
/// network (issue #1163 R9): a `rediscovery` node carrying the query text,
/// the rank at fetch time, the URL, the SHA-256, the timestamp, and whether
/// the bytes came from the cache, so the record can be dropped under
/// storage pressure and re-fetched deterministically.
pub fn annotate_capture(network: &mut LinkNetwork, capture: &SourceCapture, query: &str, rank: u32, trust: u8) {
    let rediscovery = network.insert_object("rediscovery");
    for (label, value) in [
        ("query", query.to_owned()),
        ("rank", rank.to_string()),
        ("url", capture.source_url().to_owned()),
        ("sha256", capture.sha256().to_owned()),
        ("fetched_at", capture.fetched_at().to_owned()),
        ("cached", capture.cached().to_string()),
        ("trust", trust.to_string()),
    ] {
        let value_id = network.insert_point(&value);
        network.insert_field(rediscovery, label, value_id);
    }
}

/// Every statement text a network carries: the terms of its links.
#[must_use]
pub fn network_statements(network: &LinkNetwork) -> Vec<String> {
    let mut out = Vec::new();
    for link in network.links() {
        if let Some(term) = link.metadata().term() {
            if !term.is_empty() {
                out.push(term.to_owned());
            }
        }
    }
    out
}

/// True when every `statement` appears among the network's statements, so
/// the generic formalizer yields everything a bespoke extractor yields on
/// the same bytes (issue #1163 R3: generic ⊇ bespoke).
#[must_use]
pub fn covers_statements(network: &LinkNetwork, statements: &[String]) -> bool {
    let carried = network_statements(network);
    statements
        .iter()
        .filter(|statement| !statement.trim().is_empty())
        .all(|statement| {
            carried
                .iter()
                .any(|term| term == statement || term.contains(statement.as_str()))
        })
}

/// Statements of a freshly fetched payload, for the `generic_page_v1`
/// extractor branch in the source dispatcher.
#[must_use]
pub fn generic_page_statements(bytes: &[u8], mime_hint: Option<&str>) -> Vec<String> {
    network_statements(&formalize_page(bytes, mime_hint))
}

/// One indexed code block of a formalized page.
#[derive(Debug, Clone)]
pub struct CodeBlockIndex {
    /// The language tag the block carries.
    pub language: String,
    /// The block's text.
    pub text: String,
    /// The block's `code_block` link in the page network.
    pub link: Link,
}

/// One indexed command paragraph of a formalized page.
#[derive(Debug, Clone)]
pub struct CommandIndex {
    /// The paragraph's text.
    pub text: String,
    /// The paragraph's `paragraph` link in the page network.
    pub link: Link,
}

/// One indexed paragraph of a formalized page.
#[derive(Debug, Clone)]
pub struct ParagraphIndex {
    /// The paragraph's text.
    pub text: String,
    /// The paragraph's `paragraph` link in the page network.
    pub link: Link,
}

/// A fetched, formalized, and annotated page held in working memory.
#[derive(Debug, Clone)]
pub struct FormalizedPage {
    /// Where the bytes came from.
    pub url: String,
    /// The SHA-256 of the fetched bytes.
    pub sha256: String,
    /// The result's rank at fetch time.
    pub rank: u32,
    /// The query that found the page.
    pub query: String,
    /// The page's trust score in 0..=100.
    pub trust: u8,
    /// The formalized page network.
    pub network: LinkNetwork,
    code_blocks: Vec<CodeBlockIndex>,
    commands: Vec<CommandIndex>,
    paragraphs: Vec<ParagraphIndex>,
}

/// The working-memory key under which a formalized page is stored, encoding
/// the URL and the SHA-256 so a second solve for the same URL reuses the
/// network without a re-fetch (issue #1163 R6).
#[must_use]
pub fn page_key(url: &str, sha256: &str) -> String {
    format!("page:{sha256}:{url}")
}

impl FormalizedPage {
    /// Build a page record from fetched bytes: formalize, index the code
    /// blocks, command paragraphs, and plain paragraphs, and annotate the
    /// network with the rediscovery procedure.
    #[must_use]
    pub fn from_capture(capture: &SourceCapture, query: &str, rank: u32, trust: u8, mime_hint: Option<&str>) -> Self {
        let (mut network, nodes) =
            formalize_page_with_context(capture.bytes(), mime_hint, Some(capture.source_url()));
        annotate_capture(&mut network, capture, query, rank, trust);
        let mut code_blocks = Vec::new();
        let mut commands = Vec::new();
        let mut paragraphs = Vec::new();
        for (node, block) in nodes {
            let Some(link) = network.link(node) else { continue };
            match block {
                PageBlock::CodeBlock { language, text } => code_blocks.push(CodeBlockIndex {
                    language,
                    text,
                    link: link.clone(),
                }),
                PageBlock::Paragraph { text, command } => {
                    if command {
                        commands.push(CommandIndex {
                            text: text.clone(),
                            link: link.clone(),
                        });
                    }
                    paragraphs.push(ParagraphIndex { text, link: link.clone() });
                }
                _ => {}
            }
        }
        Self {
            url: capture.source_url().to_owned(),
            sha256: capture.sha256().to_owned(),
            rank,
            query: query.to_owned(),
            trust,
            network,
            code_blocks,
            commands,
            paragraphs,
        }
    }

    /// The store key for this page.
    #[must_use]
    pub fn key(&self) -> String {
        page_key(&self.url, &self.sha256)
    }

    /// The page's indexed code blocks.
    #[must_use]
    pub fn code_blocks(&self) -> &[CodeBlockIndex] {
        &self.code_blocks
    }

    /// The page's indexed command paragraphs.
    #[must_use]
    pub fn commands(&self) -> &[CommandIndex] {
        &self.commands
    }

    /// Whether any statement of the page mentions the phrase.
    #[must_use]
    pub fn mentions(&self, phrase: &str) -> bool {
        network_statements(&self.network)
            .iter()
            .any(|statement| statement.contains(phrase))
    }
}

/// Formalized pages held in the solver's working memory, keyed by URL and
/// SHA-256 (issue #1163 R6).
#[derive(Debug, Default)]
pub struct FormalizedPageStore {
    pages: BTreeMap<String, FormalizedPage>,
}

impl FormalizedPageStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a page under its key, replacing any earlier version.
    pub fn insert(&mut self, page: FormalizedPage) -> String {
        let key = page.key();
        self.pages.insert(key.clone(), page);
        key
    }

    /// The page stored under a key, if any.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&FormalizedPage> {
        self.pages.get(key)
    }

    /// Whether any stored page was fetched from the URL.
    #[must_use]
    pub fn contains_url(&self, url: &str) -> bool {
        self.pages.values().any(|page| page.url == url)
    }

    /// How many pages the store holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Whether the store holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// "Code blocks on `<domain>` whose text contains `<term>`": every code
    /// block link of every page fetched from the domain whose text carries
    /// the term (issue #1163 R8).
    #[must_use]
    pub fn code_blocks_on(&self, domain: &str, term: &str) -> Vec<Link> {
        self.pages
            .values()
            .filter(|page| url_domain(&page.url) == domain)
            .flat_map(|page| page.code_blocks.iter())
            .filter(|block| block.text.contains(term))
            .map(|block| block.link.clone())
            .collect()
    }

    /// "The command in the paragraph that mentions `<phrase>`": command
    /// paragraphs of every page that also carries a statement mentioning
    /// the phrase (issue #1163 R8).
    #[must_use]
    pub fn command_mentioning(&self, phrase: &str) -> Vec<Link> {
        self.pages
            .values()
            .filter(|page| page.mentions(phrase))
            .flat_map(|page| page.commands.iter())
            .map(|command| command.link.clone())
            .collect()
    }
}
