//! Universal Links Notation seed shared by every formal-ai interface.
//!
//! `data/seed/*.lino` is the canonical source of truth for the agent's
//! multilingual responses, concept knowledge base, tool registry, language
//! detection rules, prompt-question patterns, and metadata. The browser
//! worker, the Rust library, the CLI, the HTTP server, and the Telegram bot
//! all read from the same files.
//!
//! In the browser the files are fetched at runtime by `seed_loader.js`. In
//! Rust they are compiled into the binary with [`include_str!`] so even
//! offline builds expose the same data. The two implementations stay
//! consistent through `scripts/sync-seed.sh`, which mirrors `data/seed/` into
//! `src/web/seed/` for GitHub Pages deployment.
//!
//! See `VISION.md` and `REQUIREMENTS.md` (R97-R104) for the universal
//! data-driven configuration goal.
//!
//! # Stability
//!
//! The parser is intentionally tiny — Links Notation files in this repo are
//! shallow trees of `name "value"` lines with two-space indentation. The
//! schema for each category is documented in the corresponding `.lino` file.

use std::collections::BTreeMap;

/// Embedded copy of every Links Notation seed file. Returned in declaration
/// order so callers can render the merged bundle deterministically.
pub fn seed_files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("data/seed/agent-info.lino", AGENT_INFO_LINO),
        (
            "data/seed/multilingual-responses.lino",
            MULTILINGUAL_RESPONSES_LINO,
        ),
        ("data/seed/concepts.lino", CONCEPTS_LINO),
        ("data/seed/tools.lino", TOOLS_LINO),
        (
            "data/seed/language-detection.lino",
            LANGUAGE_DETECTION_LINO,
        ),
        ("data/seed/prompt-patterns.lino", PROMPT_PATTERNS_LINO),
        ("data/seed/greetings.lino", GREETINGS_LINO),
        ("data/seed/identity.lino", IDENTITY_LINO),
        (
            "data/seed/hello-world-programs.lino",
            HELLO_WORLD_PROGRAMS_LINO,
        ),
        ("data/seed/demo-dialogs.lino", DEMO_DIALOGS_LINO),
    ]
}

/// Merge every embedded seed file into a single Links Notation document with
/// the `formal_ai_seed_bundle` header. The output is exactly what the browser
/// `Download bundle` action produces minus the user-specific event log: it
/// represents the AI's static knowledge surface, fully portable in one file.
pub fn merged_bundle() -> String {
    let mut out = String::new();
    out.push_str("formal_ai_seed_bundle\n");
    for (name, contents) in seed_files() {
        out.push_str("  file \"");
        out.push_str(&escape_value(name));
        out.push_str("\"\n");
        for line in contents.lines() {
            if line.is_empty() {
                continue;
            }
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// A single response variant for an intent in a particular language.
#[derive(Debug, Clone)]
pub struct ResponseRecord {
    pub id: String,
    pub intent: String,
    pub language: String,
    pub text: String,
}

/// Parse `multilingual-responses.lino` into structured records.
pub fn multilingual_responses() -> Vec<ResponseRecord> {
    let tree = parse_lino(MULTILINGUAL_RESPONSES_LINO);
    let mut out = Vec::new();
    if let Some(root) = tree.children.first() {
        for entry in root.children.iter().filter(|c| c.name == "response") {
            let intent = entry.find_child_value("intent").to_string();
            let language = entry.find_child_value("language").to_string();
            let text = entry.find_child_value("text").to_string();
            if intent.is_empty() || language.is_empty() {
                continue;
            }
            out.push(ResponseRecord {
                id: entry.id.clone(),
                intent,
                language,
                text,
            });
        }
    }
    out
}

/// Look up a localized response by intent and language, returning `None` if
/// the seed has no matching record.
pub fn response_for(intent: &str, language: &str) -> Option<String> {
    for record in multilingual_responses() {
        if record.intent == intent && record.language == language {
            return Some(record.text);
        }
    }
    None
}

/// Generic key/value config from `agent-info.lino`.
pub fn agent_info() -> BTreeMap<String, String> {
    let tree = parse_lino(AGENT_INFO_LINO);
    let mut out = BTreeMap::new();
    if let Some(root) = tree.children.first() {
        for entry in root.children.iter().filter(|c| c.name == "field") {
            let key = entry.id.clone();
            let value = entry.find_child_value("value").to_string();
            if !key.is_empty() {
                out.insert(key, value);
            }
        }
    }
    out
}

/// A Unicode-range based language detection rule.
#[derive(Debug, Clone)]
pub struct LanguageRule {
    pub id: String,
    pub language: String,
    pub label: String,
    pub start: u32,
    pub end: u32,
}

pub fn language_rules() -> Vec<LanguageRule> {
    let tree = parse_lino(LANGUAGE_DETECTION_LINO);
    let mut out = Vec::new();
    if let Some(root) = tree.children.first() {
        for entry in root.children.iter().filter(|c| c.name == "rule") {
            let language = entry.find_child_value("language").to_string();
            if language.is_empty() {
                continue;
            }
            out.push(LanguageRule {
                id: entry.id.clone(),
                language,
                label: entry.find_child_value("label").to_string(),
                start: parse_codepoint(entry.find_child_value("start")),
                end: parse_codepoint(entry.find_child_value("end")),
            });
        }
    }
    out
}

/// A multilingual question pattern for routing intents.
#[derive(Debug, Clone)]
pub struct PromptPattern {
    pub id: String,
    pub intent: String,
    pub language: String,
    pub kind: String,
    pub text: String,
}

pub fn prompt_patterns() -> Vec<PromptPattern> {
    let tree = parse_lino(PROMPT_PATTERNS_LINO);
    let mut out = Vec::new();
    if let Some(root) = tree.children.first() {
        for entry in root.children.iter().filter(|c| c.name == "pattern") {
            let text = entry.find_child_value("text").to_string();
            if text.is_empty() {
                continue;
            }
            out.push(PromptPattern {
                id: entry.id.clone(),
                intent: entry.find_child_value("intent").to_string(),
                language: entry.find_child_value("language").to_string(),
                kind: entry.find_child_value("kind").to_string(),
                text,
            });
        }
    }
    out
}

/// Raw embedded contents (used by `merged_bundle` and by tests).
pub const AGENT_INFO_LINO: &str = include_str!("../data/seed/agent-info.lino");
pub const MULTILINGUAL_RESPONSES_LINO: &str =
    include_str!("../data/seed/multilingual-responses.lino");
pub const CONCEPTS_LINO: &str = include_str!("../data/seed/concepts.lino");
pub const TOOLS_LINO: &str = include_str!("../data/seed/tools.lino");
pub const LANGUAGE_DETECTION_LINO: &str =
    include_str!("../data/seed/language-detection.lino");
pub const PROMPT_PATTERNS_LINO: &str = include_str!("../data/seed/prompt-patterns.lino");
pub const GREETINGS_LINO: &str = include_str!("../data/seed/greetings.lino");
pub const IDENTITY_LINO: &str = include_str!("../data/seed/identity.lino");
pub const HELLO_WORLD_PROGRAMS_LINO: &str =
    include_str!("../data/seed/hello-world-programs.lino");
pub const DEMO_DIALOGS_LINO: &str = include_str!("../data/seed/demo-dialogs.lino");

/// Minimal Links Notation parser: shallow indented tree of `name "value"`
/// lines. Two-space indentation, no comments, no escapes beyond `\"` and
/// `\n` and `\\`.
#[derive(Debug, Default, Clone)]
pub struct LinoNode {
    pub name: String,
    pub id: String,
    pub children: Vec<LinoNode>,
}

impl LinoNode {
    fn find_child_value(&self, name: &str) -> &str {
        for child in &self.children {
            if child.name == name {
                return &child.id;
            }
        }
        ""
    }
}

fn parse_lino(text: &str) -> LinoNode {
    let mut root = LinoNode::default();
    // Stack entries are `(indent, path)`; `path` is the sequence of child
    // indices from `root`. Root has an empty path and indent -1.
    let mut stack: Vec<(i32, Vec<usize>)> = vec![(-1, Vec::new())];
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|c| *c == ' ').count() as i32;
        let content = &line[indent as usize..];
        let node = parse_lino_line(content);
        while stack.len() > 1 && stack.last().map(|s| s.0).unwrap_or(-1) >= indent {
            stack.pop();
        }
        let parent_path = stack.last().map(|s| s.1.clone()).unwrap_or_default();
        let parent = navigate_mut(&mut root, &parent_path);
        parent.children.push(node);
        let new_index = parent.children.len() - 1;
        let mut new_path = parent_path;
        new_path.push(new_index);
        stack.push((indent, new_path));
    }
    root
}

fn navigate_mut<'a>(root: &'a mut LinoNode, path: &[usize]) -> &'a mut LinoNode {
    let mut current = root;
    for &index in path {
        current = &mut current.children[index];
    }
    current
}

fn parse_lino_line(content: &str) -> LinoNode {
    let mut node = LinoNode::default();
    if let Some(quote_start) = content.find(" \"") {
        node.name = content[..quote_start].trim().to_string();
        let rest = &content[quote_start + 2..];
        if let Some(close) = find_closing_quote(rest) {
            node.id = unescape_value(&rest[..close]);
        }
    } else {
        node.name = content.trim().to_string();
    }
    node
}

fn find_closing_quote(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn unescape_value(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut iter = raw.chars();
    while let Some(c) = iter.next() {
        if c == '\\' {
            match iter.next() {
                Some('n') => out.push('\n'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn escape_value(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

fn parse_codepoint(value: &str) -> u32 {
    let trimmed = value.trim();
    if let Some(stripped) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        u32::from_str_radix(stripped, 16).unwrap_or(0)
    } else {
        trimmed.parse::<u32>().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_files_are_present_and_non_empty() {
        let files = seed_files();
        assert!(files.len() >= 10);
        for (name, contents) in files {
            assert!(!contents.trim().is_empty(), "{name} should not be empty");
        }
    }

    #[test]
    fn multilingual_responses_contain_four_languages() {
        let records = multilingual_responses();
        let languages: std::collections::BTreeSet<String> =
            records.iter().map(|r| r.language.clone()).collect();
        for expected in ["en", "ru", "hi", "zh"] {
            assert!(
                languages.contains(expected),
                "expected language {expected} in seed",
            );
        }
        let intents: std::collections::BTreeSet<String> =
            records.iter().map(|r| r.intent.clone()).collect();
        for expected in ["greeting", "identity", "unknown"] {
            assert!(
                intents.contains(expected),
                "expected intent {expected} in seed",
            );
        }
    }

    #[test]
    fn response_for_returns_known_text() {
        let greeting = response_for("greeting", "en").expect("english greeting");
        assert!(greeting.contains("Hi"), "got {greeting}");
        let identity = response_for("identity", "ru").expect("russian identity");
        assert!(identity.contains("formal-ai"));
    }

    #[test]
    fn agent_info_exposes_expected_keys() {
        let info = agent_info();
        for key in ["name", "supported_languages", "default_language"] {
            assert!(info.contains_key(key), "missing key {key} in agent_info");
        }
        assert_eq!(info.get("name").map(String::as_str), Some("formal-ai"));
    }

    #[test]
    fn language_rules_cover_ru_hi_zh() {
        let rules = language_rules();
        let languages: std::collections::BTreeSet<String> =
            rules.iter().map(|r| r.language.clone()).collect();
        for expected in ["ru", "hi", "zh"] {
            assert!(
                languages.contains(expected),
                "expected language rule for {expected}",
            );
        }
        for rule in rules.iter().filter(|r| r.language != "en") {
            assert!(rule.start > 0 && rule.end >= rule.start);
        }
    }

    #[test]
    fn prompt_patterns_have_intents() {
        let patterns = prompt_patterns();
        let intents: std::collections::BTreeSet<String> =
            patterns.iter().map(|p| p.intent.clone()).collect();
        assert!(intents.contains("concept_lookup"));
        assert!(intents.contains("greeting"));
    }

    #[test]
    fn merged_bundle_includes_every_file_name() {
        let bundle = merged_bundle();
        assert!(bundle.starts_with("formal_ai_seed_bundle"));
        for (name, _) in seed_files() {
            assert!(bundle.contains(name), "bundle missing entry for {name}");
        }
    }
}
