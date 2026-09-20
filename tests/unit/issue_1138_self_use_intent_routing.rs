//! Issue #1138, plan 14 **wave F** — self-use observations for plan 10.
//!
//! The seven reported frontier prompts of #1087, one held-out paraphrase per
//! intent per language from plan 10's capability-routing sets, and six Spanish
//! variations of the #745 matrix were given to Formal AI through the real
//! `@link-assistant/agent` CLI and through `formal-ai chat`. The observations
//! and the prompts are in `data/benchmarks/self-use-intent-routing.lino`; the
//! raw transcripts are under `docs/case-studies/issue-1138/self-use/`.
//!
//! Two of the frontier prompts already work: `我不明白` (#721) and
//! `Назначь мне встречу с Александром на 20:00 по Грузии` (#869). Both work as
//! *literals*: the paraphrases below, which name the same act, do not reach the
//! same route. That is what these tests are for — a class is closed when its
//! paraphrases route, not when the reported string does.
//!
//! Every assertion here was observed **failing** on `formal-ai 0.350.0` built
//! at commit `74875c1b9b6e36bee9b942343ba295541fdb6997`.
//!
//! These tests drive `solver::solve`, which is the non-agent-mode path. Where
//! agent mode behaves differently the difference is recorded in the corpus and
//! in the case-study README; a route that exists in only one mode is not a route
//! the surfaces share.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::SymbolicAnswer;
use formal_ai::capability_routing::{RoutingOutcome, route};
use formal_ai::solver::solve;

const CORPUS: &str = "data/benchmarks/self-use-intent-routing.lino";

/// The canned description of the retrieval machinery the solver emits in place
/// of doing anything. Every misroute in this file lands on it.
const CAPABILITY_DESCRIPTION: &str = "Providers considered";

/// The localized openers of the same canned description, so a misroute is
/// detected in every language rather than only in the two the English marker
/// happens to cover.
const SEARCH_OPENERS: &[&str] = &["Web search requested", "Поиск в интернете запрошен"];

const ROUTED_CAPABILITIES: &[&str] = &[
    "web_fetch",
    "web_search",
    "read_file",
    "write_file",
    "list_dir",
    "grep",
    "shell",
    "calendar_create_event",
    "response_language_demonstration",
    "concept_measurement_lookup",
    "compose_from_sources",
    "explain_previous_turn",
    "report_issue",
    "ask_user",
];

#[derive(Debug, Clone)]
struct Case {
    family: String,
    language: String,
    prompt: String,
    names_language: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn unquote(raw: &str) -> String {
    let trimmed = raw.trim();
    let inner = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(trimmed);
    inner.replace("\"\"", "\"")
}

fn corpus() -> String {
    fs::read_to_string(repo_root().join(CORPUS))
        .unwrap_or_else(|error| panic!("{CORPUS} should be readable: {error}"))
}

/// Every paraphrase of one family, in file order.
fn family(id: &str) -> Vec<Case> {
    let text = corpus();
    let mut out: Vec<Case> = Vec::new();
    let mut current = String::new();
    let mut language = String::new();
    let mut names_language = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("id ") {
            current = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("language ") {
            language = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("names_language ") {
            let value = unquote(value);
            // The corpus keeps record attributes beneath their prompt.  Accept
            // that natural Lino order as well as a predeclared attribute so a
            // later case cannot inherit the preceding case's language object.
            if current == id
                && let Some(case) = out.last_mut()
                && case.family == current
                && case.names_language.is_empty()
            {
                case.names_language = value;
            } else {
                names_language = value;
            }
        } else if let Some(value) = trimmed.strip_prefix("prompt ")
            && current == id
        {
            out.push(Case {
                family: current.clone(),
                language: language.clone(),
                prompt: unquote(value),
                names_language: std::mem::take(&mut names_language),
            });
        }
    }
    assert!(
        !out.is_empty(),
        "corpus family `{id}` must exist in {CORPUS}"
    );
    out
}

/// One scalar field of a family record.
fn field(id: &str, name: &str) -> String {
    let text = corpus();
    let mut current = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("id ") {
            current = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix(&format!("{name} "))
            && current == id
        {
            return unquote(value);
        }
    }
    panic!("{CORPUS} should carry `{name}` for family `{id}`");
}

/// Whether an answer is the canned search description rather than an answer.
fn is_a_search_instead_of_an_answer(answer: &str) -> bool {
    answer.contains(CAPABILITY_DESCRIPTION)
        || SEARCH_OPENERS.iter().any(|opener| answer.contains(opener))
}

/// Solve through the production entry point and prove its answer carries the
/// exact capability selected by the shared object/act/locus table.
fn solve_through_capability_table(prompt: &str) -> SymbolicAnswer {
    let capability = match route(prompt, ROUTED_CAPABILITIES) {
        RoutingOutcome::Routed { capability } | RoutingOutcome::Lowered { capability, .. } => {
            capability
        }
        other => {
            panic!("held-out self-use prompt must have a table route: {prompt:?} -> {other:?}")
        }
    };
    let answer = solve(prompt);
    let marker = format!("capability={capability}");
    assert!(
        answer.links_notation.contains(&marker),
        "solver::solve must execute the shared object/act/locus decision for {prompt:?}; \
         expected trace marker {marker:?}, got intent {:?}\n{}",
        answer.intent,
        answer.links_notation,
    );
    answer
}

fn misrouted(id: &str) -> Vec<String> {
    family(id)
        .into_iter()
        .filter_map(|case| {
            let answer = solve_through_capability_table(&case.prompt).answer;
            is_a_search_instead_of_an_answer(&answer).then(|| {
                let head: String = answer.trim().chars().take(100).collect();
                format!("{}/{}: {head}", case.family, case.language)
            })
        })
        .collect()
}

/// **Wave F observation, plan 10 / #721.** The reported literal `我不明白` is
/// answered correctly — because the bug report's own string was pasted into
/// `data/seed/intent-routing.lino:400-402`. Held-out paraphrases of the same
/// act are sent to `websearch`, which fetches dictionary pages *about the
/// idiom*: `dictionary.cambridge.org/us/dictionary/english/over-head`,
/// `ldoceonline.com/es-LA/dictionary/spanish-english/se-me-le-escapo`.
///
/// Plan 10 names this shape exactly: the fix for a missing phrasing was another
/// phrasing. A class is closed when its paraphrases route.
#[test]
fn a_statement_of_non_understanding_routes_by_act_not_by_memorized_phrase() {
    let offenders = misrouted("non_understanding_paraphrase");
    assert!(
        offenders.is_empty(),
        "plan 10 (#721): non-understanding is an act, not a list of strings. Paraphrases \
         sent to the web instead:\n{}",
        offenders.join("\n")
    );
}

/// **Wave F observation, plan 10 / #724.** A speak-in-language directive is the
/// triple `(language_name, demonstrate, dialogue)`. All five paraphrases are
/// sent to `websearch`, and the language the prompt names is **dropped from the
/// request**: `Скажи что то на Китайском` is searched as `Скажи что то`,
/// `Give me a line in Hindi.` as `Give me a line`. The named language occurs
/// nowhere in the reply.
///
/// This test asserts both halves: the request is not a web search, and whatever
/// the answer is, it still knows which language was asked for.
#[test]
fn a_speak_in_language_request_keeps_the_language_it_names() {
    let mut offenders = Vec::new();
    for case in family("speak_in_language") {
        let answer = solve_through_capability_table(&case.prompt).answer;
        if case.language == "en" {
            assert_eq!(answer, "Hindi (in hindi): नमस्ते! मैं आपकी क्या मदद कर सकता हूँ?");
        }
        if is_a_search_instead_of_an_answer(&answer) {
            offenders.push(format!("{}: sent to the web", case.language));
        }
        assert!(
            !case.names_language.is_empty(),
            "every speak-in-language case must declare the language it names"
        );
        if !answer.contains(&case.names_language) {
            offenders.push(format!(
                "{}: the answer never mentions `{}`, the language the prompt asked for",
                case.language, case.names_language
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 10 (#724): the object of a demonstrate request may not be dropped on the way \
         to a route.\n{}",
        offenders.join("\n")
    );
}

/// **Wave F observation, plan 10 / #869.** The reported prompt now works:
/// `Назначь мне встречу с Александром на 20:00 по Грузии` produces a complete
/// `VCALENDAR` event at 20:00 `Asia/Tbilisi`. The verb plan 10 recorded as
/// missing has since been added to the list.
///
/// Not one of the five held-out paraphrases reaches the route. Every one is sent
/// to `websearch`, which fetches `time.is/Tbilisi` and then produces no event.
/// The act is reachable through a verb list, not through the act.
///
/// The reported prompt is asserted here too, as a control: if it ever stops
/// producing an event this test says so rather than going quiet.
#[test]
fn a_scheduling_request_reaches_the_calendar_whatever_verb_it_uses() {
    let marker = field("schedule_paraphrase", "expected_marker");
    let reported = field("schedule_paraphrase", "reported_prompt");
    let reported_answer = solve_through_capability_table(&reported).answer;
    assert_eq!(
        reported_answer
            .lines()
            .find(|line| line.starts_with("SUMMARY:")),
        Some("SUMMARY:С александром"),
        "the dynamic calendar timestamps are normalized by documenting its exact summary line"
    );
    assert!(
        reported_answer.contains(&marker),
        "the reported #869 prompt produced a calendar event when wave F was recorded; \
         losing that is a regression, not progress"
    );

    let mut offenders = Vec::new();
    for case in family("schedule_paraphrase") {
        if !solve_through_capability_table(&case.prompt)
            .answer
            .contains(&marker)
        {
            offenders.push(case.language);
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 10 (#869): scheduling is an act, not a verb list. The reported prompt makes \
         an event; these paraphrases of the same act do not: {}",
        offenders.join(", ")
    );
}

/// **Wave F observation, plan 10 / #745.** #745's variation matrix asserts that
/// a URL object routes to `web_fetch`, a local path to `read_file`,
/// content-plus-file to `write_file`, a listing phrasing to `ls` and a code
/// search to `grep_search`. The matrix covers en/ru/hi/zh and contains no
/// Spanish row; the seven capability cue lists in
/// `data/seed/agentic-tool-capabilities.lino` carry 280 phrases in four
/// languages and zero Spanish.
///
/// Observed through the Agent CLI in agent mode: four of six Spanish variations
/// misroute to `websearch`. The starkest is the URL case — `Lee
/// https://example.com y dime qué dice.` carries the URL in the prompt, and the
/// system searched the web for the Spanish sentence instead, opening two
/// `LinkedIn` posts and a `SpanishDict` entry. The write case wrote no file.
#[test]
fn spanish_routing_variations_reach_their_capability() {
    let offenders = misrouted("spanish_routing_matrix");
    assert!(
        offenders.is_empty(),
        "plan 10 (#745): Spanish is one of the five registered languages and is absent \
         from the variation matrix and from every capability cue list. Spanish requests \
         sent to the web instead of their capability:\n{}",
        offenders.join("\n")
    );
}

/// A Spanish write stem is one act observation shared by all typed workspace
/// objects. Inflection may change the surface, but a concrete destination path
/// keeps the table authoritative over a generic concept-lookup promotion.
#[test]
fn spanish_typed_write_variations_keep_the_table_authoritative() {
    for prompt in [
        "Escribe un saludo en notes.txt",
        "Escriba un saludo en notes.txt",
        "Quiero escribir un saludo en notes.txt",
    ] {
        let routed = route(prompt, ROUTED_CAPABILITIES);
        assert_eq!(
            routed,
            RoutingOutcome::Routed {
                capability: String::from("write_file"),
            },
            "the Spanish compose stem plus a typed path must select write_file: {prompt:?}"
        );
        let answer = solve_through_capability_table(prompt);
        assert_eq!(
            answer.answer,
            "This request routes to the `write_file` capability, but this chat surface does not expose the required `shell` tool. Use an agent client that advertises it."
        );
        assert!(
            !is_a_search_instead_of_an_answer(&answer.answer),
            "a typed workspace object must not fall through to concept lookup or web search: {prompt:?}"
        );
    }
}

/// The language-demonstration exception is deliberately narrower than
/// language-name routing as a whole. An explicit translation action retains
/// the translation recipe and its source operand.
#[test]
fn a_real_translation_recipe_is_not_preempted_by_language_demonstration() {
    let answer = solve("Translate \"hello\" to Hindi.");
    assert!(
        answer.intent.starts_with("translate_"),
        "an explicit translation recipe must remain a translation, got intent {:?}\n{}",
        answer.intent,
        answer.links_notation
    );
    assert!(
        !answer
            .links_notation
            .contains("capability=response_language_demonstration"),
        "a target language inside a real translation must not turn into a response-language demonstration"
    );
}

/// **Wave F observation, plan 10 / #1133.** A request naming a local location
/// must never become a web search — in any mode and any language.
///
/// Agent mode: English and Russian route correctly and answer honestly, naming
/// the directory they searched and stating that no wider location was searched.
/// Hindi, Chinese and Spanish are sent to `websearch`, which opens a
/// Windows-desktop tutorial, a GitHub repository called `hivemind-os` and a
/// `SpiderOak` help page. Chat mode, which this test drives, misroutes Russian,
/// Chinese and Spanish to the web as well.
#[test]
fn a_local_location_never_becomes_a_web_search() {
    let offenders = misrouted("local_path_is_not_the_web");
    assert!(
        offenders.is_empty(),
        "plan 10 (#1133): a prompt naming a local location must reach the filesystem, \
         never the web. Languages sent to the web:\n{}",
        offenders.join("\n")
    );
}
