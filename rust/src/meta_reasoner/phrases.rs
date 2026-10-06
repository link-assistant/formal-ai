//! The trace vocabulary: the detail templates the JavaScript reasoner writes
//! into its numbered events, one constant per template literal, so the Rust
//! derivation is byte-identical to the JavaScript one. They are machine
//! notation of the derivation (event kind + detail), not answer prose: the
//! reader-facing sentences come from the seed's `response` records.
//!
//! `{}` marks a slot; [`fill`] fills the slots in order.

/// `${indent}cycle ${chain}` (`metaGround`).
pub const CYCLE: &str = "{}cycle {}";
/// `${indent}${word} := "${definition}" (request definition)` (`metaGround`).
pub const REQUEST_DEFINITION: &str = "{}{} := \"{}\" (request definition)";
/// `${indent}${word} → ${operation} (learned from ${via})` (`metaGround`).
pub const RECALL: &str = "{}{} → {} (learned from {})";
/// `${indent}${word} → ${top} (documentation)` (`metaGround`).
pub const DOCUMENTED: &str = "{}{} → {} (documentation)";
/// `${indent}understand ${word} through ${n} gloss(es)` (`metaGround`).
pub const UNDERSTAND: &str = "{}understand {} through {} gloss(es)";
/// `${indent}${word} → ${operation} via ${via}` (`metaGround`).
pub const GROUNDED_VIA: &str = "{}{} → {} via {}";
/// ` acting on ${objectType}` (`metaClauses`).
pub const ACTING_ON: &str = " acting on {}";
/// `; modifiers {${others}}` (`metaClauses`).
pub const MODIFIERS: &str = "; modifiers {{{}}}";
/// `head {${head}}` (`metaClauses`).
pub const CLAUSE_HEAD: &str = "head {{{}}}{}{}";
/// `none` (`metaReasonCore`).
pub const NONE: &str = "none";
/// `${n} sub-goals: ${list}` (`metaReasonComposite`).
pub const DECOMPOSE: &str = "{} sub-goals: {}";
/// `${index}) ${request}` (`metaReasonComposite`).
pub const SUBGOAL_ITEM: &str = "{}) {}";
/// `${request} → ${goal} ${status}${program}` (`metaReasonComposite`).
pub const SUBGOAL: &str = "{} → {} {}{}";
/// ` (${steps})` (`metaReasonComposite`).
pub const SUBGOAL_PROGRAM: &str = " ({})";
/// `${input} → ${output}` (`metaReasonCore`, `metaAnswer`).
pub const ARROW: &str = "{} → {}";
/// `no probe` (`metaReasonCore`).
pub const NO_PROBE: &str = "no probe";
/// `learned ${word} → ${operation}` (`metaLearn`).
pub const LEARNED: &str = "learned {} → {}";
/// `${program}: ${input} gave ${actual}, expected ${expected} (difference ${n})`.
pub const COUNTEREXAMPLE: &str = "{}: {} gave {}, expected {} (difference {})";
/// `length ${n}: ${count} typed candidate(s), ${passing} with difference 0`.
pub const SEARCH: &str = "length {}: {} typed candidate(s), {} with difference 0";
/// `program serving ${groups}` (`metaSynthesizeFromMeaning`).
pub const MEANING_GOAL: &str = "program serving {}";
/// `lookups: ${rounds}` (`metaAnswer`).
pub const LOOKUPS: &str = "lookups: {}";
/// `${seq}. ${kind}: ${detail}` (`metaAnswer`).
pub const REASONING_LINE: &str = "{}. {}: {}";
/// `${intent} → general loop: ${goal} ${status}` (`metaResolveImpasse`).
pub const IMPASSE_HANDOFF: &str = "{} → general loop: {} {}";
/// `${goal} ${status}` (`meta_reason` step detail).
pub const GOAL_STATUS: &str = "{} {}";
/// The arrow joining a cycle's words (`metaGround`).
pub const CHAIN: &str = " → ";
/// The composition sign joining program steps (`metaDerivationLino`).
pub const COMPOSE: &str = " ∘ ";
/// The separator between alternatives in a trace detail.
pub const ALTERNATIVES: &str = " | ";

/// Fill the `{}` slots of a template in order; `{{` and `}}` are literal
/// braces.
///
/// Mirrors JavaScript template literals in js/worker/formal_ai_worker_meta_reasoner.js.
#[must_use]
pub fn fill(template: &str, slots: &[&str]) -> String {
    let mut out =
        String::with_capacity(template.len() + slots.iter().map(|slot| slot.len()).sum::<usize>());
    let mut next = slots.iter();
    let mut characters = template.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '{' if characters.peek() == Some(&'{') => {
                characters.next();
                out.push('{');
            }
            '}' if characters.peek() == Some(&'}') => {
                characters.next();
                out.push('}');
            }
            '{' if characters.peek() == Some(&'}') => {
                characters.next();
                if let Some(slot) = next.next() {
                    out.push_str(slot);
                }
            }
            other => out.push(other),
        }
    }
    out
}
