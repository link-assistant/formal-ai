//! Free-function helpers extracted from `solver.rs` to keep that module under
//! the 1000-line cap enforced by `scripts/check-file-size.rs`. These helpers
//! are pure: they do not access any solver state. Items are declared `pub`
//! inside the `pub(crate)` module so the universal solver in `crate::solver`
//! can call them directly without exposing them outside the crate.
//!
//! Arithmetic evaluation lives in [`crate::arithmetic`] and the offline
//! concept knowledge base lives in [`crate::concepts`]; this module
//! re-exports nothing — callers import those modules directly.

use crate::engine::{normalize_prompt, ExecutionStatus, ProgramSpec, SelectedRule};
use crate::event_log::EventLog;
use crate::intent_formalization::{formalize_intent, IntentKind};
use crate::language::{detect as detect_language, Language};
use crate::solver::{BlueprintComposition, ExecutionSurface, SolverConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecomposedSubImpulse {
    pub id: String,
    pub text: String,
    pub independent: bool,
}

pub const fn confidence_for(rule: &SelectedRule, validation: Option<&ValidationChoice>) -> f32 {
    if validation.is_some() {
        return 1.0;
    }
    match rule {
        SelectedRule::Unknown => 0.0,
        SelectedRule::UnsupportedWriteProgram { .. } => 0.4,
        _ => 1.0,
    }
}

pub fn is_unbounded_autonomy(normalized: &str) -> bool {
    let triggers = [
        "forever",
        "continuously",
        "non-stop",
        "nonstop",
        "indefinitely",
        "without stopping",
        "until i tell you to stop",
    ];
    triggers.iter().any(|trigger| normalized.contains(trigger))
}

pub fn is_forget_request(normalized: &str) -> bool {
    normalized.contains("forget ")
        || normalized.starts_with("forget")
        || normalized.contains("delete the greeting concept")
}

pub fn is_cache_flush_request(normalized: &str) -> bool {
    (normalized.contains("flush") || normalized.contains("clear")) && normalized.contains("cache")
}

pub fn is_agent_request(normalized: &str) -> bool {
    normalized.contains("[agent]")
        || normalized.contains("enable agent")
        || normalized.contains("agent mode")
}

pub fn is_agent_opt_in(normalized: &str) -> bool {
    normalized.contains("[agent]")
        || normalized.contains("enable agent")
        || normalized.contains("agent mode")
}

pub fn is_destructive_action(normalized: &str) -> bool {
    let triggers = [
        "rm -rf",
        "delete the .git",
        "drop table",
        "delete /",
        "delete the database",
    ];
    triggers.iter().any(|trigger| normalized.contains(trigger))
}

pub fn is_unbounded_loop(normalized: &str) -> bool {
    normalized.contains("while true")
        || normalized.contains("infinite loop")
        || normalized.contains("for one hour")
        || normalized.contains("forever")
}

pub fn is_inappropriate_content(normalized: &str) -> bool {
    // The vulgar/obscene surfaces — Russian mat and English profanity migrated
    // verbatim from the original hardcoded lists, plus Hindi and Chinese
    // equivalents — live in `data/seed/meanings-policy.lino` under the
    // [`vulgar_content_marker`](crate::seed::ROLE_VULGAR_CONTENT_MARKER) role.
    // They are matched as raw substrings, so the screen stays language-
    // independent and tolerant of inflection without listing any profanity in
    // code.
    crate::seed::lexicon().mentions_role_raw(crate::seed::ROLE_VULGAR_CONTENT_MARKER, normalized)
}

pub fn requires_external_lookup(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    lower.contains("capital of")
        || lower.contains("cite a definition")
        || lower.contains("define associative memory")
        || lower.contains("from wikipedia")
        || lower.contains("born in")
}

pub fn record_decomposition(
    log: &mut EventLog,
    prompt: &str,
    max_depth: u8,
) -> Vec<DecomposedSubImpulse> {
    if max_depth == 0 {
        return Vec::new();
    }

    let language = detect_language(prompt);
    let whole_intent = formalize_intent(prompt, language.slug(), None);
    if whole_intent.route.is_none() || whole_intent.kind == IntentKind::Courtesy {
        let independent_parts = independent_actionable_segments(prompt);
        if independent_parts.len() > 1 {
            return record_sub_impulses(log, independent_parts, true);
        }
    }

    let lower = prompt.to_lowercase();
    let triggers = [" and ", " with tests", " with benchmarks", "; "];
    if !triggers.iter().any(|trigger| lower.contains(trigger)) {
        return Vec::new();
    }

    let parts: Vec<String> = prompt
        .split([',', ';'])
        .flat_map(|chunk| chunk.split(" and "))
        .flat_map(|chunk| chunk.split(" with "))
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .map(str::to_owned)
        .collect();
    record_sub_impulses(log, parts, false)
}

fn record_sub_impulses(
    log: &mut EventLog,
    parts: Vec<String>,
    independent: bool,
) -> Vec<DecomposedSubImpulse> {
    let mut sub_impulses = Vec::new();
    for sub_impulse in parts {
        let id = log.append("sub_impulse", sub_impulse.clone());
        sub_impulses.push(DecomposedSubImpulse {
            id,
            text: sub_impulse,
            independent,
        });
    }
    sub_impulses
}

fn independent_actionable_segments(prompt: &str) -> Vec<String> {
    let parts = split_candidate_actionable_parts(prompt);
    if parts.len() <= 1
        || !parts
            .iter()
            .all(|part| looks_like_independent_impulse(part))
    {
        return Vec::new();
    }
    parts
}

fn split_candidate_actionable_parts(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    for sentence in split_sentences(text) {
        for clause in sentence.split(';') {
            for comma_part in clause.split(',') {
                for and_part in comma_part.split(" and ") {
                    let trimmed = strip_leading_coordinator(and_part.trim());
                    if !trimmed.is_empty() {
                        parts.push(trimmed.to_owned());
                    }
                }
            }
        }
    }
    parts
}

fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut current = String::new();
    for (index, &ch) in chars.iter().enumerate() {
        current.push(ch);
        let strong_terminator = matches!(ch, '?' | '!' | '。' | '！' | '？');
        let period_boundary =
            ch == '.' && chars.get(index + 1).is_none_or(|next| next.is_whitespace());
        if strong_terminator || period_boundary {
            push_trimmed_segment(&mut sentences, &current);
            current.clear();
        }
    }
    push_trimmed_segment(&mut sentences, &current);
    sentences
}

fn push_trimmed_segment(out: &mut Vec<String>, candidate: &str) {
    let trimmed = candidate.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_owned());
    }
}

fn strip_leading_coordinator(text: &str) -> &str {
    let trimmed = text.trim_start();
    let lowered = trimmed.to_ascii_lowercase();
    for coordinator in ["and", "then"] {
        if lowered == coordinator {
            return "";
        }
        let prefix = format!("{coordinator} ");
        if lowered.starts_with(&prefix) {
            return trimmed[prefix.len()..].trim_start();
        }
    }
    trimmed
}

fn looks_like_independent_impulse(segment: &str) -> bool {
    let normalized = normalize_prompt(segment);
    if normalized.is_empty() {
        return false;
    }
    let language = detect_language(segment);
    let formalization = formalize_intent(segment, language.slug(), None);
    formalization.route.is_some()
        || matches!(
            formalization.kind,
            IntentKind::Task
                | IntentKind::Question
                | IntentKind::Requirement
                | IntentKind::Courtesy
        )
        || formalization
            .relevants
            .iter()
            .any(|relevant| relevant.starts_with("handler:"))
}

pub fn record_candidates(log: &mut EventLog, prompt: &str, intent: &str) {
    let lower = prompt.to_lowercase();
    if lower.contains("suggest a name") || lower.contains("suggest names") {
        for candidate in ["LinkLight", "Doublet", "FormalLeaf"] {
            log.append("candidate", candidate.to_owned());
        }
        return;
    }
    if lower.contains("pick a") || lower.contains("choose a") {
        log.append("candidate", "primary".to_owned());
        log.append("candidate", "secondary".to_owned());
        return;
    }
    log.append("candidate", intent.to_owned());
}

#[derive(Debug, Clone)]
pub struct ValidationChoice {
    pub answer: String,
}

pub fn record_validation(log: &mut EventLog, prompt: &str) -> Option<ValidationChoice> {
    let lower = prompt.to_lowercase();
    if lower.contains("prime") {
        if let Some((low, high)) = extract_range(&lower) {
            for candidate in low..=high {
                if is_prime(candidate) {
                    let answer = format!("{candidate}");
                    log.append("validation", format!("prime_between_{low}_and_{high}"));
                    return Some(ValidationChoice { answer });
                }
            }
        }
        log.append("validation", "no_prime_in_range".to_owned());
    }
    None
}

pub fn extract_range(lower: &str) -> Option<(u64, u64)> {
    let numbers: Vec<u64> = lower
        .split(|character: char| !character.is_ascii_digit())
        .filter_map(|token| token.parse::<u64>().ok())
        .collect();
    match numbers.as_slice() {
        [low, high] if low <= high => Some((*low, *high)),
        _ => None,
    }
}

pub const fn is_prime(value: u64) -> bool {
    if value < 2 {
        return false;
    }
    let mut divisor: u64 = 2;
    while divisor.saturating_mul(divisor) <= value {
        if value.is_multiple_of(divisor) {
            return false;
        }
        divisor += 1;
    }
    true
}

pub fn extract_quoted_phrase(text: &str) -> Option<String> {
    for (open, close) in [('\'', '\''), ('"', '"'), ('`', '`'), ('«', '»')] {
        if let Some(start) = text.find(open) {
            if let Some(end_offset) = text[start + open.len_utf8()..].find(close) {
                let inner = &text[start + open.len_utf8()..start + open.len_utf8() + end_offset];
                return Some(inner.to_owned());
            }
        }
    }
    None
}

pub fn extract_backticked(text: &str) -> Option<String> {
    let start = text.find('`')?;
    let rest = &text[start + 1..];
    let end = rest.find('`')?;
    Some(rest[..end].to_owned())
}

/// Walk the event log for a user-introduced name. Looks in the current
/// prompt first, then in each `prior_turn:user` event so name recall works
/// across multi-turn conversations.
pub fn recall_name_from_history(log: &EventLog, prompt: &str) -> Option<String> {
    if let Some(name) = extract_introduced_name(prompt) {
        return Some(name);
    }
    for event in log.events() {
        if event.kind == "prior_turn:user" {
            if let Some(name) = extract_introduced_name(&event.payload) {
                return Some(name);
            }
        }
    }
    None
}

/// Return the last user turn recorded in the log, ignoring the current
/// impulse. Used by "what did I just ask?" style recall handlers.
pub fn last_user_turn(log: &EventLog) -> Option<&str> {
    log.events()
        .iter()
        .rev()
        .find(|event| event.kind == "prior_turn:user")
        .map(|event| event.payload.as_str())
}

/// Return the last assistant turn recorded in the log. Used by follow-up
/// handlers such as "how it works?" that need to infer the topic from the
/// previous reply.
pub fn last_assistant_turn(log: &EventLog) -> Option<&str> {
    log.events()
        .iter()
        .rev()
        .find(|event| event.kind == "prior_turn:assistant")
        .map(|event| event.payload.as_str())
}

/// Return the most recent prior turn (regardless of role) together with its
/// role label, ignoring the current impulse. Used by the "what was written in
/// the previous message?" recall handler, which replays the immediately
/// preceding message whether it came from the user or the assistant.
pub fn last_turn(log: &EventLog) -> Option<(&'static str, &str)> {
    log.events()
        .iter()
        .rev()
        .find_map(|event| match event.kind {
            "prior_turn:user" => Some(("user", event.payload.as_str())),
            "prior_turn:assistant" => Some(("assistant", event.payload.as_str())),
            _ => None,
        })
}

pub fn extract_introduced_name(prompt: &str) -> Option<String> {
    let needles = ["my name is", "i am called", "call me", "i'm", "i am "];
    extract_name_after_needles(prompt, &needles)
}

/// Assistant-name-setting phrasings (issue #676). Each needle pins the *assistant*
/// as the subject being (re)named — "your name is", "I'll call you", "you are
/// called" — so a declarative rename like "Now your name is Ineffa" is recognised
/// while questions ("what is your name") and user self-introductions are left alone.
const ASSISTANT_NAME_NEEDLES: [&str; 16] = [
    "your name is",
    "your name shall be",
    "your name will be",
    "your name would be",
    "your new name is",
    "let your name be",
    "you are named",
    "you're named",
    "you are called",
    "you're called",
    "i'll call you",
    "i will call you",
    "i'll name you",
    "i will name you",
    "i name you",
    "i'll refer to you as",
];

/// Extract a name the user assigns to the *assistant* from a single prompt.
///
/// Mirrors [`extract_introduced_name`] but keys off assistant-directed needles so
/// "Now your name is Ineffa" yields `Ineffa`. Returns `None` for questions and for
/// user self-introductions, keeping the two name paths from colliding.
#[must_use]
pub fn extract_assistant_name(prompt: &str) -> Option<String> {
    extract_name_after_needles(prompt, &ASSISTANT_NAME_NEEDLES)
}

/// Shared token scan: find the first non-empty word following any needle and clean
/// it down to an alphabetic-leading identifier.
fn extract_name_after_needles(prompt: &str, needles: &[&str]) -> Option<String> {
    let lower = prompt.to_lowercase();
    for needle in needles {
        let mut search_from = 0;
        while let Some(offset) = lower[search_from..].find(needle) {
            let absolute = search_from + offset + needle.len();
            let tail = &prompt[absolute..];
            let token = tail
                .trim_start()
                .split(|c: char| {
                    c.is_whitespace() || matches!(c, '.' | ',' | '!' | '?' | ';' | ':' | '\n')
                })
                .find(|token| !token.is_empty());
            if let Some(token) = token {
                let cleaned = token.trim_matches(|c: char| !c.is_alphanumeric());
                if !cleaned.is_empty() && cleaned.chars().next().is_some_and(char::is_alphabetic) {
                    return Some(cleaned.to_owned());
                }
            }
            search_from = absolute;
        }
    }
    None
}

/// Recall the most recently assigned assistant name across the conversation.
///
/// Checks the current prompt first, then walks `prior_turn:user` events from newest
/// to oldest so a later rename ("actually, call you Ada") wins over an earlier one.
#[must_use]
pub fn recall_assistant_name_from_history(log: &EventLog, prompt: &str) -> Option<String> {
    if let Some(name) = extract_assistant_name(prompt) {
        return Some(name);
    }
    log.events()
        .iter()
        .rev()
        .filter(|event| event.kind == "prior_turn:user")
        .find_map(|event| extract_assistant_name(&event.payload))
}

pub fn detect_program_languages(normalized: &str) -> Option<(&'static str, &'static str)> {
    let langs = [
        "python",
        "rust",
        "javascript",
        "typescript",
        "go",
        "java",
        "c",
        "ruby",
    ];
    let from = langs
        .iter()
        .find(|lang| normalized.contains(&format!("from {lang}")))
        .copied();
    let to = langs
        .iter()
        .find(|lang| normalized.contains(&format!("to {lang}")))
        .copied();
    match (from, to) {
        (Some(f), Some(t)) => Some((f, t)),
        _ => None,
    }
}

/// Language-neutral meta representation of a code fragment — the code meta
/// language.
///
/// Issue #526 forbids direct pair-specific translation: with `N` languages a
/// direct table needs `N * N` translators, which is computationally
/// intractable. Code translation therefore mirrors the natural-language
/// pipeline. A fragment is first *formalized* into a `CodeMeaning` and then
/// *rendered* into the requested target, so the translator stays at `N`
/// formalizers plus `N` renderers instead of `N * N` hardcoded pairs. Adding a
/// language is one new [`formalize_code_meaning`] recognizer and one new
/// [`render_code_meaning`] arm — never a new `(source, target)` pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeMeaning {
    /// A binary function `add(a, b)` that returns the sum of its two
    /// parameters. This is the currently seeded code meaning; widen coverage by
    /// adding variants here rather than by adding direct language pairs.
    BinaryAddFunction,
    /// The fragment has not been formalized into a known code meaning. Carries
    /// the trimmed source so callers render a traceable gap instead of
    /// manufacturing an incorrect translation.
    Unformalized(String),
}

impl CodeMeaning {
    /// Stable, language-neutral slug for this code meaning. Two fragments that
    /// share a meaning (e.g. the same add function written in Rust and in
    /// JavaScript) collapse to the same slug, which is what lets a round trip
    /// preserve its `meaning:` link.
    pub fn slug(&self) -> String {
        match self {
            Self::BinaryAddFunction => String::from("function:add:binary_sum"),
            Self::Unformalized(source) => source
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .collect::<String>()
                .to_lowercase(),
        }
    }
}

/// Formalize a source-language code fragment into the language-neutral code
/// meta language. The source language is irrelevant to the result: `add`
/// written in Rust, Python, or JavaScript all collapse to the same
/// [`CodeMeaning::BinaryAddFunction`], which is exactly why the round trip
/// preserves meaning.
pub fn formalize_code_meaning(code: &str) -> CodeMeaning {
    if is_binary_add_function(code) {
        CodeMeaning::BinaryAddFunction
    } else {
        CodeMeaning::Unformalized(code.trim().to_owned())
    }
}

/// Render a [`CodeMeaning`] into `target`'s surface syntax. When the meaning is
/// unknown, or the target language has no seeded rendering yet, this returns a
/// traceable, language-appropriate translation-gap comment rather than
/// inventing plausible-but-wrong code.
pub fn render_code_meaning(meaning: &CodeMeaning, source: &str, target: &str) -> String {
    if matches!(meaning, CodeMeaning::BinaryAddFunction) {
        if let Some(rendered) = render_binary_add(target) {
            return rendered;
        }
    }
    let subject = match meaning {
        CodeMeaning::BinaryAddFunction => "add function",
        CodeMeaning::Unformalized(source_code) => source_code.as_str(),
    };
    format!(
        "{} translation gap for `{subject}` from {source} to {target}",
        code_comment_prefix(target)
    )
}

/// Render the seeded [`CodeMeaning::BinaryAddFunction`] into `target`. Returns
/// `None` when `target` has no seeded rendering, so the caller can emit a gap.
fn render_binary_add(target: &str) -> Option<String> {
    let rendered = match target {
        "python" => "def add(a, b):\n    return a + b",
        "rust" => "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}",
        "javascript" => "function add(a, b) {\n    return a + b;\n}",
        "typescript" => "function add(a: number, b: number): number {\n    return a + b;\n}",
        "go" => "func add(a int, b int) int {\n    return a + b\n}",
        _ => return None,
    };
    Some(String::from(rendered))
}

/// Comment prefix used to render a translation gap in `language`'s own syntax.
fn code_comment_prefix(language: &str) -> &'static str {
    match language {
        "python" | "ruby" => "#",
        _ => "//",
    }
}

/// Translate a code fragment from `source` to `target` by routing through the
/// code meta language: `source -> CodeMeaning -> target`. There is no direct
/// `(source, target)` path — see [`CodeMeaning`] for why.
pub fn translate_program(code: &str, source: &str, target: &str) -> String {
    let meaning = formalize_code_meaning(code);
    render_code_meaning(&meaning, source, target)
}

/// Language-neutral meaning slug for a code fragment, used to key its
/// `meaning:` evidence link. Delegates to [`formalize_code_meaning`] so the
/// meaning a fragment translates *through* is the same meaning its trace
/// records — that shared identity is the #526 round-trip invariant.
pub fn normalize_code_meaning(code: &str) -> String {
    formalize_code_meaning(code).slug()
}

fn is_binary_add_function(code: &str) -> bool {
    let compact = code
        .chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    let declares_add = compact.contains("defadd(")
        || compact.contains("fnadd(")
        || compact.contains("functionadd(");
    declares_add && compact.contains("a+b")
}

/// Normalize a surface fragment into a deterministic, language-independent
/// key for hashing into a meaning id. The previous implementation looked
/// the surface up in a hand-curated registry; that is now removed (the
/// real meaning id comes from Wikidata via the translation pipeline). We
/// keep the normalization step so the legacy hash continues to be stable
/// across whitespace, casing, and punctuation differences.
pub fn normalize_meaning(surface: &str) -> String {
    let raw: String = surface
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_alphanumeric())
        .collect();
    canonical_meaning_token(&raw)
}

/// Return the canonical meaning token for a normalized surface. With the
/// offline registry gone, this is currently the identity function — the
/// translation pipeline supplies the language-neutral [`MeaningId`] when a
/// translation request actually fires, and callers that need a hash key
/// (e.g. the engine's stable id) feed the normalized surface directly.
pub fn canonical_meaning_token(raw: &str) -> String {
    String::from(raw)
}

pub fn infer_source_from_prompt(prompt: &str) -> &'static str {
    let lower = prompt.to_lowercase();
    if let Some(surface) = extract_quoted_phrase(prompt)
        .or_else(|| crate::translation::extract_unquoted_translation_surface(prompt))
    {
        let language = detect_language(&surface);
        if language != Language::Unknown {
            return language.slug();
        }
    }
    // Issue #386: the source language of an un-annotated request is the language
    // the user issued the *translation command* in. Ask the lexicon which
    // language's translation-action verb (переведи/опиши, अनुवाद, 翻译/翻譯…) the
    // prompt carries — the per-language stems live once in
    // data/seed/meanings-translation.lino under the `translate` meaning; this
    // code knows only the concept and the language-code bridge. English is the
    // default when no command verb is present.
    crate::seed::lexicon()
        .first_role_language(
            crate::seed::ROLE_TRANSLATION_ACTION,
            &lower,
            &["ru", "hi", "zh"],
        )
        .unwrap_or("en")
}

pub fn infer_program_languages_from_code(
    code: &str,
    normalized: &str,
) -> Option<(&'static str, &'static str)> {
    let trimmed = code.trim();
    let source = if trimmed.contains("fn ") || trimmed.contains("let ") || trimmed.contains("-> ") {
        "rust"
    } else if trimmed.contains("def ") || trimmed.contains("print(") {
        "python"
    } else if trimmed.contains("function ") || trimmed.contains("console.log") {
        "javascript"
    } else {
        return None;
    };
    let langs = [
        "python",
        "rust",
        "javascript",
        "typescript",
        "go",
        "java",
        "c",
        "ruby",
    ];
    let target = langs
        .iter()
        .find(|lang| normalized.contains(&format!("to {lang}")))
        .copied()?;
    Some((source, target))
}

/// Translate `surface` and return the full pipeline result so callers can
/// inspect the meaning id, candidate list, and provenance trail.
///
/// The caller is responsible for matching the source's leading case and
/// terminal punctuation; see [`crate::translation::match_source_formatting`].
pub fn translate_surface_detailed(
    surface: &str,
    source: &str,
    target: &str,
) -> Result<crate::translation::Translation, crate::translation::HttpError> {
    crate::translation::translate_via_default_pipeline(surface, source, target)
}

pub fn extract_concept_from_query(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    if !(lower.contains("what do you know about") || lower.contains("introspect")) {
        return None;
    }
    let quoted = extract_quoted_phrase(prompt)?;
    Some(quoted)
}

pub fn detect_algorithm_language(normalized: &str) -> &'static str {
    let langs = [
        ("python", "python"),
        (" py ", "python"),
        ("rust", "rust"),
        (" rs ", "rust"),
        ("javascript", "javascript"),
        ("typescript", "typescript"),
        ("go ", "go"),
        ("golang", "go"),
        ("java", "java"),
        ("ruby", "ruby"),
    ];
    for (needle, slug) in langs {
        if normalized.contains(needle) {
            return slug;
        }
    }
    "python"
}

pub fn build_sorting_algorithm_answer(lang: &str, with_tests: bool) -> String {
    let (fence, code, tests) = match lang {
        "rust" => (
            "rust",
            "fn sort(values: &mut Vec<i32>) {\n    values.sort();\n}",
            "#[test]\nfn test_sort_ascending() {\n    let mut v = vec![3, 1, 2];\n    sort(&mut v);\n    assert_eq!(v, vec![1, 2, 3]);\n}",
        ),
        "javascript" | "typescript" => (
            lang,
            "function sort(values) {\n  return [...values].sort((a, b) => a - b);\n}",
            "function test_sort_ascending() {\n  assert.deepEqual(sort([3,1,2]), [1,2,3]);\n}",
        ),
        _ => (
            "python",
            "def sort(values):\n    return sorted(values)\n",
            "def test_sort_ascending():\n    assert sort([3, 1, 2]) == [1, 2, 3]\n",
        ),
    };

    if with_tests {
        format!(
            "Here is a reviewable sorting algorithm in {lang} with a test:\n\n```{fence}\n{code}\n```\n\nTests:\n```{fence}\n{tests}\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable."
        )
    } else {
        format!(
            "Here is a reviewable sorting algorithm in {lang}:\n\n```{fence}\n{code}\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable."
        )
    }
}

/// Extract a JavaScript program from a prompt that asks the solver to run it.
/// Looks for triple-backtick code fences first (with optional `js`/`javascript`
/// language tag), then single-backtick spans, then `run "...";` quoted bodies.
/// Returns `None` when the prompt does not appear to request JS execution.
pub fn extract_javascript_program(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let asks_to_run = lower.contains("run this javascript")
        || lower.contains("run this js")
        || lower.contains("execute this javascript")
        || lower.contains("execute this js")
        || lower.contains("run the following javascript")
        || lower.contains("run the following js")
        || lower.contains("evaluate this javascript")
        || lower.contains("evaluate this js");
    if !asks_to_run {
        return None;
    }
    if let Some(body) = extract_fenced_block(prompt, &["javascript", "js"]) {
        return Some(body);
    }
    if let Some(body) = extract_backticked(prompt) {
        return Some(body);
    }
    extract_quoted_phrase(prompt)
}

/// Render a percent-encoded URL in its readable IRI form (RFC 3987).
///
/// Leaves reserved URI delimiters (`; / ? : @ & = + $ , #`) percent-encoded so
/// query strings and fragments still resolve. Returns the input unchanged when
/// the URL has no percent-escapes or when decoding would produce invalid
/// UTF-8.
///
/// Mirrors the JavaScript `decodeURI` semantics used in
/// `src/web/formal_ai_worker.js::humanizeUrl` so Wikipedia source links render
/// identically across every formal-ai surface (issue #21).
#[must_use]
pub fn humanize_url(url: &str) -> String {
    if !url.contains('%') {
        return url.to_owned();
    }
    let bytes = url.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_nibble(bytes[i + 1]), hex_nibble(bytes[i + 2])) {
                let value = (hi << 4) | lo;
                if is_reserved_uri_delimiter(value) {
                    out.extend_from_slice(&bytes[i..=i + 2]);
                } else {
                    out.push(value);
                }
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| url.to_owned())
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

const fn is_reserved_uri_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b';' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b',' | b'#'
    )
}

/// Find a fenced code block whose info string matches one of the supplied
/// languages (case-insensitive). Returns the block body with trailing newlines
/// trimmed.
pub fn extract_fenced_block(text: &str, languages: &[&str]) -> Option<String> {
    let fence = "```";
    let mut cursor = 0usize;
    while let Some(open_offset) = text[cursor..].find(fence) {
        let open = cursor + open_offset;
        let info_start = open + fence.len();
        let info_end = text[info_start..]
            .find('\n')
            .map_or(text.len(), |n| info_start + n);
        let info = text[info_start..info_end].trim().to_lowercase();
        let body_start = (info_end + 1).min(text.len());
        let body_end_offset = text[body_start..].find(fence)?;
        let body_end = body_start + body_end_offset;
        let body = text[body_start..body_end].trim_end_matches('\n').to_owned();
        if info.is_empty() || languages.iter().any(|lang| info == *lang) {
            return Some(body);
        }
        cursor = body_end + fence.len();
    }
    None
}

/// Return true when the normalized prompt asks for a script or code to be
/// *authored* — the author verb ([`ROLE_SCRIPT_AUTHORING_VERB`], carried by the
/// `write` meaning) paired with a script-or-code artifact noun
/// ([`ROLE_SCRIPT_OR_CODE_ARTIFACT`], carried by `script` and `code`) — in any
/// supported language. No natural-language word is hardcoded here; the lexicon
/// answers which surface forms evidence each role.
///
/// Defers to the parametric write-program route for prompts that name the broad
/// program genus ([`ROLE_PROGRAM_GENUS`]) or the canonical hello-world archetype
/// ([`ROLE_HELLO_WORLD_REFERENCE`]), so those keep their richer formalization
/// instead of collapsing into a bare script.
///
/// [`ROLE_SCRIPT_AUTHORING_VERB`]: crate::seed::ROLE_SCRIPT_AUTHORING_VERB
/// [`ROLE_SCRIPT_OR_CODE_ARTIFACT`]: crate::seed::ROLE_SCRIPT_OR_CODE_ARTIFACT
/// [`ROLE_PROGRAM_GENUS`]: crate::seed::ROLE_PROGRAM_GENUS
/// [`ROLE_HELLO_WORLD_REFERENCE`]: crate::seed::ROLE_HELLO_WORLD_REFERENCE
pub fn is_write_script_request(normalized: &str) -> bool {
    use crate::seed::{
        ROLE_HELLO_WORLD_REFERENCE, ROLE_PROGRAM_GENUS, ROLE_SCRIPT_AUTHORING_VERB,
        ROLE_SCRIPT_OR_CODE_ARTIFACT,
    };
    let lexicon = crate::seed::lexicon();
    // The parametric write-program route owns the broad program genus and the
    // canonical hello-world archetype; step aside for those.
    if lexicon.mentions_role(ROLE_PROGRAM_GENUS, normalized)
        || lexicon.mentions_role(ROLE_HELLO_WORLD_REFERENCE, normalized)
    {
        return false;
    }
    // Author a script: the write verb plus a script-or-code artifact noun.
    lexicon.mentions_role(ROLE_SCRIPT_AUTHORING_VERB, normalized)
        && lexicon.mentions_role(ROLE_SCRIPT_OR_CODE_ARTIFACT, normalized)
}

pub fn format_write_script_execution(program: ProgramSpec) -> String {
    let execution = &program.language.execution;
    let expected_output = program.expected_output();
    let cmd = execution.check_command.map_or_else(
        || format!("Run command: `{}`", execution.run_command),
        |check| {
            format!(
                "Check command: `{check}`\nRun command: `{}`",
                execution.run_command
            )
        },
    );
    let output_label = if matches!(execution.status, ExecutionStatus::Verified) {
        "Output"
    } else {
        "Expected output after verification"
    };
    format!(
        "Execution status: {} in {}.\n{}\n{}:\n```text\n{}\n```\n{}",
        execution.status.label(),
        execution.environment,
        cmd,
        output_label,
        expected_output,
        execution.notes
    )
}

/// Parse the `FORMAL_AI_DEFINITION_FUSION` env switch into an explicit override.
pub fn env_definition_fusion_by_default() -> Option<bool> {
    env_bool_with_extra_truthy(
        "FORMAL_AI_DEFINITION_FUSION",
        &["auto", "merge", "fusion", "default"],
        &["explicit", "manual", "none"],
    )
}

/// Parse a boolean env var using the standard truthy/falsy vocabulary.
pub fn env_bool(name: &str) -> Option<bool> {
    env_bool_with_extra_truthy(name, &[], &[])
}

/// Parse a boolean env var, extending the truthy/falsy vocabulary with extras.
pub fn env_bool_with_extra_truthy(name: &str, truthy: &[&str], falsy: &[&str]) -> Option<bool> {
    let raw = std::env::var(name).ok()?;
    let value = raw.trim().to_ascii_lowercase();
    if value.is_empty() {
        return None;
    }
    match value.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        other if truthy.contains(&other) => Some(true),
        other if falsy.contains(&other) => Some(false),
        _ => None,
    }
}

/// Parse a finite `f32` env var, clamped into `[min, max]`.
pub fn env_bounded_f32(name: &str, min: f32, max: f32) -> Option<f32> {
    let parsed = std::env::var(name).ok()?.trim().parse::<f32>().ok()?;
    if parsed.is_finite() {
        Some(parsed.clamp(min, max))
    } else {
        None
    }
}

/// Return `true` when an env var is set to anything other than a falsy value.
pub fn env_truthy(name: &str) -> bool {
    std::env::var(name).is_ok_and(|raw| {
        let value = raw.trim();
        !value.is_empty()
            && !matches!(
                value.to_ascii_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            )
    })
}

/// Build a [`crate::solver::SolverConfig`] from the documented environment
/// overrides. This is the body of [`crate::solver::SolverConfig::from_env`],
/// extracted here so `src/solver.rs` stays under the 1000-line cap.
pub fn config_from_env() -> SolverConfig {
    let mut config = SolverConfig::default();
    if env_truthy("FORMAL_AI_OFFLINE") {
        config.offline = true;
    }
    if env_truthy("FORMAL_AI_AGENT_MODE") {
        config.agent_mode = true;
    }
    if env_truthy("FORMAL_AI_DIAGNOSTIC_MODE") {
        config.diagnostic_mode = true;
    }
    if let Some(value) = env_definition_fusion_by_default() {
        config.definition_fusion_by_default = value;
    }
    if let Some(value) = env_bool("FORMAL_AI_ASSOCIATIVE_PROJECT_PROMOTION")
        .or_else(|| env_bool("FORMAL_AI_PROJECT_PROMOTION"))
    {
        config.associative_project_promotion = value;
    }
    if let Ok(value) =
        std::env::var("FORMAL_AI_EXECUTION_SURFACE").or_else(|_| std::env::var("FORMAL_AI_SURFACE"))
    {
        if let Some(surface) = ExecutionSurface::from_env_value(&value) {
            config.execution_surface = surface;
        }
    }
    if let Some(value) = env_bounded_f32("FORMAL_AI_TEMPERATURE", 0.0, 1.0) {
        config.temperature = value;
    }
    if let Some(value) = env_bounded_f32("FORMAL_AI_GUESS_PROBABILITY", 0.0, 1.0) {
        config.guess_probability = value;
    }
    if let Some(value) = env_bounded_f32("FORMAL_AI_FOLLOW_UP_PROBABILITY", 0.0, 1.0) {
        config.follow_up_probability = value;
    }
    if let Ok(value) = std::env::var("FORMAL_AI_CACHE_TTL_SECONDS") {
        if let Ok(parsed) = value.parse::<u64>() {
            config.cache_ttl_seconds = parsed;
        }
    }
    if let Ok(value) = std::env::var("FORMAL_AI_BLUEPRINT_COMPOSITION")
        .or_else(|_| std::env::var("FORMAL_AI_PROGRAM_COMPOSITION"))
    {
        if let Some(mode) = BlueprintComposition::from_value(&value) {
            config.blueprint_composition = mode;
        }
    }
    crate::meta_core::apply_env_modes(
        &mut config.recursion_mode,
        &mut config.selection_mode,
        &mut config.skill_mode,
    );
    config
}
