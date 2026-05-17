//! Free-function helpers extracted from `solver.rs` to keep that module under
//! the 1000-line cap enforced by `scripts/check-file-size.rs`. These helpers
//! are pure: they do not access any solver state. Items are declared `pub`
//! inside the `pub(crate)` module so the universal solver in `crate::solver`
//! can call them directly without exposing them outside the crate.
//!
//! Arithmetic evaluation lives in [`crate::arithmetic`] and the offline
//! concept knowledge base lives in [`crate::concepts`]; this module
//! re-exports nothing — callers import those modules directly.

use crate::engine::{ExecutionStatus, HelloWorldProgram, SelectedRule};
use crate::event_log::EventLog;

pub const fn confidence_for(rule: &SelectedRule, validation: Option<&ValidationChoice>) -> f32 {
    if validation.is_some() {
        return 1.0;
    }
    match rule {
        SelectedRule::Unknown => 0.0,
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

pub fn requires_external_lookup(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    lower.contains("capital of")
        || lower.contains("cite a definition")
        || lower.contains("define associative memory")
        || lower.contains("from wikipedia")
        || lower.contains("born in")
}

pub fn record_decomposition(log: &mut EventLog, prompt: &str, max_depth: u8) {
    if max_depth == 0 {
        return;
    }
    let lower = prompt.to_lowercase();
    let triggers = [" and ", " with tests", " with benchmarks", "; "];
    if !triggers.iter().any(|trigger| lower.contains(trigger)) {
        return;
    }

    let parts: Vec<&str> = prompt
        .split([',', ';'])
        .flat_map(|chunk| chunk.split(" and "))
        .flat_map(|chunk| chunk.split(" with "))
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .collect();
    for sub_impulse in parts {
        log.append("sub_impulse", sub_impulse.to_owned());
    }
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

pub fn detect_source_language(normalized: &str) -> Option<&'static str> {
    if normalized.contains("from english") {
        return Some("en");
    }
    if normalized.contains("from russian") || normalized.starts_with("переведи") {
        return Some("ru");
    }
    if normalized.contains("from hindi") {
        return Some("hi");
    }
    if normalized.contains("from chinese") {
        return Some("zh");
    }
    None
}

pub fn detect_target_language(normalized: &str) -> Option<&'static str> {
    if normalized.contains("to english") {
        return Some("en");
    }
    if normalized.contains("to russian") || normalized.contains("на русский") {
        return Some("ru");
    }
    if normalized.contains("to hindi") || normalized.contains("на хинди") {
        return Some("hi");
    }
    if normalized.contains("to chinese") || normalized.contains("на китайский") {
        return Some("zh");
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

pub fn normalize_meaning(surface: &str) -> String {
    let raw: String = surface
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_alphanumeric())
        .collect();
    canonical_meaning_token(&raw)
}

pub fn canonical_meaning_token(raw: &str) -> String {
    match raw {
        "hello" | "hi" | "hey" | "привет" | "здравствуйте" | "नमस्ते" | "你好" => {
            String::from("greeting")
        }
        "hellohowareyou" | "приветкакдела" | "здравствуйтекаквашидела" => {
            String::from("greeting_how_are_you")
        }
        _ => String::from(raw),
    }
}

pub fn infer_source_from_prompt(prompt: &str) -> &'static str {
    let lower = prompt.to_lowercase();
    if lower.contains("переведи") || lower.contains("опиши") {
        return "ru";
    }
    if let Some(quoted) = extract_quoted_phrase(prompt) {
        let mut latin = 0_usize;
        let mut cyrillic = 0_usize;
        for character in quoted.chars() {
            let codepoint = u32::from(character);
            if character.is_ascii_alphabetic() {
                latin += 1;
            } else if (0x0400..=0x04FF).contains(&codepoint) {
                cyrillic += 1;
            }
        }
        if cyrillic > latin {
            return "ru";
        }
    }
    "en"
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

pub fn translate_surface(surface: &str, source: &str, target: &str) -> String {
    let _ = source;
    let normalized = surface.trim().to_lowercase();
    match target {
        "ru" => match normalized.as_str() {
            "hello" | "hi" => String::from("Привет"),
            "hello, how are you?" => String::from("Здравствуйте, как ваши дела?"),
            _ => format!("[ru] {surface}"),
        },
        "en" => match normalized.as_str() {
            "привет" => String::from("Hi"),
            "hello, how are you?" => String::from("Hello, how are you?"),
            _ => format!("[en] {surface}"),
        },
        "hi" => format!("[hi] {surface}"),
        "zh" => format!("[zh] {surface}"),
        _ => surface.to_owned(),
    }
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

/// Return true when the normalized prompt is a "write a script/program in
/// <language>" request in English, Russian, Hindi, or Chinese.
///
/// Excludes "hello world" prompts — those are already handled by the
/// hello-world rule in the symbolic engine.
pub fn is_write_script_request(normalized: &str) -> bool {
    // Exclude hello-world prompts so the existing rule keeps its intent.
    if normalized.contains("hello") && normalized.contains("world") {
        return false;
    }
    // English: "write a script", "write a program", "write me a script", etc.
    let en_write = normalized.contains("write")
        && (normalized.contains("script")
            || normalized.contains("program")
            || normalized.contains("code"));
    // Russian: "напиши скрипт", "напиши программу", "напиши код", "написать скрипт"
    let ru_write = (normalized.contains("напиши") || normalized.contains("написать"))
        && (normalized.contains("скрипт")
            || normalized.contains("программ")
            || normalized.contains("код"));
    // Hindi: "script likhो", "code likhо"
    let hi_write = normalized.contains("लिखो") || normalized.contains("लिखें");
    // Chinese: "写一个" (write one), "帮我写" (help me write)
    let zh_write = normalized.contains("写一个") || normalized.contains("帮我写");
    en_write || ru_write || hi_write || zh_write
}

pub fn format_write_script_execution(program: &HelloWorldProgram) -> String {
    let cmd = program.execution.check_command.map_or_else(
        || format!("Run command: `{}`", program.execution.run_command),
        |check| {
            format!(
                "Check command: `{check}`\nRun command: `{}`",
                program.execution.run_command
            )
        },
    );
    let output_label = if matches!(program.execution.status, ExecutionStatus::Verified) {
        "Output"
    } else {
        "Expected output after verification"
    };
    format!(
        "Execution status: {} in {}.\n{}\n{}:\n```text\n{}\n```\n{}",
        program.execution.status.label(),
        program.execution.environment,
        cmd,
        output_label,
        program.execution.output,
        program.execution.notes
    )
}

#[cfg(test)]
mod tests {
    use super::{extract_fenced_block, extract_javascript_program, humanize_url, is_prime};
    use crate::concepts::{extract_concept_query, lookup_concept_query, ConceptQuery};
    use crate::solver::{SolverConfig, UniversalSolver};

    fn lookup_term(term: &str) -> bool {
        lookup_concept_query(&ConceptQuery {
            term: term.to_owned(),
            context: None,
        })
        .is_some()
    }

    fn extract_term(prompt: &str) -> Option<String> {
        extract_concept_query(prompt).map(|q| q.term)
    }

    #[test]
    fn defaults_are_bounded_and_offline_capable() {
        let config = SolverConfig::default();
        assert!(!config.agent_mode);
        assert!(!config.diagnostic_mode);
        assert!(!config.offline);
        assert_eq!(config.max_decomposition_depth, 4);
    }

    #[test]
    fn greeting_walks_the_universal_loop() {
        let response = UniversalSolver::default().solve("Hi");
        assert_eq!(response.intent, "greeting");
        assert!(response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("impulse:")));
        assert!(response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("search:local")));
        assert!(response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("trace:")));
    }

    // Regression guard for the keyword/token split in intent-routing.lino:
    // before the fix, "hello" was a greeting keyword matched via `contains_token`,
    // so any multi-word prompt that mentioned "hello" (like a hello-world request)
    // got misrouted to greeting. After the fix, keywords must match the whole
    // prompt exactly, and only the dedicated `token "greet"` uses contains.
    #[test]
    fn hello_world_request_is_not_routed_to_greeting() {
        let response = UniversalSolver::default().solve("Write me hello world program in Rust");
        assert_ne!(
            response.intent, "greeting",
            "answer was: {}",
            response.answer
        );
        assert!(
            response.intent.starts_with("hello_world"),
            "expected hello_world intent, got {}",
            response.intent
        );
        assert!(
            response.answer.to_lowercase().contains("rust"),
            "expected Rust hello world, got: {}",
            response.answer
        );
    }

    #[test]
    fn prime_validation_picks_seventeen_in_range() {
        let response = UniversalSolver::default().solve("Pick a prime number between 14 and 18");
        assert!(response.answer.contains("17"));
    }

    #[test]
    fn prime_check_recognizes_seventeen() {
        assert!(is_prime(17));
        assert!(!is_prime(15));
    }

    #[test]
    fn concept_lookup_finds_seeded_terms() {
        assert!(lookup_term("Wikipedia"));
        assert!(lookup_term("links notation"));
        assert!(lookup_term("the event log"));
        assert!(lookup_term("doublet link"));
        assert!(lookup_term("WebAssembly"));
        assert!(!lookup_term("unknown-concept-xyz"));
    }

    #[test]
    fn concept_extraction_handles_common_prefixes() {
        assert_eq!(
            extract_term("What is Wikipedia?").as_deref(),
            Some("wikipedia"),
        );
        assert_eq!(
            extract_term("Tell me about Links Notation").as_deref(),
            Some("links notation"),
        );
        assert_eq!(
            extract_term("What does Wikidata mean?").as_deref(),
            Some("wikidata"),
        );
        assert_eq!(extract_term("Hi"), None);
        assert_eq!(extract_term("What is 2 + 2?").as_deref(), Some("2 + 2"));
    }

    #[test]
    fn concept_extraction_handles_who_is_variants() {
        assert_eq!(
            extract_term("Tell me, who is Trump").as_deref(),
            Some("trump"),
        );
        assert_eq!(extract_term("Who Trump is").as_deref(), Some("trump"));
    }

    #[test]
    fn concept_extraction_handles_multilingual_prefixes() {
        assert_eq!(
            extract_term("Что такое Википедия?").as_deref(),
            Some("википедия"),
        );
        assert_eq!(
            extract_term("Расскажи про Links Notation").as_deref(),
            Some("links notation"),
        );
        assert_eq!(
            extract_term("विकिपीडिया क्या है?").as_deref(),
            Some("विकिपीडिया"),
        );
        assert_eq!(extract_term("维基百科是什么?").as_deref(), Some("维基百科"),);
        assert_eq!(extract_term("什么是 Rust?").as_deref(), Some("rust"));
    }

    #[test]
    fn concept_lookup_finds_multilingual_aliases() {
        assert!(lookup_term("Википедия"));
        assert!(lookup_term("विकिपीडिया"));
        assert!(lookup_term("维基百科"));
    }

    #[test]
    fn concept_query_splits_term_and_context() {
        let query = extract_concept_query("what is IIR in ML?").expect("should extract");
        assert_eq!(query.term, "iir");
        assert_eq!(query.context.as_deref(), Some("ml"));
    }

    #[test]
    fn concept_query_handles_russian_context_delimiter() {
        let query = extract_concept_query("что такое iir в ml").expect("should extract");
        assert_eq!(query.term, "iir");
        assert_eq!(query.context.as_deref(), Some("ml"));
    }

    #[test]
    fn concept_query_handles_hindi_context_first() {
        let query = extract_concept_query("ML में IIR क्या है").expect("should extract");
        // Hindi puts context before the concept; the parser captures it as
        // the lexical term half. The lookup_concept_query swaps order as
        // needed when ranking against records.
        assert!(query.term == "ml" || query.term == "iir");
        assert!(query.context.is_some());
    }

    #[test]
    fn concept_query_handles_chinese_context_first() {
        let query = extract_concept_query("ML中的IIR是什么").expect("should extract");
        assert!(query.term == "ml" || query.term == "iir");
        assert!(query.context.is_some());
    }

    #[test]
    fn javascript_extraction_finds_fenced_program() {
        let prompt = "Please run this javascript:\n```js\nconsole.log(1 + 2);\n```";
        let body = extract_javascript_program(prompt).expect("should extract");
        assert_eq!(body, "console.log(1 + 2);");
    }

    #[test]
    fn javascript_extraction_requires_explicit_request() {
        let prompt = "Here is some javascript:\n```js\nconsole.log(1);\n```";
        assert_eq!(extract_javascript_program(prompt), None);
    }

    #[test]
    fn fenced_block_picks_matching_language() {
        let text = "intro\n```python\nprint(1)\n```\nthen\n```js\nconsole.log(2)\n```";
        assert_eq!(
            extract_fenced_block(text, &["js"]).as_deref(),
            Some("console.log(2)"),
        );
    }

    #[test]
    fn universal_solver_answers_arithmetic_via_evaluator() {
        let response = UniversalSolver::default().solve("What is 7 * (3 + 4)?");
        assert_eq!(response.intent, "calculation");
        assert!(response.answer.contains("49"));
        assert!(response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("calculation")));
    }

    #[test]
    fn universal_solver_recalls_introduced_name() {
        use crate::solver::{ConversationTurn, UniversalSolver};
        let history = [ConversationTurn::user("My name is Ada.")];
        let response = UniversalSolver::default().solve_with_history("What is my name?", &history);
        assert_eq!(response.intent, "recall_name");
        assert!(response.answer.contains("Ada"));
    }

    #[test]
    fn universal_solver_looks_up_concept() {
        let response = UniversalSolver::default().solve("What is Wikipedia?");
        assert_eq!(response.intent, "concept_lookup");
        assert!(response.answer.to_lowercase().contains("wikipedia"));
    }

    #[test]
    fn solver_config_default_is_offline_capable() {
        let config = SolverConfig::default();
        assert!(!config.offline);
        assert!(!config.agent_mode);
    }

    // ---------------------------------------------------------------------
    // Issue #21: humanize_url renders percent-encoded URLs as readable IRIs
    // across every language while preserving query strings and falling back
    // gracefully on malformed input.
    // ---------------------------------------------------------------------

    #[test]
    fn humanize_url_decodes_cyrillic_percent_escapes() {
        let encoded = "https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4";
        assert_eq!(
            humanize_url(encoded),
            "https://ru.wikipedia.org/wiki/Изумруд",
        );
    }

    #[test]
    fn humanize_url_decodes_devanagari_percent_escapes() {
        let encoded =
            "https://hi.wikipedia.org/wiki/%E0%A4%A8%E0%A4%AE%E0%A4%B8%E0%A5%8D%E0%A4%A4%E0%A5%87";
        assert_eq!(humanize_url(encoded), "https://hi.wikipedia.org/wiki/नमस्ते");
    }

    #[test]
    fn humanize_url_decodes_chinese_percent_escapes() {
        let encoded = "https://zh.wikipedia.org/wiki/%E4%BD%A0%E5%A5%BD";
        assert_eq!(humanize_url(encoded), "https://zh.wikipedia.org/wiki/你好");
    }

    #[test]
    fn humanize_url_preserves_reserved_uri_delimiters() {
        // `?`, `&`, `=`, `#`, `/`, `:` must remain percent-encoded so that
        // structural meaning of the URI is not disturbed during display.
        let encoded = "https://example.com/path?a%3Db%26c%3Dd#frag%2Fpart";
        assert_eq!(humanize_url(encoded), encoded);
    }

    #[test]
    fn humanize_url_preserves_query_string_values_around_decoded_path() {
        // The path segment is decoded; the query stays encoded.
        let encoded = "https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4?utm_source=demo&page=1";
        assert_eq!(
            humanize_url(encoded),
            "https://ru.wikipedia.org/wiki/Изумруд?utm_source=demo&page=1",
        );
    }

    #[test]
    fn humanize_url_returns_original_when_no_percent_escapes_present() {
        let url = "https://en.wikipedia.org/wiki/Albert_Einstein";
        assert_eq!(humanize_url(url), url);
    }

    #[test]
    fn humanize_url_passes_through_malformed_escapes() {
        // `%ZZ` is not a valid escape — leave the bytes as-is rather than
        // throwing. This matches the JS `decodeURI` fallback strategy
        // (catch URIError → return original).
        let url = "https://example.com/%ZZbroken";
        assert_eq!(humanize_url(url), url);
    }

    #[test]
    fn humanize_url_handles_truncated_trailing_percent() {
        let url = "https://example.com/path%";
        assert_eq!(humanize_url(url), url);
    }

    #[test]
    fn humanize_url_accepts_lowercase_hex_digits() {
        let encoded = "https://ru.wikipedia.org/wiki/%d0%98%d0%b7%d1%83%d0%bc%d1%80%d1%83%d0%b4";
        assert_eq!(
            humanize_url(encoded),
            "https://ru.wikipedia.org/wiki/Изумруд",
        );
    }

    #[test]
    fn humanize_url_decodes_only_invalid_utf8_returns_original() {
        // A lone continuation byte (0x80) is not valid UTF-8; fall back to
        // the original URL rather than emitting broken text.
        let url = "https://example.com/%80";
        assert_eq!(humanize_url(url), url);
    }

    #[test]
    fn humanize_url_decodes_mixed_already_decoded_and_encoded_path() {
        let encoded = "https://ru.wikipedia.org/wiki/Изумруд_%28минерал%29";
        // `(` is `%28` and not a reserved delimiter, so it decodes.
        assert_eq!(
            humanize_url(encoded),
            "https://ru.wikipedia.org/wiki/Изумруд_(минерал)",
        );
    }
}
