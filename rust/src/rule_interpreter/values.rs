//! Captured values of a data-owned handler rule and the log references that
//! read them.

use super::{Context, Rule, ValueRef, ValueSource};
use crate::seed;

/// Marks a captured slot sheds at its edges besides whitespace: the quotation
/// and sentence punctuation a request wraps its object in. Structural marks,
/// not words of any language.
const SLOT_EDGE_MARKS: [char; 10] = ['`', '"', '\'', ':', '-', '_', '.', ',', '?', '!'];

impl Rule {
    pub(super) fn resolve_values(&self, context: &Context<'_>) -> Option<Vec<(String, String)>> {
        let mut resolved = Vec::with_capacity(self.values.len());
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
