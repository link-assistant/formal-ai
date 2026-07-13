//! Code- and program-translation helpers split out of [`super`] so the parent
//! `solver_helpers` module stays under the 1000-line cap enforced by
//! `scripts/check-file-size.rs`. These are pure helpers that formalize a snippet
//! of code into a language-agnostic meaning and re-render it in another language,
//! plus the language/algorithm detection and extraction routines around them.
//!
//! `pub use code::*;` in [`super`] keeps every existing `crate::solver_helpers::…`
//! path resolving unchanged.

use super::{extract_backticked, extract_fenced_block, extract_quoted_phrase};
use crate::language::{detect as detect_language, Language};

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
