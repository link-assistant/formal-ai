//! Captured values of a data-owned handler rule and the log references that
//! read them.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use super::{Context, Rule, Subject, ValueRef, ValueSource, substitute};
use crate::seed;

/// A `table <name>` block of the rule document (issue #918): ordered
/// `row <key> <value>` pairs and the value used when no row answers.
#[derive(Debug, Default)]
pub(super) struct Table {
    pub(super) rows: Vec<(String, String)>,
    pub(super) default: Option<String>,
}

/// How a `table` value picks its row.
#[derive(Debug)]
pub(super) enum TableLookup {
    /// The first row whose key occurs in this subject text.
    Contained(Subject),
    /// The row whose key equals the earlier capture of this name.
    Key(String),
}

/// The embedded rule document's tables, parsed once.
fn tables() -> &'static BTreeMap<String, Table> {
    static CACHE: OnceLock<BTreeMap<String, Table>> = OnceLock::new();
    CACHE.get_or_init(|| super::parser::tables(seed::HANDLER_RULES_LINO))
}

/// The rows of the rule document's `table <name>` block in order, empty when it
/// declares none: the vocabulary a native-primitive handler reads from data
/// instead of a constant list of its own (issue #918).
#[must_use]
pub fn handler_table_rows(name: &str) -> &'static [(String, String)] {
    tables()
        .get(name)
        .map_or(&[], |table| table.rows.as_slice())
}

/// The value of the first row of `table <name>` whose key occurs in `text`.
#[must_use]
pub fn handler_table_row(name: &str, text: &str) -> Option<&'static str> {
    handler_table_rows(name)
        .iter()
        .find(|(key, _)| !key.is_empty() && text.contains(key.as_str()))
        .map(|(_, value)| value.as_str())
}

/// The value of the row of `table <name>` keyed exactly `key`, else the
/// table's `default`.
#[must_use]
pub fn handler_table_value(name: &str, key: &str) -> Option<&'static str> {
    let table = tables().get(name)?;
    table
        .rows
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map_or_else(
            || table.default.as_deref(),
            |(_, value)| Some(value.as_str()),
        )
}

/// Marks a captured slot sheds at its edges besides whitespace: the quotation
/// and sentence punctuation a request wraps its object in. Structural marks,
/// not words of any language.
const SLOT_EDGE_MARKS: [char; 10] = ['`', '"', '\'', ':', '-', '_', '.', ',', '?', '!'];

impl Rule {
    pub(super) fn resolve_values(&self, context: &Context<'_>) -> Option<Vec<(String, String)>> {
        let mut resolved: Vec<(String, String)> = Vec::with_capacity(self.values.len());
        for (name, source) in &self.values {
            let value = match source {
                ValueSource::Backticks => context
                    .prompt
                    .split('`')
                    .nth(1)
                    .map(str::trim)
                    .filter(|term| !term.is_empty())?
                    .to_owned(),
                ValueSource::AgentInfo { key, default } => seed::agent_info()
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| default.clone()),
                ValueSource::Literal(text) => text.clone(),
                ValueSource::TrimmedPrompt => context.prompt.trim().to_owned(),
                ValueSource::StableId(prefix) => crate::engine::stable_id(prefix, context.prompt),
                ValueSource::RoleSlot(role) => role_slot(role, context)?,
                ValueSource::Quoted => {
                    crate::solver_helpers::extract_quoted_phrase(context.prompt)?
                }
                ValueSource::NetworkSnapshot => crate::engine::knowledge_links_notation(),
                ValueSource::Operand { kind, index } => {
                    crate::capability_routing::claim_operands(kind, context.prompt)?
                        .into_iter()
                        .nth(*index)?
                }
                ValueSource::Table { table, lookup } => {
                    lookup_table(table, lookup, context, &resolved)?
                }
                ValueSource::Response(template) => {
                    let pairs: Vec<(&str, &str)> = resolved
                        .iter()
                        .map(|(name, value)| (name.as_str(), value.as_str()))
                        .collect();
                    seed::localized_response(&substitute(template, &pairs), &context.language)?
                }
            };
            resolved.push((name.clone(), value));
        }
        Some(resolved)
    }
}

impl ValueRef {
    pub(super) fn resolve(
        &self,
        context: &Context<'_>,
        values: &[(String, String)],
    ) -> Option<String> {
        match self {
            Self::Prompt => Some(context.prompt.to_owned()),
            Self::Trimmed => Some(context.prompt.trim().to_owned()),
            Self::Capture(name) => values
                .iter()
                .find(|(candidate, _)| candidate == name)
                .map(|(_, value)| value.clone()),
            Self::Literal(text) => Some(text.clone()),
        }
    }
}

/// The value a `table` block answers for one lookup; `None` when the table is
/// missing, the keying capture is absent, or no row answers and the table
/// declares no default.
fn lookup_table(
    name: &str,
    lookup: &TableLookup,
    context: &Context<'_>,
    resolved: &[(String, String)],
) -> Option<String> {
    let table = tables().get(name)?;
    let row = match lookup {
        TableLookup::Contained(subject) => {
            let text = context.text(*subject);
            table
                .rows
                .iter()
                .find(|(key, _)| !key.is_empty() && text.contains(key.as_str()))
        }
        TableLookup::Key(capture) => {
            let (_, key) = resolved.iter().find(|(captured, _)| captured == capture)?;
            table.rows.iter().find(|(candidate, _)| candidate == key)
        }
    };
    row.map(|(_, value)| value.clone())
        .or_else(|| table.default.clone())
}

/// The text filling the open slot of a role's prefix surface.
///
/// The first surface whose lead opens the normalized subject wins; failing
/// that, the first surface whose `action` is `scan` may open the slot anywhere
/// in the lowercased prompt, so an opener that follows a greeting still
/// counts. The slot sheds whitespace and [`SLOT_EDGE_MARKS`] at both edges and
/// is `None` when nothing is left. The surfaces are the lexicon's, in
/// declaration order, so no opener phrase lives here.
fn role_slot(role: &str, context: &Context<'_>) -> Option<String> {
    let forms = seed::lexicon().role_word_forms(role);
    let slot = forms
        .iter()
        .find_map(|form| context.normalized.strip_prefix(form.before_slot()))
        .or_else(|| {
            forms
                .iter()
                .filter(|form| form.action == "scan")
                .find_map(|form| {
                    context
                        .lowercase
                        .split_once(form.before_slot())
                        .map(|(_, tail)| tail)
                })
        })?;
    let trimmed = slot.trim_matches(|ch: char| ch.is_whitespace() || SLOT_EDGE_MARKS.contains(&ch));
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
