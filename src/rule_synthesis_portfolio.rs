//! Rule synthesis as a candidate-solution portfolio (issue #704).
//!
//! The portfolio is a property of the meta algorithm, not of arithmetic. This
//! module proves that by plugging a *second*, completely unrelated work-unit
//! leaf into the same [`crate::draft_portfolio`] engine: constructing the rule
//! that answers a bare program-modification follow-up ("now make it reverse the
//! order").
//!
//! Two of the seed-declared strategies apply here, and they are the same two
//! the sequential path already tried one after another:
//!
//! - `reuse` — recall an approved lesson from the learning ledger and lower it
//!   onto the *current* program artifact.
//! - `rule_derivation` — decompose the request through the operation vocabulary
//!   and construct a fresh candidate rule.
//!
//! The difference is that the portfolio runs them as independent drafts, tests
//! both against the same verification fixture, and keeps the one that verifies
//! at least cost — instead of taking whichever happened to be tried first. When
//! the recalled lesson no longer verifies against the current artifact, the
//! derived rule rescues the turn, and the `draft_comparison` records why.

use crate::coding::{ProgramSpec, program_spec};
use crate::draft_portfolio::{DraftArtifact, DraftPlan, PortfolioLeaf, run_portfolio};
use crate::engine::SelectedRule;
use crate::event_log::EventLog;
use crate::intent_formalization::active_program_context;
use crate::program_plan::ProgramPlan;
use crate::rule_synthesis::{UnknownRuleConstruction, construct_rule_from_unknown};
use crate::solver::ConversationTurn;

/// One synthesized rule candidate, with everything selection needs to judge it.
#[derive(Clone)]
struct RuleDraft {
    spec: ProgramSpec,
    /// Did the plan actually apply a modifier (the lowering check)?
    lowering_passed: bool,
    /// Did the rendered program match the requested operation (render check)?
    render_passed: bool,
    /// Is the produced program in the language of the artifact under discussion?
    language_matches: bool,
}

struct RuleLeaf<'a> {
    follow_up: &'a str,
    history: &'a [ConversationTurn],
}

impl PortfolioLeaf for RuleLeaf<'_> {
    type Artifact = RuleDraft;

    fn supports(&self, strategy: &str) -> bool {
        matches!(strategy, "reuse" | "rule_derivation")
    }

    fn draft(&self, plan: &DraftPlan) -> Option<DraftArtifact<Self::Artifact>> {
        match plan.strategy.as_str() {
            "reuse" => self.draft_from_ledger(),
            "rule_derivation" => self.draft_from_vocabulary(),
            _ => None,
        }
    }

    fn run_tests(&self, artifact: &Self::Artifact) -> Vec<bool> {
        vec![
            artifact.lowering_passed,
            artifact.render_passed,
            artifact.language_matches,
        ]
    }

    fn test_count(&self) -> usize {
        3
    }

    fn composes(&self, artifact: &Self::Artifact) -> bool {
        // Composition with the rest of the answer means the selected rule can
        // actually be rendered into the program the turn will show.
        !artifact.spec.template.code.trim().is_empty()
    }
}

impl RuleLeaf<'_> {
    fn language(&self) -> Option<String> {
        active_program_context(self.history).map(|context| context.language)
    }

    /// `reuse`: replay an approved lesson from the learning ledger against the
    /// program artifact currently under discussion.
    fn draft_from_ledger(&self) -> Option<DraftArtifact<RuleDraft>> {
        let lesson = crate::learning_ledger::approved_lesson_for(self.follow_up)?;
        let context = active_program_context(self.history)?;
        let plan =
            crate::program_plan::lower(&context.task, std::slice::from_ref(&lesson.modifier));
        let spec = program_spec(&plan.resolved_task, &context.language)?;
        let mut trace = EventLog::new();
        trace.append("learning_ledger_recall.lesson", lesson.lesson_id);
        trace.append("learning_ledger_recall.rule", lesson.rule_id);
        trace.append("learning_ledger_recall.modifier", lesson.modifier);
        trace.append("learning_ledger_recall.approved_by", lesson.reviewer);
        trace.append("write_program_plan", plan.links_notation());
        Some(artifact_from(&plan, spec, &context.language, trace))
    }

    /// `rule_derivation`: construct the rule from the operation vocabulary.
    fn draft_from_vocabulary(&self) -> Option<DraftArtifact<RuleDraft>> {
        let construction = construct_rule_from_unknown(self.follow_up, self.history)?;
        let SelectedRule::WriteProgram(spec) = construction.rule else {
            return None;
        };
        let language = self.language()?;
        let mut trace = EventLog::new();
        record_construction(&mut trace, &construction);
        Some(RuleDraft {
            spec,
            // `construct_rule_from_unknown` returns `None` unless both checks of
            // its verification fixture already passed, so a produced draft is a
            // verified one.
            lowering_passed: true,
            render_passed: true,
            language_matches: spec.language.slug == language,
        })
        .map(|value| draft_artifact(value, trace))
    }
}

fn artifact_from(
    plan: &ProgramPlan,
    spec: ProgramSpec,
    language: &str,
    trace: EventLog,
) -> DraftArtifact<RuleDraft> {
    draft_artifact(
        RuleDraft {
            spec,
            lowering_passed: plan.was_modified() && plan.report.applied_count() > 0,
            render_passed: program_spec(&plan.resolved_task, language).is_some(),
            language_matches: spec.language.slug == language,
        },
        trace,
    )
}

/// Cost is time-independent: the size of the program the draft would emit, and
/// the number of events it took to justify it.
fn draft_artifact(value: RuleDraft, trace: EventLog) -> DraftArtifact<RuleDraft> {
    let cost_size = value.spec.template.code.chars().count();
    DraftArtifact {
        value,
        cost_steps: u32::try_from(trace.events().len()).unwrap_or(u32::MAX),
        cost_size,
        trace,
    }
}

fn record_construction(log: &mut EventLog, construction: &UnknownRuleConstruction) {
    log.append(
        "write_program_coreference_rewrite",
        construction.coreference_trace.clone(),
    );
    log.append(
        "rule_synthesis_operation_vocabulary",
        construction.operation_hits.clone(),
    );
    log.append("rule_synthesis_request", construction.request.clone());
    log.append("rule_synthesis_candidate", construction.candidate.clone());
    log.append("rule_verification", construction.verification.clone());
    log.append(
        "write_program_context_recovery",
        construction.recovery_trace.clone(),
    );
    log.append("write_program_plan", construction.plan.clone());
}

/// Seed the portfolio from the impulse content, exactly like every other leaf.
fn seed_from_impulse(follow_up: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in follow_up.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Resolve an unknown program follow-up by drafting candidate rules in parallel
/// and selecting the one the verification fixture approves at least cost.
///
/// Returns `rule` unchanged when it is not `Unknown`, when only one draft was
/// requested (the sequential path stays the default), or when no draft verifies.
pub fn try_portfolio_rule(
    rule: SelectedRule,
    follow_up: &str,
    history: &[ConversationTurn],
    log: &mut EventLog,
    draft_count: u8,
) -> SelectedRule {
    if draft_count <= 1 || !matches!(rule, SelectedRule::Unknown) {
        return rule;
    }
    // Applicability is checked before any slot is planned: with no program
    // artifact under discussion this leaf has nothing to draft, and spending
    // slots on it would put unrelated `draft:` events in the ledger of whatever
    // other leaf is actually solving the turn.
    if active_program_context(history).is_none() {
        return rule;
    }
    let leaf = RuleLeaf { follow_up, history };
    let selection = run_portfolio(
        &leaf,
        seed_from_impulse(follow_up),
        usize::from(draft_count),
        log,
    );
    selection
        .winner
        .map_or(rule, |draft| SelectedRule::WriteProgram(draft.spec))
}
