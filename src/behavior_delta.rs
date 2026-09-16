//! The #701 criterion, generalized: a learned item must demonstrably change the
//! next answer, and the change must be an observation rather than a claim
//! (plan 07 Architecture).
//!
//! The contract is identical for every learning pipeline, which is the point:
//! six pipelines each had their own idea of what "it worked" meant, and five of
//! them never checked the next answer at all.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::engine::stable_id;
use crate::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use crate::links_format::format_lino_record;
use crate::method_registry::MethodRegistry;

/// The five languages the adoption contract requires evidence in.
pub const ADOPTION_LANGUAGES: [&str; 5] = ["en", "ru", "hi", "zh", "es"];

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
    /// Stable slug used in the adoption ledger.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::Improved => "improved",
            Self::Regressed => "regressed",
            Self::ChangedUnverified => "changed_unverified",
        }
    }

    /// Only `Improved` may count toward adoption.
    ///
    /// `ChangedUnverified` is the honest middle: the answer did change, and
    /// nothing on either side could check whether it changed for the better.
    /// Counting it would turn "we do not know" into "it worked".
    #[must_use]
    pub const fn supports_adoption(self) -> bool {
        matches!(self, Self::Improved)
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
    /// The content address of one delta: the item, the language and the prompt.
    /// No wall clock, so the same seeds reproduce the same id.
    #[must_use]
    pub fn identity(item_id: &str, language: &str, prompt: &str) -> String {
        stable_id("behavior_delta", &format!("{item_id}:{language}:{prompt}"))
    }

    /// Whether the two observed answers differ at all.
    #[must_use]
    pub fn answer_changed(&self) -> bool {
        self.before.observed_output_sha256 != self.after.observed_output_sha256
    }

    /// Links Notation projection, appended to the adoption ledger.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let pairs: Vec<(&str, String)> = vec![
            ("record_type", String::from("behavior_delta")),
            ("item_id", self.item_id.clone()),
            ("item_kind", self.item_kind.clone()),
            ("language", self.language.clone()),
            ("prompt", self.prompt.clone()),
            ("before", self.before.evidence_id.clone()),
            ("after", self.after.evidence_id.clone()),
            ("before_sha256", self.before.observed_output_sha256.clone()),
            ("after_sha256", self.after.observed_output_sha256.clone()),
            ("answer_changed", self.answer_changed().to_string()),
            ("verdict", self.verdict.slug().to_owned()),
        ];
        format_lino_record(&self.delta_id, &pairs)
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
        if !self.regressions().is_empty() {
            return false;
        }
        let covered = self.languages_covered();
        ADOPTION_LANGUAGES
            .iter()
            .all(|language| covered.contains(language))
    }

    /// The languages that carry at least one adoption-supporting delta.
    ///
    /// A language with only `Unchanged` or `ChangedUnverified` deltas is not
    /// covered, which is what lets the report name what was missing instead of
    /// counting rows.
    #[must_use]
    pub fn languages_covered(&self) -> Vec<&str> {
        let mut covered: Vec<&str> = Vec::new();
        for delta in &self.deltas {
            if delta.verdict.supports_adoption() && !covered.contains(&delta.language.as_str()) {
                covered.push(delta.language.as_str());
            }
        }
        covered
    }

    /// Every regression, preserved rather than dropped: one anywhere blocks.
    #[must_use]
    pub fn regressions(&self) -> Vec<&BehaviorDelta> {
        self.deltas
            .iter()
            .filter(|delta| delta.verdict == DeltaVerdict::Regressed)
            .collect()
    }

    /// Links Notation projection: the verdict, the covered and missing
    /// languages, and every delta behind them.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let covered = self.languages_covered();
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", String::from("adoption_effect")),
            ("item_id", self.item_id.clone()),
            ("item_kind", self.item_kind.clone()),
            ("qualifies", self.qualifies().to_string()),
            ("delta_count", self.deltas.len().to_string()),
            ("regression_count", self.regressions().len().to_string()),
        ];
        for language in &covered {
            pairs.push(("language_covered", (*language).to_owned()));
        }
        for missing in ADOPTION_LANGUAGES
            .iter()
            .filter(|language| !covered.contains(language))
        {
            pairs.push(("language_missing", (*missing).to_owned()));
        }
        for delta in &self.deltas {
            pairs.push(("delta", delta.delta_id.clone()));
        }
        let identity = stable_id(
            "adoption_effect",
            &format!("{}:{}", self.item_kind, self.item_id),
        );
        let mut out = format_lino_record(&identity, &pairs);
        for delta in &self.deltas {
            out.push('\n');
            out.push_str(&delta.to_links_notation());
        }
        out
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
    let deltas = held_out
        .iter()
        .map(|(language, prompt)| {
            let before = observe(without_item, item_id, language, prompt, false);
            let after = observe(with_item, item_id, language, prompt, true);
            let verdict = if before.observed_output_sha256 == after.observed_output_sha256 {
                DeltaVerdict::Unchanged
            } else {
                // The answer changed, and nothing here can say whether it changed
                // for the better: that judgement belongs to the expectation the
                // caller attaches to the prompt. Until one is attached the honest
                // verdict is the one that never supports adoption.
                DeltaVerdict::ChangedUnverified
            };
            BehaviorDelta {
                delta_id: BehaviorDelta::identity(item_id, language, prompt),
                item_id: item_id.to_owned(),
                item_kind: item_kind.to_owned(),
                language: (*language).to_owned(),
                prompt: (*prompt).to_owned(),
                before,
                after,
                verdict,
            }
        })
        .collect();
    AdoptionEffect {
        item_id: item_id.to_owned(),
        item_kind: item_kind.to_owned(),
        deltas,
    }
}

/// Answer one held-out prompt against one registry and record what was observed.
///
/// The observation is the ordered method selection the registry produces for the
/// prompt's relevants -- the thing a learned method can change, and deterministic
/// for a given seed. Hashing it makes "the answer changed" a digest comparison
/// over bytes that were actually produced, never a claim.
fn observe(
    registry: &MethodRegistry,
    item_id: &str,
    language: &str,
    prompt: &str,
    item_present: bool,
) -> Evidence {
    let mut relevants = vec![format!("language:{language}"), format!("prompt:{prompt}")];
    if item_present {
        relevants.push(format!("method:{item_id}"));
    }
    let ordered = registry.ordered_method_names_for_relevants(&relevants);
    let observed = ordered.join("\u{1e}");
    let command = format!("method_selection:{language}");
    Evidence::observed(
        command,
        relevants,
        Some(0),
        observed.as_bytes(),
        ObservationKind::SymbolicCheck,
        EvidenceSource::Engine,
    )
}
