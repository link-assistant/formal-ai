//! Agentic recipe for issues #498 + #558: route the Google Trends learning
//! frontier through the human-gated self-improvement loop.
//!
//! The catalog recipe ([`super::google_trends_catalog`]) turns trending searches
//! into reviewable multilingual prompts. This recipe closes the loop: it asks
//! Formal AI which of those prompts it *cannot yet resolve* and feeds that frontier
//! into the same proposal-only, human-gated learner the rest of the system uses
//! ([`crate::self_improvement`]). Trending searches are open-domain questions, not
//! program-plan modifiers, so the learner honestly adopts nothing — the value is the
//! auditable frontier and the proof the gap flows into the gated loop rather than off
//! a cliff. The rendered document is a pure function of the committed catalog, so it
//! pins byte-for-byte.

use crate::google_trends_learning::trending_learning_report;

/// The workspace path the planner writes, mirrored by the committed artifact under
/// `data/meta/`.
pub const GOOGLE_TRENDS_LEARNING_PATH: &str = "google-trends-learning.lino";

/// A differently worded task for the Google Trends learning-frontier recipe.
pub const GOOGLE_TRENDS_LEARNING_TASK: &str =
    "Collect the Google Trends learning frontier — the trending searches Formal AI cannot \
     yet resolve — route them through the human-gated self-improvement loop, and record the \
     learning report in Links Notation.";

/// The learning-capability slug this recipe ingests. A learn-from-source
/// directive naming a seed source with this capability drives this exact recipe.
const LEARNING_CAPABILITY: &str = "google_trends_learning";

const GOOGLE_TRENDS_KEYWORDS: [&str; 2] = ["google trends", "trending search"];

/// Whether `prompt` asks for the Google Trends learning-frontier recipe.
///
/// Two disjoint phrasings route here, and both stay clear of the sibling catalog
/// recipe (which keys on prompt/answer/catalog/test):
///
/// 1. **Operator framing** — the explicit learning-loop request: the *frontier*,
///    the *self-improvement loop*, or the "cannot … resolve" pairing.
/// 2. **User teaching directive** (issue #499) — a natural-language "learn from
///    this source" directive that names the Google Trends source, in *any*
///    supported language. This is detected from the same seed-declared registry
///    the chat handler uses ([`crate::seed::learning_sources`]), so the very
///    directive a user types — e.g. "Обратясь сюда ты узнаешь актуальные темы
///    &lt;Google Trends URL&gt;" — drives this artifact-writing recipe through the
///    Agent CLI, not just a chat acknowledgement. Routing keys on the source's
///    declared `capability` slug, never a literal URL or phrase, so a new
///    learnable source is a seed edit rather than a code change (CONTRIBUTING
///    rule 7).
#[must_use]
pub fn is_google_trends_learning_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    if is_learn_from_source_directive(&lower) {
        return true;
    }
    let cannot_resolve = lower.contains("resolve")
        && (lower.contains("cannot") || lower.contains("can't") || lower.contains("not yet"));
    GOOGLE_TRENDS_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
        && (lower.contains("learning frontier")
            || lower.contains("self-improvement loop")
            || cannot_resolve)
}

/// Whether `lowercased` is a user directive teaching the engine to learn from the
/// Google Trends source, resolved entirely from the seed `learning_sources`
/// registry so it stays language-agnostic and free of hardcoded phrases.
fn is_learn_from_source_directive(lowercased: &str) -> bool {
    crate::seed::learning_sources()
        .match_directive(lowercased)
        .is_some_and(|source| source.capability == LEARNING_CAPABILITY)
}

/// Render the deterministic Google Trends learning-frontier report.
#[must_use]
pub fn render_document() -> String {
    format!("{}\n", trending_learning_report().links_notation())
}

/// The self-contained final answer for the agentic loop.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let report = trending_learning_report();
    format!(
        "Routed the Google Trends learning frontier through the human-gated self-improvement \
         loop: of {total} trending prompts, the engine already resolves {handled} and leaves \
         {frontier} on the learning frontier. Every frontier trace was handed to the issue-#558 \
         learner, which — because trending searches are open-domain questions, not program-plan \
         modifiers — proposed {proposals} rules and adopted {adopted}: nothing changes without \
         human review. The report is a pure function of the committed catalog.\n\nGenerated \
         document ({GOOGLE_TRENDS_LEARNING_PATH}):\n\n{document}",
        total = report.total_prompts,
        handled = report.handled_by_engine,
        frontier = report.frontier_count(),
        proposals = report.run.proposals.len(),
        adopted = report.adopted_count(),
        document = document.trim_end(),
    )
}

/// Shell command used by the agentic recipe to verify the written report exists.
#[must_use]
pub fn verification_command() -> String {
    format!(
        "python3 -c p='{GOOGLE_TRENDS_LEARNING_PATH}';s=open(p).read().splitlines();print(len(s));print('\\n'.join(s[:14]))"
    )
}
