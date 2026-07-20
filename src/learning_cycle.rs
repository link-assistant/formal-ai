//! The adoption contract of the auto-learning loop (issue #701).
//!
//! Issue #558 gave the engine a *proposal-only* learner and issue #498 gave it a
//! *frontier* — the trending prompts it cannot route. What was missing is the
//! step between them: a cycle that turns a frontier item into a **candidate piece
//! of knowledge**, proves the candidate generalizes on prompts it was not derived
//! from, and emits it as a **promotion proposal in the exact shape the issue-#656
//! protocol consumes** ([`crate::promotion`]). Without that step a "learning run"
//! could only ever record its own failure, which is what
//! [`crate::google_trends_learning`] honestly reported: 60 frontier prompts, zero
//! proposals.
//!
//! The cycle is deliberately small and symbolic — no statistics, no fabrication:
//!
//! 1. **Read a recorded frontier.** Frontier items come from a committed record
//!    (`data/meta/learning-frontier-google-trends.lino`), not from a live network
//!    call, so `formal-ai learn cycle --dry-run` is deterministic, reproducible
//!    offline, and still replayable *after* its own proposals were adopted.
//! 2. **Derive a template.** Deleting the topic query from a frontier prompt
//!    leaves the request frame around it. Two frontier items of the same class
//!    must agree on that frame — a frame supported by one prompt is memorisation,
//!    not a rule.
//! 3. **Classify the slot.** Where the query sat inside the frame *is* the word
//!    order of the language: query last → [`Slot::Prefix`] frame, query first →
//!    [`Slot::Suffix`], query in the middle → [`Slot::Circumfix`]. This is why the
//!    adoption gap was structural: Hindi is verb-final and Chinese wraps its
//!    object, so a prefix-only recogniser could never route them.
//! 4. **Generate held-out tests.** The frame is derived from the first two items
//!    of a class; every *other* item of the class is a held-out test the candidate
//!    must match — same frame, and the query recovered from the prompt must be the
//!    topic the prompt was generated from.
//! 5. **Emit a promotion proposal** carrying the seed edit and the gate the
//!    held-out tests form. Nothing is applied here: promotion stays human-gated
//!    ([`crate::promotion::PromotionRun`]), and `--dry-run` is the default.
//! 6. **Preserve every failure.** A class that yields no validated candidate is
//!    not dropped: it becomes a durable blocked record naming the gap (R425).
//!
//! Validation runs on the candidate itself rather than by swapping the process's
//! lexicon, because [`crate::seed::lexicon`] is a process-global cache: the
//! recogniser's slot-matching rule is applied directly to the held-out prompts.
//! The complementary "after" evidence — the live engine answering the same prompt
//! once the surface is adopted — is recorded by
//! [`crate::learning_adoption_ledger`].

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::engine::normalize_prompt;
use crate::promotion::{PromotionProposal, PromotionRatchet, SeedEdit};
use crate::seed::parser::parse_lino;
use crate::seed::Slot;

/// The frontier slug of the Google Trends learning frontier (issues #498/#499).
pub const GOOGLE_TRENDS_FRONTIER: &str = "google-trends";

/// The committed, frozen record of the Google Trends frontier.
pub const GOOGLE_TRENDS_FRONTIER_RECORD: &str =
    include_str!("../data/meta/learning-frontier-google-trends.lino");

/// The seed file learned request openers are promoted into.
pub const LEARNED_REQUEST_OPENERS_SEED_FILE: &str = "data/seed/learned-request-openers.lino";

/// The semantic role a learned request opener is filed under. The recogniser
/// reads this role in every slot form, so a learned surface changes routing as
/// pure data.
pub const TERM_INFORMATION_ROLE: &str = "term_information_request_opener";

/// How many distinct frontier items must agree on a frame before it is a
/// candidate. One prompt can only memorise; two prompts share a rule.
pub const MINIMUM_SUPPORT: usize = 2;

/// One recorded frontier item: a prompt the engine could not route, kept with
/// the topic it was generated from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontierItem {
    /// One-based rank of the originating topic.
    pub rank: usize,
    /// The trending search term the prompt was generated from.
    pub query: String,
    /// Language tag of the prompt (`en`, `ru`, `hi`, `zh`).
    pub language: String,
    /// Stable variation key of the prompt class.
    pub variation: String,
    /// The prompt text.
    pub prompt: String,
    /// The intent recorded when the item landed on the frontier.
    pub engine_intent: String,
}

/// A held-out test generated for a candidate: a frontier prompt of the same
/// class that was *not* used to derive the frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldOutTest {
    /// The prompt under test.
    pub prompt: String,
    /// The topic the prompt was generated from — the query the candidate must
    /// recover.
    pub expected_query: String,
    /// Whether the candidate frame matched and recovered exactly that query.
    pub passed: bool,
}

/// A candidate surface derived from a frontier class and checked against
/// held-out prompts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSurface {
    /// Language tag the surface belongs to.
    pub language: String,
    /// The frontier class (prompt variation) it was derived from.
    pub variation: String,
    /// The surface text in seed notation, with `…` marking the subject slot.
    pub surface: String,
    /// Which slot form the frame occupies.
    pub slot: Slot,
    /// Topics whose prompts the frame was derived from.
    pub support: Vec<String>,
    /// Held-out prompts of the same class, none of them used for derivation.
    pub held_out: Vec<HeldOutTest>,
}

impl CandidateSurface {
    /// Whether every held-out prompt is matched and its topic recovered.
    #[must_use]
    pub fn validated(&self) -> bool {
        !self.held_out.is_empty() && self.held_out.iter().all(|test| test.passed)
    }

    /// How many held-out tests pass.
    #[must_use]
    pub fn passed_count(&self) -> usize {
        self.held_out.iter().filter(|test| test.passed).count()
    }

    /// How many held-out tests fail.
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.held_out.len() - self.passed_count()
    }
}

/// A frontier class that produced no adoptable candidate, preserved verbatim so
/// the failure is durable rather than silently dropped (R425).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedClass {
    /// Language tag of the class.
    pub language: String,
    /// Prompt variation of the class.
    pub variation: String,
    /// Named blocking gap, e.g. `insufficient_support`.
    pub reason: String,
    /// One prompt of the class, so a reviewer can see what is blocked.
    pub sample_prompt: String,
}

/// The result of one learning cycle over a recorded frontier.
#[derive(Debug, Clone)]
pub struct LearningCycleRun {
    /// Frontier slug the cycle ran over.
    pub frontier: String,
    /// How many recorded frontier items were read.
    pub frontier_items: usize,
    /// Candidates derived, validated or not, in class order.
    pub candidates: Vec<CandidateSurface>,
    /// Classes that produced no adoptable candidate.
    pub blocked: Vec<BlockedClass>,
    /// Promotion proposals in the issue-#656 shape, one per prompt variation.
    pub proposals: Vec<PromotionProposal>,
}

impl LearningCycleRun {
    /// The candidates whose held-out tests all passed.
    #[must_use]
    pub fn validated_candidates(&self) -> Vec<&CandidateSurface> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.validated())
            .collect()
    }

    /// Total held-out tests generated across all candidates.
    #[must_use]
    pub fn held_out_count(&self) -> usize {
        self.candidates
            .iter()
            .map(|candidate| candidate.held_out.len())
            .sum()
    }

    /// Render the cycle as the auditable Links Notation record a human reviews.
    /// Ends trimmed of trailing whitespace.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("learning_cycle\n");
        let _ = writeln!(out, "  record_type \"learning_cycle_run\"");
        let _ = writeln!(out, "  issue \"701\"");
        let _ = writeln!(out, "  frontier \"{}\"", self.frontier);
        let _ = writeln!(out, "  mode \"proposal_only\"");
        let _ = writeln!(out, "  human_gated \"true\"");
        let _ = writeln!(out, "  frontier_items \"{}\"", self.frontier_items);
        let _ = writeln!(out, "  candidates \"{}\"", self.candidates.len());
        let _ = writeln!(
            out,
            "  validated_candidates \"{}\"",
            self.validated_candidates().len()
        );
        let _ = writeln!(out, "  held_out_tests \"{}\"", self.held_out_count());
        let _ = writeln!(out, "  proposals \"{}\"", self.proposals.len());
        let _ = writeln!(out, "  blocked_classes \"{}\"", self.blocked.len());
        for candidate in &self.candidates {
            out.push_str("  candidate\n");
            let _ = writeln!(out, "    language \"{}\"", candidate.language);
            let _ = writeln!(out, "    variation \"{}\"", candidate.variation);
            let _ = writeln!(out, "    role \"{TERM_INFORMATION_ROLE}\"");
            let _ = writeln!(out, "    slot \"{}\"", slot_slug(candidate.slot));
            let _ = writeln!(out, "    surface \"{}\"", candidate.surface);
            for query in &candidate.support {
                let _ = writeln!(out, "    derived_from \"{query}\"");
            }
            let _ = writeln!(out, "    held_out_tests \"{}\"", candidate.held_out.len());
            let _ = writeln!(out, "    held_out_passed \"{}\"", candidate.passed_count());
            let _ = writeln!(out, "    validated \"{}\"", candidate.validated());
        }
        for blocked in &self.blocked {
            out.push_str("  blocked_class\n");
            let _ = writeln!(out, "    language \"{}\"", blocked.language);
            let _ = writeln!(out, "    variation \"{}\"", blocked.variation);
            let _ = writeln!(out, "    reason \"{}\"", blocked.reason);
            let _ = writeln!(out, "    sample_prompt \"{}\"", blocked.sample_prompt);
            let _ = writeln!(out, "    routed_to \"human_triage\"");
        }
        out.trim_end().to_owned()
    }
}

/// Parse a recorded learning-frontier document into its items.
#[must_use]
pub fn parse_frontier_record(document: &str) -> Vec<FrontierItem> {
    let tree = parse_lino(document);
    let mut items = Vec::new();
    for root in &tree.children {
        for node in &root.children {
            if node.name != "frontier_prompt" {
                continue;
            }
            items.push(FrontierItem {
                rank: node.find_child_value("rank").parse().unwrap_or_default(),
                query: node.find_child_value("query").to_owned(),
                language: node.find_child_value("language").to_owned(),
                variation: node.find_child_value("variation").to_owned(),
                prompt: node.find_child_value("prompt").to_owned(),
                engine_intent: node.find_child_value("engine_intent").to_owned(),
            });
        }
    }
    items
}

/// The recorded Google Trends learning frontier.
#[must_use]
pub fn recorded_google_trends_frontier() -> Vec<FrontierItem> {
    parse_frontier_record(GOOGLE_TRENDS_FRONTIER_RECORD)
}

/// Run the adoption cycle over a recorded frontier.
#[must_use]
pub fn run_learning_cycle(frontier: &str, items: &[FrontierItem]) -> LearningCycleRun {
    let mut classes: BTreeMap<(String, String), Vec<&FrontierItem>> = BTreeMap::new();
    for item in items {
        classes
            .entry((item.variation.clone(), item.language.clone()))
            .or_default()
            .push(item);
    }

    let mut candidates = Vec::new();
    let mut blocked = Vec::new();
    for ((variation, language), class_items) in &classes {
        match derive_candidate(language, variation, class_items) {
            Ok(candidate) => candidates.push(candidate),
            Err(reason) => blocked.push(BlockedClass {
                language: language.clone(),
                variation: variation.clone(),
                reason,
                sample_prompt: class_items
                    .first()
                    .map(|item| item.prompt.clone())
                    .unwrap_or_default(),
            }),
        }
    }
    for candidate in &candidates {
        if candidate.validated() {
            continue;
        }
        blocked.push(BlockedClass {
            language: candidate.language.clone(),
            variation: candidate.variation.clone(),
            reason: String::from("held_out_validation_failed"),
            sample_prompt: candidate
                .held_out
                .iter()
                .find(|test| !test.passed)
                .map(|test| test.prompt.clone())
                .unwrap_or_default(),
        });
    }

    let proposals = build_proposals(frontier, &candidates);
    LearningCycleRun {
        frontier: frontier.to_owned(),
        frontier_items: items.len(),
        candidates,
        blocked,
        proposals,
    }
}

/// Run the cycle over the recorded Google Trends frontier.
#[must_use]
pub fn google_trends_learning_cycle() -> LearningCycleRun {
    run_learning_cycle(
        GOOGLE_TRENDS_FRONTIER,
        &recorded_google_trends_frontier(),
    )
}

/// Derive one candidate surface for a frontier class.
///
/// The frame is the residue of deleting the topic query from the prompt. The
/// first [`MINIMUM_SUPPORT`] items (in recorded order) derive it; every later
/// item of the class is held out.
fn derive_candidate(
    language: &str,
    variation: &str,
    items: &[&FrontierItem],
) -> Result<CandidateSurface, String> {
    let frames: Vec<(&FrontierItem, (String, String))> = items
        .iter()
        .filter_map(|item| frame_of(&item.prompt, &item.query).map(|frame| (*item, frame)))
        .collect();
    if frames.len() < MINIMUM_SUPPORT {
        return Err(String::from("insufficient_support"));
    }
    let (before, after) = frames[0].1.clone();
    let support: Vec<String> = frames
        .iter()
        .take(MINIMUM_SUPPORT)
        .map(|(item, _)| item.query.clone())
        .collect();
    if frames
        .iter()
        .take(MINIMUM_SUPPORT)
        .any(|(_, frame)| frame != &(before.clone(), after.clone()))
    {
        return Err(String::from("supporting_prompts_disagree_on_frame"));
    }

    let slot = classify_slot(&before, &after);
    if slot == Slot::Bare {
        return Err(String::from("frame_has_no_subject_slot"));
    }
    let surface = format!("{before}…{after}");
    let held_out = frames
        .iter()
        .skip(MINIMUM_SUPPORT)
        .map(|(item, _)| HeldOutTest {
            prompt: item.prompt.clone(),
            expected_query: item.query.clone(),
            passed: recovers_query(&before, &after, slot, &item.prompt, &item.query),
        })
        .collect::<Vec<_>>();
    if held_out.is_empty() {
        return Err(String::from("no_held_out_prompts_to_validate_against"));
    }

    Ok(CandidateSurface {
        language: language.to_owned(),
        variation: variation.to_owned(),
        surface,
        slot,
        support,
        held_out,
    })
}

/// The request frame around `query` inside `prompt`, as the normalized text
/// before and after the query, or [`None`] when the prompt does not contain the
/// query it was generated from.
fn frame_of(prompt: &str, query: &str) -> Option<(String, String)> {
    let normalized = normalize_prompt(prompt);
    let needle = normalize_prompt(query);
    if needle.is_empty() {
        return None;
    }
    let index = normalized.find(&needle)?;
    Some((
        normalized[..index].to_owned(),
        normalized[index + needle.len()..].to_owned(),
    ))
}

/// Which slot form a frame occupies — the word order of the language, read off
/// the data rather than assumed.
fn classify_slot(before: &str, after: &str) -> Slot {
    match (!before.trim().is_empty(), !after.trim().is_empty()) {
        (true, true) => Slot::Circumfix,
        (true, false) => Slot::Prefix,
        (false, true) => Slot::Suffix,
        (false, false) => Slot::Bare,
    }
}

/// Whether a candidate frame matches `prompt` the way the recogniser matches a
/// lexicon slot form, and recovers exactly the topic `query` from it.
///
/// This mirrors `solver_handlers::web_search_intent::extract_term_information_request`:
/// the seed surface is split at `…` and matched as a prefix, a suffix, or both.
fn recovers_query(before: &str, after: &str, slot: Slot, prompt: &str, query: &str) -> bool {
    let normalized = normalize_prompt(prompt);
    let recovered = match slot {
        Slot::Prefix => normalized.strip_prefix(before).map(str::to_owned),
        Slot::Suffix => normalized.strip_suffix(after).map(str::to_owned),
        Slot::Circumfix => normalized
            .strip_prefix(before)
            .and_then(|rest| rest.strip_suffix(after))
            .map(str::to_owned),
        Slot::Bare => None,
    };
    recovered.is_some_and(|recovered| recovered.trim() == normalize_prompt(query))
}

/// Group validated candidates into one promotion proposal per prompt variation.
///
/// A proposal is exactly what issue #656 consumes: a source link, a review
/// summary, a concrete seed edit, and the benchmark gates that must replay green
/// before it can be applied. The gate carried here is the cycle's *own* evidence
/// — the generated held-out prompts — and it is deliberately unsigned
/// (`command_succeeded` stays false, no evidence digest), because
/// [`crate::promotion::replay_promotion_gates`] replaces every proposal's gates
/// with freshly executed canonical suites before promotion. A learner never gets
/// to certify itself.
fn build_proposals(frontier: &str, candidates: &[CandidateSurface]) -> Vec<PromotionProposal> {
    let mut by_variation: BTreeMap<&str, Vec<&CandidateSurface>> = BTreeMap::new();
    for candidate in candidates.iter().filter(|c| c.validated()) {
        by_variation
            .entry(candidate.variation.as_str())
            .or_default()
            .push(candidate);
    }

    by_variation
        .into_iter()
        .map(|(variation, mut group)| {
            group.sort_by(|left, right| {
                language_order(&left.language).cmp(&language_order(&right.language))
            });
            let passed: usize = group.iter().map(|c| c.passed_count()).sum();
            let failed: usize = group.iter().map(|c| c.failed_count()).sum();
            let languages: Vec<&str> = group.iter().map(|c| c.language.as_str()).collect();
            let summary = format!(
                "Adopt {} learned request-opener surface(s) for the '{variation}' frontier class \
                 ({}), derived from {MINIMUM_SUPPORT} recorded frontier prompts each and \
                 validated on {passed} held-out prompt(s) with {failed} failure(s).",
                group.len(),
                languages.join(", ")
            );
            let gates = vec![held_out_gate(frontier, variation, passed, failed)];
            PromotionProposal::new(
                format!("learning_frontier:{frontier}:{variation}"),
                summary,
                SeedEdit::new(
                    LEARNED_REQUEST_OPENERS_SEED_FILE,
                    meaning_block(variation, &group),
                ),
                gates,
            )
        })
        .collect()
}

/// The benchmark ratchet formed by a class's generated held-out prompts: every
/// one of them must route, and the floor is their count.
fn held_out_gate(
    frontier: &str,
    variation: &str,
    passed: usize,
    failed: usize,
) -> PromotionRatchet {
    let mut gate = PromotionRatchet::new(
        format!("issue_701_learning_cycle_held_out:{frontier}:{variation}"),
        "cargo test --test unit issue_701 -- --nocapture",
        passed.max(1),
        passed,
        failed,
    );
    gate.minimum_pass_rate_basis_points = 10_000;
    gate
}

/// The seed body a proposal would append: one meaning per frontier class, whose
/// lexemes are the learned surfaces, filed under the recogniser's role.
fn meaning_block(variation: &str, candidates: &[&CandidateSurface]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "  {variation}_request_opener");
    out.push_str("    defined-by inquiry\n");
    out.push_str("    defined-by concept\n");
    let _ = writeln!(out, "    role {TERM_INFORMATION_ROLE}");
    for candidate in candidates {
        let _ = writeln!(out, "    lexeme {}", candidate.language);
        out.push_str("      surface\n");
        let _ = writeln!(out, "        text \"{}\"", candidate.surface);
    }
    out
}

/// Declaration order of the supported languages, so proposals and seed edits are
/// emitted in one stable order.
fn language_order(language: &str) -> usize {
    crate::seed::supported_languages()
        .iter()
        .position(|supported| supported == language)
        .unwrap_or(usize::MAX)
}

/// Stable slug of a slot form, for the audit record.
const fn slot_slug(slot: Slot) -> &'static str {
    match slot {
        Slot::Bare => "bare",
        Slot::Prefix => "prefix",
        Slot::Suffix => "suffix",
        Slot::Circumfix => "circumfix",
    }
}
