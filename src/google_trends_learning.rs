//! Wire the Google Trends catalog's *unanswered* prompts into the human-gated
//! auto-learning loop (issues #498 + #558).
//!
//! Issue #498 asks Formal AI to "train" on popular Google queries. For a symbolic,
//! inspectable engine, "training" is not gradient descent: it is (a) turning
//! trending searches into reviewable test cases — the job of
//! [`mod@crate::google_trends_catalog`] — and (b) recognising which of those cases the
//! engine *cannot yet route* and feeding that frontier into the same proposal-only,
//! human-gated learning pipeline the rest of the system uses
//! ([`crate::self_improvement`], issue #558) instead of silently dropping it.
//!
//! This module builds that bridge and reports it faithfully. Trending searches are
//! open-domain factual questions, not program-plan modifiers, so the rule-synthesis
//! learner — [`learn_rules_from_unknown_traces`] — correctly *declines* to fabricate
//! a seed rule for them: every frontier prompt becomes a rejected trace routed to
//! human triage, and **nothing is auto-adopted**. The value is twofold: the
//! auditable frontier itself — the engine's own map of which trending prompts it
//! does and does not cover across every supported language — and the proof that the
//! gap flows into the gated loop rather than off a cliff. Reporting the honest
//! rejection (rather than manufacturing a rule) is exactly the faithful,
//! proposal-only behaviour issue #558 requires.

use std::fmt::Write as _;

use crate::engine::FormalAiEngine;
use crate::google_trends_catalog::google_trends_catalog;
use crate::self_improvement::{
    learn_rules_from_unknown_traces, BenchmarkGateReport, LearningRun, UnknownTrace,
};

/// One trending prompt the engine cannot yet route — a point on the learning
/// frontier that the human-gated loop is handed for triage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendingFrontierEntry {
    /// One-based rank of the originating Trends topic.
    pub rank: usize,
    /// The trending search term the prompt was generated from.
    pub query: String,
    /// Language tag of the generated prompt (`en`, `ru`, `hi`, `zh`).
    pub language: String,
    /// Stable variation key of the generated prompt.
    pub variation_key: String,
    /// The prompt text the engine could not route.
    pub prompt: String,
    /// The intent the engine returned (always the unknown path for a frontier entry).
    pub engine_intent: String,
}

/// The result of routing the Trends catalog's unanswered prompts through the
/// issue-#558 self-improvement loop.
///
/// It records the coverage split (how many prompts the engine routes vs. leaves on
/// the frontier) and the honest, gated [`LearningRun`] over the frontier. Because
/// trending questions are open-domain, the run adopts nothing: it is a faithful,
/// proposal-only artifact, reproducible as a pure function of the committed catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendingLearningReport {
    /// Total generated prompts considered (across all topics and languages).
    pub total_prompts: usize,
    /// Prompts the engine routed to a real capability (e.g. `web_search`).
    pub handled_by_engine: usize,
    /// Prompts the engine could not route — the learning frontier.
    pub frontier: Vec<TrendingFrontierEntry>,
    /// The gated self-improvement run over the frontier traces.
    pub run: LearningRun,
}

impl TrendingLearningReport {
    /// How many prompts are on the learning frontier.
    #[must_use]
    pub const fn frontier_count(&self) -> usize {
        self.frontier.len()
    }

    /// How many learned rules the loop adopted — always zero here, because
    /// open-domain trending questions produce no adoptable program-plan rule.
    #[must_use]
    pub fn adopted_count(&self) -> usize {
        self.run.adoptable_rules().len()
    }

    /// Whether the loop stayed proposal-only: it proposed nothing and adopted
    /// nothing, so no seed data changed without human review.
    #[must_use]
    pub fn is_proposal_only(&self) -> bool {
        self.run.proposals.is_empty() && self.adopted_count() == 0
    }

    /// The single reason the loop rejected every frontier trace, if the frontier is
    /// non-empty and the reasons are uniform (they are: an open-domain question
    /// carries no rule-synthesis candidate for the learner to verify).
    #[must_use]
    pub fn uniform_rejection_reason(&self) -> Option<&str> {
        let first = self.run.rejections.first()?;
        self.run
            .rejections
            .iter()
            .all(|rejection| rejection.reason == first.reason)
            .then_some(first.reason.as_str())
    }

    /// Render the report as Links Notation — the committable, auditable artifact a
    /// human reviews. Ends trimmed of trailing whitespace.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("google_trends_learning\n");
        field(&mut out, "record_type", "google_trends_learning_report");
        field(&mut out, "issue", "498");
        field(&mut out, "auto_learning_loop", "issue_558_self_improvement");
        field(&mut out, "human_gated", "true");
        field(
            &mut out,
            "summary",
            "Trending searches are open-domain questions, not program-plan modifiers, \
             so the rule-synthesis learner declines to fabricate seed rules for them. \
             The learning frontier — every trending prompt the engine cannot yet route — \
             is recorded here and handed to the human-gated self-improvement loop for \
             triage; nothing is auto-adopted.",
        );
        let _ = writeln!(out, "  total_prompts \"{}\"", self.total_prompts);
        let _ = writeln!(out, "  handled_by_engine \"{}\"", self.handled_by_engine);
        let _ = writeln!(out, "  learning_frontier \"{}\"", self.frontier.len());
        field(&mut out, "learning_run_id", &self.run.id);
        let _ = writeln!(
            out,
            "  learning_run_trace_count \"{}\"",
            self.run.trace_count
        );
        let _ = writeln!(
            out,
            "  learning_run_proposals \"{}\"",
            self.run.proposals.len()
        );
        let _ = writeln!(out, "  learning_run_adopted \"{}\"", self.adopted_count());
        field(
            &mut out,
            "learning_run_rejection_reason",
            self.uniform_rejection_reason().unwrap_or("none"),
        );
        field(&mut out, "benchmark_suite", &self.run.gate.suite_id);
        for entry in &self.frontier {
            out.push_str("  frontier_prompt\n");
            let _ = writeln!(out, "    rank \"{}\"", entry.rank);
            nested(&mut out, "query", &entry.query);
            nested(&mut out, "language", &entry.language);
            nested(&mut out, "variation", &entry.variation_key);
            nested(&mut out, "prompt", &entry.prompt);
            nested(&mut out, "engine_intent", &entry.engine_intent);
            nested(&mut out, "routed_to", "human_triage");
        }
        out.trim_end().to_owned()
    }
}

/// Build the deterministic trending-learning report from the committed catalog.
///
/// Every generated prompt is re-answered through the same [`FormalAiEngine`] the
/// catalog uses (the catalog keeps only a flattened answer, so the full
/// [`crate::engine::SymbolicAnswer`] is recovered here); prompts on the unknown path
/// become [`UnknownTrace`]s and are fed to [`learn_rules_from_unknown_traces`] under
/// a green issue-#362 gate. The result is a pure function of the committed seed, so
/// it is reproducible and pinnable byte-for-byte.
#[must_use]
pub fn trending_learning_report() -> TrendingLearningReport {
    let catalog = google_trends_catalog();
    let engine = FormalAiEngine;

    let mut total_prompts = 0usize;
    let mut handled_by_engine = 0usize;
    let mut frontier = Vec::new();
    let mut traces = Vec::new();

    for topic in &catalog.topics {
        for prompt in &topic.prompts {
            total_prompts += 1;
            let answer = engine.answer(&prompt.prompt);
            // The frontier is the prompts the engine left *unrouted* (intent
            // `unknown`). A prompt the solver ultimately routed to a real capability
            // (e.g. `web_search`) is handled, even though its internal trace touched
            // the unknown path — so classify on the final intent, not on whether a
            // trace can be recovered.
            if answer.intent == "unknown" {
                let trace = UnknownTrace::from_symbolic_answer(&prompt.prompt, &answer)
                    .expect("an unknown-intent answer must yield an unknown trace");
                frontier.push(TrendingFrontierEntry {
                    rank: topic.rank,
                    query: topic.query.clone(),
                    language: prompt.language.clone(),
                    variation_key: prompt.variation_key.clone(),
                    prompt: prompt.prompt.clone(),
                    engine_intent: answer.intent.clone(),
                });
                traces.push(trace);
            } else {
                handled_by_engine += 1;
            }
        }
    }

    // A green coding-modification gate, matching the canonical self-healing case:
    // the point is that even with tests green, an open-domain question yields no
    // adoptable rule, so the loop still adopts nothing.
    let gate = BenchmarkGateReport::issue_362_from_counts(4, 0);
    let run = learn_rules_from_unknown_traces(&traces, gate);

    TrendingLearningReport {
        total_prompts,
        handled_by_engine,
        frontier,
        run,
    }
}

fn field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "  {key} \"{}\"", quote(value));
}

fn nested(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "    {key} \"{}\"", quote(value));
}

fn quote(value: &str) -> String {
    collapse_whitespace(value)
        .replace('\\', "\\\\")
        .replace('"', "'")
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
