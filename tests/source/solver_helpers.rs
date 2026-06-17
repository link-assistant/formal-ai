//! Free-function helpers extracted from `solver.rs` to keep that module under
//! the 1000-line cap enforced by `scripts/check-file-size.rs`. These helpers
//! are pure: they do not access any solver state. Items are declared `pub`
//! inside the `pub(crate)` module so the universal solver in `crate::solver`
//! can call them directly without exposing them outside the crate.
//!
//! Arithmetic evaluation lives in [`crate::arithmetic`] and the offline
//! concept knowledge base lives in [`crate::concepts`]; this module
//! re-exports nothing — callers import those modules directly.

use crate::engine::{ExecutionStatus, ProgramSpec, SelectedRule};
use crate::event_log::EventLog;
use crate::language::{detect as detect_language, Language};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecomposedSubImpulse {
    pub id: String,
    pub text: String,
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
    let lower = prompt.to_lowercase();
    let triggers = [" and ", " with tests", " with benchmarks", "; "];
    if !triggers.iter().any(|trigger| lower.contains(trigger)) {
        return Vec::new();
    }

    let parts: Vec<&str> = prompt
        .split([',', ';'])
        .flat_map(|chunk| chunk.split(" and "))
        .flat_map(|chunk| chunk.split(" with "))
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .collect();
    let mut sub_impulses = Vec::new();
    for sub_impulse in parts {
        let id = log.append("sub_impulse", sub_impulse.to_owned());
        sub_impulses.push(DecomposedSubImpulse {
            id,
            text: sub_impulse.to_owned(),
        });
    }
    sub_impulses
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
        if value % divisor == 0 {
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

pub fn extract_introduced_name(prompt: &str) -> Option<String> {
    let needles = ["my name is", "i am called", "call me", "i'm", "i am "];
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
                .find(|token| !token.is_empty())?;
            let cleaned = token.trim_matches(|c: char| !c.is_alphanumeric());
            if !cleaned.is_empty() && cleaned.chars().next().is_some_and(char::is_alphabetic) {
                return Some(cleaned.to_owned());
            }
            search_from = absolute;
        }
    }
    None
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

pub fn translate_program(code: &str, source: &str, target: &str) -> String {
    let trimmed = code.trim();
    match (source, target) {
        ("python", "rust") => {
            if trimmed.starts_with("def add") {
                String::from("fn add(a: i32, b: i32) -> i32 {\n    a + b\n}")
            } else {
                format!("// translation gap for `{trimmed}` from python to rust")
            }
        }
        ("rust", "python") => {
            if trimmed.contains("fn add") {
                String::from("def add(a, b):\n    return a + b")
            } else {
                format!("# translation gap for `{trimmed}` from rust to python")
            }
        }
        _ => format!("// translation gap from {source} to {target}: {trimmed}"),
    }
}

pub fn normalize_code_meaning(code: &str) -> String {
    code.chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_lowercase()
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

#[path = "source_tests/solver_helpers/tests.rs"]
mod tests;
