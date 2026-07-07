//! Eleventh agentic recipe — generate every possible question and answer it.
//!
//! Issue #527 asks Formal AI to *"generate all possible questions and answer them"*:
//! produce a lazy stream of questions from smallest to largest, drawn from a
//! frequency-tiered vocabulary; classify each candidate as grammatical or not and,
//! within the grammatical ones, logically meaningful or not; then give the best
//! possible answer to the meaningful ones. [`crate::question_generation`] realises
//! that capability as a deterministic iterator (the enumeration, the frequency-tier
//! curve, and the grammatical/logical gates read from
//! `data/seed/question-generation-lexicon.lino`, never from hardcoded words). This
//! module makes it reachable *through the agentic interface*: an external agent CLI
//! (`Codex`, `OpenCode`, `Gemini`, `Agent CLI`) — or the in-repo driver — asks Formal
//! AI to build the question catalog, and the deterministic planner walks a
//! write → verify → final recipe that emits the catalog as Links Notation, exactly
//! like the self-healing, ledger, and repair-strategy recipes emit their documents.
//!
//! The emitted document is a pure function of the seed lexicon and the deterministic
//! engine ([`crate::engine::FormalAiEngine`]), never of the whole source tree or the
//! network, so it is committed byte-for-byte as `data/meta/question-catalog.lino` and
//! asserted against a fresh render in the issue-#527 tests — like the self-healing and
//! repair-strategy documents. Nothing here trains or promotes anything: the answered
//! questions are a reviewable *recall table* ([`answer_for`]), so a question the system
//! already answered is recognised and answered from the catalog instead of re-derived,
//! while the human-gated learning ledger stays the only path that changes solver
//! behaviour. Neural inference stays a NON-GOAL — every candidate, class, and answer is
//! a deterministic function of the lexicon.

use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::question_generation::{
    generated_question_answers, GeneratedQuestionClass, LogicalMeaningClass, QuestionAcceptance,
    QuestionGenerationConfig, QuestionGenerator, QuestionGrammarClass,
};

/// The workspace path the planner writes the generated catalog document to, and the
/// committed byte-for-byte artifact under `data/meta/`.
pub const QUESTION_CATALOG_PATH: &str = "question-catalog.lino";

/// How many raw candidates (smallest-first, every class) the catalog records to
/// demonstrate the grammatical/logical distinction the issue asks for.
const CATALOG_CANDIDATE_COUNT: usize = 12;

/// How many grammatical-and-meaningful questions the catalog answers via the engine.
const CATALOG_ANSWER_COUNT: usize = 6;

/// A single classified candidate the catalog records (the four-way distinction).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogCandidate {
    /// The candidate question text, e.g. `"what is formal?"`.
    pub text: String,
    /// Word count — candidates are enumerated smallest-first.
    pub word_count: usize,
    /// Grammatical class slug (`grammatical` / `fragment` / `ungrammatical`).
    pub grammar: &'static str,
    /// Logical-meaning class slug (`meaningful` / `open_slot` / `not_meaningful`).
    pub logical_meaning: &'static str,
    /// The combined four-way class slug.
    pub class: &'static str,
}

/// A single grammatical-and-meaningful question paired with its best answer.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogAnswer {
    /// The question text.
    pub question: String,
    /// The engine's intent classification for the question.
    pub intent: String,
    /// The engine's confidence, rounded to three decimals for a stable artifact.
    pub confidence: f32,
    /// The best-possible answer, whitespace-collapsed for a single-line record.
    pub answer: String,
}

/// The full catalog: the classified candidates and the answered meaningful questions.
#[derive(Debug, Clone, PartialEq)]
pub struct QuestionCatalog {
    /// Size of the frequency-ranked vocabulary the candidates were drawn from.
    pub vocabulary_size: usize,
    /// The first [`CATALOG_CANDIDATE_COUNT`] candidates, smallest-first, each classified.
    pub candidates: Vec<CatalogCandidate>,
    /// The first [`CATALOG_ANSWER_COUNT`] grammatical-and-meaningful questions, answered.
    pub answered: Vec<CatalogAnswer>,
}

impl QuestionCatalog {
    /// The answered question whose text matches `question` (trimmed, case-insensitive).
    ///
    /// This is what makes the catalog a *recall table*: a question the system already
    /// generated and answered is recognised and answered from the catalog instead of
    /// re-derived. Matching mirrors the learning ledger's `lesson_for`.
    #[must_use]
    pub fn answer_for(&self, question: &str) -> Option<&CatalogAnswer> {
        let needle = normalise_question(question);
        self.answered
            .iter()
            .find(|answered| normalise_question(&answered.question) == needle)
    }
}

/// Build the canonical catalog from the seed lexicon and the deterministic engine,
/// computed once per process.
///
/// Enumerating the candidates and answering the meaningful questions runs the real
/// generator and engine; the recipe touches the catalog several times per run (planner
/// write step, verify step, final answer) and a server may serve it repeatedly, so
/// memoising keeps the loop responsive without changing its deterministic result.
fn cached_catalog() -> &'static QuestionCatalog {
    static CATALOG: OnceLock<QuestionCatalog> = OnceLock::new();
    CATALOG.get_or_init(build_catalog)
}

fn build_catalog() -> QuestionCatalog {
    // The catalog enumerates the whole frequency-ranked vocabulary ("all possible
    // questions") so the record shows the complete four-way distinction and answers a
    // representative set of meaningful questions. The frequency-tier curve — the "top
    // 10%, then halve" prioritization the issue also asks for — is exercised and pinned
    // separately by the issue-#527 grounding test, so the catalog does not have to
    // shrink its own pool to demonstrate it.
    let base_config = QuestionGenerationConfig::default().with_all_ranked_words();
    let vocabulary_size = base_config.words().len();

    // Candidates: the raw stream (every class), smallest-first, so the record shows the
    // grammatical/logical four-way distinction the issue asks for.
    let candidate_config = base_config
        .clone()
        .with_acceptance(QuestionAcceptance::AnyQuestionLike);
    let candidates = QuestionGenerator::new(candidate_config)
        .take(CATALOG_CANDIDATE_COUNT)
        .map(|question| CatalogCandidate {
            text: question.text.clone(),
            word_count: question.word_count,
            grammar: grammar_slug(question.grammar),
            logical_meaning: logical_meaning_slug(question.logical_meaning),
            class: class_slug(question.class),
        })
        .collect();

    // Answers: the strict stream (grammatical *and* meaningful only), each answered with
    // the deterministic engine — the "best possible answer" the issue asks for.
    let answered = generated_question_answers(base_config)
        .take(CATALOG_ANSWER_COUNT)
        .map(|pair| CatalogAnswer {
            question: pair.question.text.clone(),
            intent: pair.answer.intent.clone(),
            confidence: round_confidence(pair.answer.confidence),
            answer: collapse_whitespace(&pair.answer.answer),
        })
        .collect();

    QuestionCatalog {
        vocabulary_size,
        candidates,
        answered,
    }
}

/// A *differently worded* request for the question-catalog recipe.
///
/// The router recognises the intent from the words, not a hardcoded string.
/// Deliberately avoids the sibling recipes' keywords (no "formalize", no "self-heal",
/// no "ledger", no "repair strategy") so the recipes never collide: this recipe is the
/// *generate-and-answer* capability that enumerates questions and answers them.
pub const QUESTION_CATALOG_TASK: &str =
    "Generate all possible questions from smallest to largest, drawing words from the \
     frequency-tiered vocabulary, classify each candidate as grammatical or not and, \
     among the grammatical ones, logically meaningful or not, then answer the meaningful \
     ones and record the whole question catalog in Links Notation.";

/// Keywords that mark a user turn as the question-catalog recipe.
///
/// Deliberately narrow: every keyword pins the "enumerate questions and answer them"
/// intent, and none overlaps the other recipes' keywords.
const QUESTION_CATALOG_KEYWORDS: [&str; 5] = [
    "question catalog",
    "all possible questions",
    "generate every possible question",
    "generate all questions",
    "enumerate questions",
];

/// Whether `prompt` asks the system to generate the question catalog (issue #527).
#[must_use]
pub fn is_question_catalog_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    // A dedicated keyword, or an explicit "generate/enumerate … questions … answer"
    // pairing, routes here. Kept narrow so ordinary "answer this question" requests do
    // not match.
    QUESTION_CATALOG_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
        || ((lower.contains("generate") || lower.contains("enumerate"))
            && lower.contains("question")
            && lower.contains("answer"))
}

/// Render the catalog document (Links Notation) for the canonical catalog.
/// Deterministic and ends with exactly one trailing newline.
///
/// The output is what the Agent CLI writes to [`QUESTION_CATALOG_PATH`] and what
/// `data/meta/question-catalog.lino` is committed as, asserted byte-for-byte in the
/// issue-#527 tests. The document depends only on the seed lexicon and the deterministic
/// engine, so committing it does not couple it to unrelated source edits.
#[must_use]
pub fn render_document() -> String {
    let catalog = cached_catalog();
    let mut out = String::from("question_catalog\n");
    out.push_str("  record_type \"question_catalog\"\n");
    out.push_str("  intent \"generate_all_possible_questions\"\n");
    let _ = writeln!(out, "  vocabulary_size \"{}\"", catalog.vocabulary_size);
    let _ = writeln!(out, "  candidate_count \"{}\"", catalog.candidates.len());
    let _ = writeln!(out, "  answered_count \"{}\"", catalog.answered.len());
    for candidate in &catalog.candidates {
        out.push_str("  candidate\n");
        field(&mut out, "text", &candidate.text);
        let _ = writeln!(out, "    word_count \"{}\"", candidate.word_count);
        field(&mut out, "grammar", candidate.grammar);
        field(&mut out, "logical_meaning", candidate.logical_meaning);
        field(&mut out, "class", candidate.class);
    }
    for answered in &catalog.answered {
        out.push_str("  answered\n");
        field(&mut out, "question", &answered.question);
        field(&mut out, "intent", &answered.intent);
        let _ = writeln!(out, "    confidence \"{:.3}\"", answered.confidence);
        field(&mut out, "answer", &answered.answer);
    }
    format!("{}\n", out.trim_end())
}

/// The canonical catalog backing the recipe. Exposed so tests can assert the catalog
/// covers every class and answers the meaningful questions without rebuilding it.
#[must_use]
pub fn catalog() -> QuestionCatalog {
    cached_catalog().clone()
}

/// The self-contained final answer: a natural-language summary plus the generated
/// catalog document inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let catalog = cached_catalog();
    format!(
        "Generated the question catalog from a frequency-tiered vocabulary of {vocabulary} words: \
         classified {candidates} candidates smallest-first into the four-way distinction \
         (grammatical-and-meaningful, grammatical open-slot, fragment, ungrammatical) and answered \
         the first {answers} grammatical-and-meaningful questions with the deterministic engine. \
         The answered questions are a reviewable recall table — a repeated question is answered from \
         the catalog instead of re-derived — and nothing changes solver behaviour without the \
         human-gated learning ledger. Neural inference is not used; every candidate, class, and \
         answer is a deterministic function of the lexicon.\n\n\
         Generated document ({QUESTION_CATALOG_PATH}):\n\n{document}",
        vocabulary = catalog.vocabulary_size,
        candidates = catalog.candidates.len(),
        answers = catalog.answered.len(),
        document = document.trim_end(),
    )
}

/// Emit `  <name> "<value>"` at the two-space nested field indent, with the value
/// escaped so it stays a single reviewable Links Notation token.
fn field(out: &mut String, name: &str, value: &str) {
    let _ = writeln!(out, "    {name} \"{}\"", escape_value(value));
}

/// Escape a value for a quoted Links Notation field: collapse whitespace to single
/// spaces and replace embedded quotes with apostrophes (mirrors the sibling recipes).
fn escape_value(value: &str) -> String {
    collapse_whitespace(value).replace('"', "'")
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Round a confidence to three decimals so the committed artifact is stable regardless
/// of floating-point formatting drift.
fn round_confidence(confidence: f32) -> f32 {
    (confidence * 1000.0).round() / 1000.0
}

fn normalise_question(question: &str) -> String {
    collapse_whitespace(question).to_ascii_lowercase()
}

const fn grammar_slug(grammar: QuestionGrammarClass) -> &'static str {
    match grammar {
        QuestionGrammarClass::Grammatical => "grammatical",
        QuestionGrammarClass::Fragment => "fragment",
        QuestionGrammarClass::Ungrammatical => "ungrammatical",
    }
}

const fn logical_meaning_slug(meaning: LogicalMeaningClass) -> &'static str {
    match meaning {
        LogicalMeaningClass::Meaningful => "meaningful",
        LogicalMeaningClass::OpenSlot => "open_slot",
        LogicalMeaningClass::NotMeaningful => "not_meaningful",
    }
}

const fn class_slug(class: GeneratedQuestionClass) -> &'static str {
    match class {
        GeneratedQuestionClass::GrammaticalAndMeaningful => "grammatical_and_meaningful",
        GeneratedQuestionClass::GrammaticalOpenSlot => "grammatical_open_slot",
        GeneratedQuestionClass::Fragment => "fragment",
        GeneratedQuestionClass::Ungrammatical => "ungrammatical",
    }
}
