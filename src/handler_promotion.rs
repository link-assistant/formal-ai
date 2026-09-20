//! Which handlers a prompt hoists ahead of `data/seed/handler-precedence.lino`
//! (#1138 B9, plan 09 Architecture 2).
//!
//! The nineteen promotion predicates hard-coded in
//! `src/intent_formalization/prompt_relevants.rs` become rows of
//! `data/seed/handler-promotions.lino`, whose `when` conditions are written in
//! exactly the grammar `data/seed/handler-rules.lino` already uses and
//! `crate::rule_interpreter` already evaluates. One evaluator, two callers: a
//! promotion becomes a seed edit, never a Rust edit.
//!
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write as _;

use crate::rule_interpreter::{ConditionSource, HandlerRules, LinkStoreSource};
use crate::seed::parser::{LinoNode, parse_lino};

const PROMOTIONS_LINO: &str = include_str!("../data/seed/handler-promotions.lino");

/// One promotion row: a handler name, the rank that orders it against the other
/// promotions that fired, the condition block in the handler-rules grammar, and
/// the written reason the row exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerPromotion {
    /// The handler this row hoists, e.g. `web_search`.
    pub handler: String,
    /// Lower rank wins when two promotions fire.
    pub rank: u32,
    /// The `when` block verbatim, in the `data/seed/handler-rules.lino` grammar.
    pub when: String,
    /// Why the row exists, quoted from the issue that asked for it.
    pub because: String,
}

impl HandlerPromotion {
    /// The record as Links Notation, so a promotion round-trips through the seed.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "  promotion {}", self.handler);
        let _ = writeln!(out, "    rank {}", self.rank);
        out.push_str("    when\n");
        for line in self.when.lines() {
            let _ = writeln!(out, "      {line}");
        }
        let _ = writeln!(out, "    because {}", quoted(&self.because));
        out
    }
}

/// Every promotion the shipped seed declares, in declaration order.
#[must_use]
pub fn promotions() -> Vec<HandlerPromotion> {
    promotions_from(PROMOTIONS_LINO)
        .unwrap_or_else(|error| panic!("invalid data/seed/handler-promotions.lino: {error}"))
}

/// Parse a promotions document. Used by the tests to inject a fixture row and
/// observe routing change with no Rust edit.
///
/// # Errors
/// Returns the parse failure when the document is not a promotions document.
pub fn promotions_from(text: &str) -> Result<Vec<HandlerPromotion>, String> {
    let tree = parse_lino(text);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "handler_promotions")
        .ok_or_else(|| String::from("handler_promotions:missing_root"))?;
    let mut rows = Vec::new();
    for node in root.children.iter().filter(|node| node.name == "promotion") {
        if node.id.trim().is_empty() {
            return Err(String::from("handler_promotions:missing_handler"));
        }
        let rank = node
            .find_child_value("rank")
            .parse::<u32>()
            .map_err(|_| format!("handler_promotions:{}:invalid_rank", node.id))?;
        let when_node = node
            .children
            .iter()
            .find(|child| child.name == "when")
            .ok_or_else(|| format!("handler_promotions:{}:missing_when", node.id))?;
        let when = render_nodes(&when_node.children, 0);
        if when.trim().is_empty() {
            return Err(format!("handler_promotions:{}:empty_when", node.id));
        }
        let because = node.find_child_value("because").trim().to_owned();
        if because.is_empty() {
            return Err(format!("handler_promotions:{}:missing_because", node.id));
        }
        let row = HandlerPromotion {
            handler: node.id.clone(),
            rank,
            when,
            because,
        };
        compile_condition(&row)?;
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(String::from("handler_promotions:no_rows"));
    }
    Ok(rows)
}

/// The `handler:<name>` relevants a prompt promotes, in `rank` order.
///
/// Evaluated through the one rule interpreter reading the projected link store
/// (plan 09 leaf 40: the store, not the parsed seed tables, is the read path).
/// Keeps no handler names.
#[must_use]
pub fn promoted_relevants(promotions: &[HandlerPromotion], prompt: &str) -> Vec<String> {
    promoted_relevants_with_source(promotions, prompt, LinkStoreSource::shared())
}

/// Evaluate promotions through an explicit condition source.
#[must_use]
pub fn promoted_relevants_with_source(
    promotions: &[HandlerPromotion],
    prompt: &str,
    source: &dyn ConditionSource,
) -> Vec<String> {
    let normalized = crate::engine::normalize_prompt(prompt);
    let normalized = crate::seed::operation_vocabulary().canonicalized_prompt(&normalized);
    let mut ordered: Vec<(usize, &HandlerPromotion)> = promotions.iter().enumerate().collect();
    ordered.sort_by_key(|(declaration, row)| (row.rank, *declaration));
    let mut relevants = Vec::new();
    for (_, row) in ordered {
        let Ok(rules) = compile_condition(row) else {
            continue;
        };
        let matched = rules
            .handler("promotion")
            .is_some_and(|handler| handler.matches_with_source(source, prompt, &normalized));
        let relevant = format!("handler:{}", row.handler);
        if matched && !relevants.contains(&relevant) {
            relevants.push(relevant);
        }
    }
    relevants
}

/// Verdict for every promotion in declaration order. This is deliberately
/// separate from the ranked relevant list so parity cannot hide an untested
/// row behind the first promotion that matched.
#[must_use]
pub fn promotion_condition_verdicts(
    promotions: &[HandlerPromotion],
    prompt: &str,
    source: &dyn ConditionSource,
) -> Vec<(String, bool)> {
    let normalized = crate::engine::normalize_prompt(prompt);
    let normalized = crate::seed::operation_vocabulary().canonicalized_prompt(&normalized);
    promotions
        .iter()
        .map(|row| {
            let matched = compile_condition(row)
                .ok()
                .and_then(|rules| {
                    rules
                        .handler("promotion")
                        .map(|handler| handler.matches_with_source(source, prompt, &normalized))
                })
                .unwrap_or(false);
            (row.handler.clone(), matched)
        })
        .collect()
}

fn compile_condition(row: &HandlerPromotion) -> Result<HandlerRules, String> {
    let mut document =
        String::from("handler_rules\n  handler promotion\n    rule predicate\n      when\n");
    for line in row.when.lines() {
        let _ = writeln!(document, "        {line}");
    }
    crate::links_format::push_lino_node(&mut document, 6, "respond_unknown", None);
    HandlerRules::parse(&document)
        .map_err(|error| format!("handler_promotions:{}:{error}", row.handler))
}

fn render_nodes(nodes: &[LinoNode], depth: usize) -> String {
    let mut out = String::new();
    for node in nodes {
        for _ in 0..depth {
            out.push_str("  ");
        }
        out.push_str(&node.name);
        if !node.id.is_empty() {
            out.push(' ');
            // `parse_lino` stores every token after the node name in `id`.
            // Those tokens are the rule language (`calendar_event of padded`),
            // not one string operand, so preserve their token boundaries when
            // rebuilding the document consumed by `HandlerRules`.
            out.push_str(&node.id);
        }
        out.push('\n');
        out.push_str(&render_nodes(&node.children, depth + 1));
    }
    out
}

fn quoted(value: &str) -> String {
    if value.is_empty()
        || value
            .chars()
            .any(|character| character.is_whitespace() || character == '#')
    {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}
