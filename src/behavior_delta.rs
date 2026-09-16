//! The #701 criterion, generalized: a learned item must demonstrably change the
//! next answer, and the change must be an observation rather than a claim
//! (plan 07 Architecture).
//!
//! Wave T skeleton: the shapes the tests name exist, the behaviour does not.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::execution_evidence::Evidence;
use crate::method_registry::MethodRegistry;

/// What one learned item did to one prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaVerdict {
    /// The answer is byte-identical with and without the item: it changed nothing.
    Unchanged,
    /// The answer changed and the after side satisfies its expectation.
    Improved,
    /// The answer changed and the after side fails an expectation the before side met.
    Regressed,
    /// The answer changed and neither side has a checkable expectation. Honest,
    /// and never sufficient for adoption.
    ChangedUnverified,
}

impl DeltaVerdict {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        panic!("plan 07 leaf 2 -- DeltaVerdict slugs")
    }

    /// Only `Improved` may count toward adoption.
    #[must_use]
    pub const fn supports_adoption(self) -> bool {
        panic!("plan 07 leaf 2 -- only Improved supports adoption")
    }
}

/// One before/after observation for one learned item on one held-out prompt.
///
/// Both sides are `Evidence`, so "the answer changed" is a hash comparison over
/// observed bytes, not a prose judgement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorDelta {
    /// `stable_id("behavior_delta", "<item_id>:<language>:<prompt>")`.
    pub delta_id: String,
    /// The learned item under test, e.g. a `LearnedMethod::name`.
    pub item_id: String,
    /// Which learning pipeline produced it: `method`, `request_opener`,
    /// `program_rule`, `repair_lesson`, `amendment`, `anticipation`.
    pub item_kind: String,
    /// BCP-47 tag: `en`, `ru`, `hi`, `zh`, `es`.
    pub language: String,
    /// The held-out prompt, never one the item was inferred from.
    pub prompt: String,
    /// The answer observed with the item absent.
    pub before: Evidence,
    /// The answer observed with the item present.
    pub after: Evidence,
    pub verdict: DeltaVerdict,
}

impl BehaviorDelta {
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 07 leaf 2 -- BehaviorDelta links notation")
    }
}

/// Every delta proved for one learned item, and whether they are enough.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptionEffect {
    pub item_id: String,
    pub item_kind: String,
    pub deltas: Vec<BehaviorDelta>,
}

impl AdoptionEffect {
    /// The adoption contract, identical for every pipeline: at least one
    /// `Improved` delta in each of `en`, `ru`, `hi`, `zh`, `es`; zero
    /// `Regressed` deltas anywhere; every prompt held out from inference.
    #[must_use]
    pub fn qualifies(&self) -> bool {
        todo!("plan 07 leaf 2 -- the adoption contract")
    }

    #[must_use]
    pub fn languages_covered(&self) -> Vec<&str> {
        todo!("plan 07 leaf 2 -- languages covered")
    }

    #[must_use]
    pub fn regressions(&self) -> Vec<&BehaviorDelta> {
        todo!("plan 07 leaf 2 -- one regression anywhere blocks")
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 07 leaf 2 -- AdoptionEffect links notation")
    }
}

/// Prove the effect of one learned item by answering each held-out prompt twice:
/// once against a registry with the item removed, once with it present.
/// Deterministic: same seed data, same prompts, same deltas.
#[must_use]
pub fn prove_effect(
    item_id: &str,
    item_kind: &str,
    held_out: &[(&str, &str)],
    with_item: &MethodRegistry,
    without_item: &MethodRegistry,
) -> AdoptionEffect {
    let _ = (item_id, item_kind, held_out, with_item, without_item);
    todo!("plan 07 leaf 2 -- prove the effect by answering each prompt twice")
}
