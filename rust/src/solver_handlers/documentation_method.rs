//! Resolve a documentation selector against the publisher's Sphinx pages.
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::rule_interpreter::{handler_matches, handler_table_rows, handler_table_value};
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};
use crate::web_formalize::{PageBlock, formalize_page_with_context};
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::LazyLock;

const HANDLER: &str = "docs_method_explanation";
const MAXIMUM_CAPTURE_BYTES: usize = 2 * 1024 * 1024;
const MAXIMUM_EXCERPT_WORDS: usize = 24;
const MAXIMUM_EXCERPT_CHARACTERS: usize = 360;
static SELECTOR_WORDS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[A-Za-z_][A-Za-z0-9_.]*|[\p{L}\p{M}]+").unwrap());

fn valid_symbol(symbol: &str, minimum_parts: usize) -> bool {
    symbol.len() <= 160
        && symbol.split('.').count() >= minimum_parts
        && symbol.split('.').all(|part| {
            let mut bytes = part.bytes();
            bytes
                .next()
                .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
                && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
}

pub(crate) fn operands(prompt: &str) -> Vec<String> {
    let words: Vec<&str> = SELECTOR_WORDS
        .find_iter(prompt)
        .map(|word| word.as_str().trim_end_matches('.'))
        .collect();
    for (project, _) in handler_table_rows("documentation-method-providers") {
        let Some(at) = words.iter().position(|word| {
            word.split('.')
                .next()
                .is_some_and(|first| first.eq_ignore_ascii_case(project))
        }) else {
            continue;
        };
        if let Some((_, suffix)) = words[at].split_once('.') {
            let symbol = format!("{project}.{suffix}");
            if valid_symbol(&symbol, 2) {
                return vec![symbol, project.clone()];
            }
            continue;
        }
        if let Some((first, rest)) = words
            .get(at + 1)
            .filter(|word| valid_symbol(word, 2))
            .and_then(|word| word.split_once('.'))
        {
            let type_name = handler_table_value(
                &format!("{project}-documentation-type-aliases"),
                &first.to_ascii_lowercase(),
            )
            .unwrap_or(first);
            return vec![format!("{project}.{type_name}.{rest}"), project.clone()];
        }
        let nouns = crate::seed::lexicon().words_for_role(crate::seed::ROLE_CODE_METHOD_NOUN);
        let Some(noun_at) = words.iter().position(|word| {
            let word = word.to_lowercase();
            nouns.iter().any(|noun| {
                word == *noun
                    || (noun
                        .chars()
                        .any(|ch| ('\u{3400}'..='\u{9fff}').contains(&ch))
                        && word.contains(noun.as_str()))
            })
        }) else {
            continue;
        };
        let before = noun_at.checked_sub(1).and_then(|index| words.get(index));
        let after = words.get(noun_at + 1);
        if let Some(bare) = [before, after].into_iter().flatten().find(|word| {
            !word.eq_ignore_ascii_case(project) && !word.contains('.') && valid_symbol(word, 1)
        }) {
            return vec![(*bare).to_owned(), project.clone()];
        }
    }
    Vec::new()
}

struct Provider {
    root: String,
    path: String,
}

impl Provider {
    fn for_project(project: &str) -> Option<Self> {
        let declaration = handler_table_value("documentation-method-providers", project)?;
        let values = crate::seed::parser::split_pipe_list(&declaration);
        let [root, path] = values.as_slice() else {
            return None;
        };
        Some(Self {
            root: root.clone(),
            path: path.clone(),
        })
    }

    fn page(&self, symbol: &str) -> String {
        format!("{}{}/{symbol}.html", self.root, self.path)
    }

    fn candidates(&self, bytes: &[u8], project: &str, selector: &str) -> Option<Vec<String>> {
        if bytes.len() > MAXIMUM_CAPTURE_BYTES {
            return None;
        }
        let index = std::str::from_utf8(bytes)
            .ok()?
            .trim()
            .strip_prefix("Search.setIndex(")?;
        let json = index.trim_end_matches(';').strip_suffix(')')?;
        #[derive(serde::Deserialize)]
        struct SearchIndex {
            docnames: Vec<String>,
        }
        let index: SearchIndex = serde_json::from_str(json).ok()?;
        let prefix = format!("{}/{project}.", self.path);
        let selector = selector.to_ascii_lowercase();
        let qualified = selector.contains('.');
        let mut matches = BTreeSet::new();
        for name in index.docnames {
            if !name.starts_with(&prefix) {
                continue;
            }
            let symbol = &name[self.path.len() + 1..];
            if !valid_symbol(symbol, 2) {
                continue;
            }
            let lower = symbol.to_ascii_lowercase();
            if (qualified && lower == selector)
                || (!qualified && lower.rsplit('.').next() == Some(selector.as_str()))
            {
                matches.insert(symbol.to_owned());
            }
            if matches.len() == 5 {
                break;
            }
        }
        Some(matches.into_iter().collect())
    }
}

fn details(bytes: &[u8], symbol: &str) -> Option<(String, String)> {
    if bytes.len() > MAXIMUM_CAPTURE_BYTES || std::str::from_utf8(bytes).is_err() {
        return None;
    }
    let (_, blocks) = formalize_page_with_context(bytes, Some("text/html"), None);
    let short = symbol.split_once('.')?.1;
    let mut blocks = blocks.into_iter().map(|(_, block)| block);
    blocks.find(|block| {
        matches!(block, PageBlock::Heading { level: 1, text }
            if text.trim().trim_end_matches(['#', '¶']).trim() == symbol)
    })?;

    let mut signature = String::new();
    let mut summaries = Vec::new();
    let mut word_budget = MAXIMUM_EXCERPT_WORDS;
    for block in blocks {
        match block {
            PageBlock::Heading { .. } => break,
            PageBlock::ListItem { text } => {
                let term = text
                    .split("[source]")
                    .next()?
                    .trim()
                    .trim_end_matches(['#', '¶'])
                    .trim();
                let is_signature = [short, symbol].into_iter().any(|name| {
                    term.strip_prefix(name)
                        .is_some_and(|tail| tail.starts_with('('))
                });
                if is_signature && term.chars().count() <= MAXIMUM_EXCERPT_CHARACTERS {
                    if signature.is_empty() {
                        signature = term.to_owned();
                    }
                } else if !signature.is_empty() {
                    break;
                }
            }
            PageBlock::Paragraph { text, .. } if !signature.is_empty() => {
                let words: Vec<&str> = text.split_whitespace().collect();
                if words.is_empty() || text.chars().count() > MAXIMUM_EXCERPT_CHARACTERS {
                    continue;
                }
                if words.len() > word_budget {
                    break;
                }
                summaries.push(words.join(" "));
                word_budget -= words.len();
                if summaries.len() == 2 {
                    break;
                }
            }
            _ => {}
        }
    }
    (!signature.is_empty() && !summaries.is_empty()).then(|| (signature, summaries.join(" ")))
}

pub(crate) fn try_with_offline(
    prompt: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    let client = CachedSourceClient::new(
        crate::coding::synthesis_runtime::source_cache_root(),
        CurlSourceTransport,
    )
    .with_online(!offline);
    try_with_client(prompt, log, &client)
}

/// Resolve a documentation request against a caller-supplied capture client.
pub fn try_with_client<T: SourceTransport>(
    prompt: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    if !handler_matches(HANDLER, prompt) {
        return None;
    }
    let operands = operands(prompt);
    let [selector, project] = operands.as_slice() else {
        return None;
    };
    let provider = Provider::for_project(project)?;
    log.append("docs_method:request", prompt);
    log.append("docs_method:project", project.as_str());
    let language = crate::language::detect_prose(prompt)
        .unwrap_or_else(|| crate::language::detect(prompt))
        .slug();
    let acquire_page = |symbol: &str, log: &mut EventLog| {
        let source = provider.page(symbol);
        let capture = match client.fetch(&source) {
            Ok(capture) => capture,
            Err(error) => {
                log.append("docs_method:source_failure", error.to_string());
                return None;
            }
        };
        let Some((signature, excerpt)) = details(capture.bytes(), symbol) else {
            log.append("docs_method:source_failure", "symbol_or_summary_mismatch");
            return None;
        };
        capture.record(log);
        log.append("docs_method:method", symbol);
        log.append("docs_method:source_kind", "official-docs");
        log.append("docs_method:source", source.clone());
        let answer = crate::seed::render_localized_once(
            "documentation-method-source-guidance",
            language,
            &[
                ("method", symbol),
                ("signature", &signature),
                ("excerpt", &excerpt),
                ("source", &source),
            ],
        );
        Some(crate::solver_handlers::finalize_simple(
            prompt,
            log,
            HANDLER,
            "response:docs_method_explanation",
            &answer,
            0.9,
        ))
    };
    if selector.contains('.')
        && let Some(answer) = acquire_page(selector, log)
    {
        return Some(answer);
    }

    let index_source = format!("{}searchindex.js", provider.root);
    let matches = match client.fetch(&index_source) {
        Ok(capture) => {
            let matches = provider
                .candidates(capture.bytes(), project, selector)
                .unwrap_or_default();
            if !matches.is_empty() {
                capture.record(log);
            }
            matches
        }
        Err(error) => {
            log.append("docs_method:source_failure", error.to_string());
            Vec::new()
        }
    };
    if let [symbol] = matches.as_slice()
        && symbol != selector
        && let Some(answer) = acquire_page(symbol, log)
    {
        return Some(answer);
    }
    log.append("docs_method:method", selector.as_str());
    let answer = if matches.len() > 1 {
        log.append("docs_method:source", index_source);
        crate::seed::render_localized_once(
            "documentation-method-ambiguous",
            language,
            &[("method", selector), ("candidates", &matches.join(", "))],
        )
    } else {
        crate::seed::render_localized_once(
            HANDLER,
            language,
            &[("project", project), ("method", selector)],
        )
    };
    Some(crate::solver_handlers::finalize_simple(
        prompt,
        log,
        HANDLER,
        "response:docs_method_explanation",
        &answer,
        0.0,
    ))
}
