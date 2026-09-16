//! Issue #1138 B10, plan 10 leaves 8-11: the 420-case held-out capability
//! routing suite.
//!
//! Seven intents, five languages, twelve paraphrases each. Every string is held
//! out: it occurs in no file under `data/seed/` and no file under `src/`, so a
//! pass cannot come from a memorized cue phrase.
//!
//! The corpus is `data/benchmarks/capability-routing/{en,ru,hi,zh,es}.lino`
//! with `data/benchmarks/capability-routing-suite.lino` as its header.
//! Written before the leaves that make it pass (plan 14 wave T).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::capability_routing::{
    Act, Locus, ObjectType, RouteRow, RoutingOutcome, route, route_with, routing_table,
    routing_table_from,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn lino_records(text: &str) -> Vec<Vec<&str>> {
    let mut records = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(current);
            current = Vec::new();
        }
        current.push(line);
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

fn lino_field(record: &[&str], wanted: &str) -> String {
    let raw = record
        .iter()
        .filter_map(|line| line.trim().split_once(' '))
        .find_map(|(name, value)| (name == wanted).then(|| value.trim().to_owned()))
        .unwrap_or_else(|| panic!("missing {wanted:?} in {record:?}"));
    for delimiter in ['"', '\''] {
        if raw.starts_with(delimiter) && raw.ends_with(delimiter) && raw.len() >= 2 {
            let doubled = format!("{delimiter}{delimiter}");
            return raw[1..raw.len() - 1].replace(&doubled, &delimiter.to_string());
        }
    }
    raw
}

struct RoutingCase {
    id: String,
    language: String,
    intent: String,
    prompt: String,
    expected: String,
    prohibited: String,
}

/// The capabilities a client in this suite advertises. Deliberately the full
/// set the decision table names, so a miss is a routing failure rather than an
/// unadvertised tool.
const ADVERTISED: &[&str] = &[
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

fn routing_cases() -> Vec<RoutingCase> {
    let mut cases = Vec::new();
    for language in ["en", "ru", "hi", "zh", "es"] {
        let path = repo_root().join(format!(
            "data/benchmarks/capability-routing/{language}.lino"
        ));
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("missing {language} routing partition: {error}"));
        for record in lino_records(&text) {
            assert_eq!(
                lino_field(&record, "record_type"),
                "capability_routing_case"
            );
            assert_eq!(lino_field(&record, "language"), language);
            cases.push(RoutingCase {
                id: lino_field(&record, "id"),
                language: language.to_owned(),
                intent: lino_field(&record, "intent"),
                prompt: lino_field(&record, "prompt"),
                expected: lino_field(&record, "expected_capability"),
                prohibited: lino_field(&record, "prohibited_capability"),
            });
        }
    }
    cases
}

fn resolved_capability(outcome: &RoutingOutcome) -> Option<&str> {
    match outcome {
        RoutingOutcome::Routed { capability } | RoutingOutcome::Lowered { capability, .. } => {
            Some(capability.as_str())
        }
        RoutingOutcome::HonestGap { .. } | RoutingOutcome::Ask { .. } => None,
    }
}

#[test]
fn the_routing_corpus_is_four_hundred_and_twenty_held_out_cases() {
    let header =
        fs::read_to_string(repo_root().join("data/benchmarks/capability-routing-suite.lino"))
            .expect("capability-routing-suite.lino readable");
    let records = lino_records(&header);
    assert_eq!(lino_field(&records[0], "minimum_pass_count"), "420");
    assert_eq!(lino_field(&records[0], "languages"), "en|ru|hi|zh|es");

    let cases = routing_cases();
    assert_eq!(cases.len(), 420, "7 intents x 5 languages x 12 paraphrases");

    let prompts: BTreeSet<&str> = cases.iter().map(|case| case.prompt.as_str()).collect();
    assert_eq!(prompts.len(), 420, "every paraphrase is distinct");

    let mut per_cell: BTreeMap<(String, String), usize> = BTreeMap::new();
    for case in &cases {
        *per_cell
            .entry((case.intent.clone(), case.language.clone()))
            .or_default() += 1;
    }
    assert_eq!(per_cell.len(), 35, "seven intents in five languages");
    for (cell, count) in &per_cell {
        assert_eq!(*count, 12, "cell {cell:?} must carry twelve paraphrases");
    }
}

#[test]
fn held_out_paraphrases_route_without_cross_tool_misroutes() {
    let cases = routing_cases();
    let mut misroutes: Vec<String> = Vec::new();
    let mut misses: Vec<String> = Vec::new();
    for case in &cases {
        let outcome = route(&case.prompt, ADVERTISED);
        match resolved_capability(&outcome) {
            Some(capability) if capability == case.expected => {}
            Some(capability) => {
                if capability == case.prohibited {
                    misroutes.push(format!(
                        "{} ({} / {}): reached the prohibited `{capability}`",
                        case.id, case.language, case.intent
                    ));
                } else {
                    misses.push(format!(
                        "{} ({} / {}): reached `{capability}`, expected `{}`",
                        case.id, case.language, case.intent, case.expected
                    ));
                }
            }
            None => misses.push(format!(
                "{} ({} / {}): resolved to no capability, expected `{}`",
                case.id, case.language, case.intent, case.expected
            )),
        }
    }
    assert!(
        misroutes.is_empty(),
        "{} of {} cases reached a capability outside their intent, which is the #745 and \
         #758 defect: {:?}",
        misroutes.len(),
        cases.len(),
        misroutes.iter().take(12).collect::<Vec<_>>()
    );
    assert!(
        misses.is_empty(),
        "{} of {} cases did not reach their expected capability: {:?}",
        misses.len(),
        cases.len(),
        misses.iter().take(12).collect::<Vec<_>>()
    );
}

#[test]
fn every_triple_resolves_to_a_row_or_asks() {
    // Silent UNKNOWN is unreachable by construction: every path ends in one of
    // routed, lowered, honest gap or ask, and each is observable.
    let cases = routing_cases();
    let mut silent: Vec<String> = Vec::new();
    for case in &cases {
        match route(&case.prompt, ADVERTISED) {
            RoutingOutcome::Routed { capability } if capability.is_empty() => {
                silent.push(case.id.clone());
            }
            RoutingOutcome::HonestGap { needed, missing }
                if needed.is_empty() || missing.is_empty() =>
            {
                silent.push(case.id.clone());
            }
            RoutingOutcome::Ask { readings } if readings.len() < 2 => {
                silent.push(case.id.clone());
            }
            _ => {}
        }
    }
    assert!(
        silent.is_empty(),
        "{} cases ended in an outcome that names nothing; an honest gap names the \
         capability that was needed and what was missing, and an ask names both \
         readings: {:?}",
        silent.len(),
        silent.iter().take(12).collect::<Vec<_>>()
    );
}

#[test]
fn a_new_route_row_changes_routing_with_no_rust_edit() {
    let shipped_text = fs::read_to_string(repo_root().join("data/seed/capability-routing.lino"))
        .expect("capability-routing.lino readable");
    let shipped: Vec<RouteRow> =
        routing_table_from(&shipped_text).expect("the shipped decision table parses");
    assert_eq!(
        shipped.len(),
        routing_table().len(),
        "the shipped table and the parsed document are one table"
    );

    // The triple `(quoted_content, transform, workspace)` has no row today, so
    // it asks. One fixture row makes it route, with no Rust edit.
    let before = route_with(
        &shipped,
        ObjectType::QuotedContent,
        Act::Transform,
        Locus::Workspace,
        ADVERTISED,
    );
    assert!(
        matches!(before, RoutingOutcome::Ask { .. }),
        "a triple with no row asks rather than guessing, got {before:?}"
    );

    let patched_text = format!(
        "{shipped_text}  route\n    object quoted_content\n    act transform\n    locus workspace\n    \
         capability write_file\n    because \"wave T fixture: a route is a seed edit, never a Rust edit\"\n"
    );
    let patched = routing_table_from(&patched_text).expect("the patched decision table parses");
    let after = route_with(
        &patched,
        ObjectType::QuotedContent,
        Act::Transform,
        Locus::Workspace,
        ADVERTISED,
    );
    assert_eq!(
        resolved_capability(&after),
        Some("write_file"),
        "adding one row to data/seed/capability-routing.lino must change routing"
    );
}

#[test]
fn a_verb_synonym_never_changes_the_capability() {
    // For each act, swap the verb across its five-language surfaces and assert
    // the capability is invariant: verbs select the act, never the capability.
    let matrices: [(&str, [&str; 5]); 4] = [
        (
            "web_fetch",
            [
                "fetch https://example.com",
                "загрузи https://example.com",
                "https://example.com लाइए",
                "获取 https://example.com",
                "descarga https://example.com",
            ],
        ),
        (
            "read_file",
            [
                "read notes.txt",
                "прочитай notes.txt",
                "notes.txt पढ़िए",
                "读取 notes.txt",
                "lee notes.txt",
            ],
        ),
        (
            "list_dir",
            [
                "list the files in this folder",
                "перечисли файлы в этой папке",
                "इस फ़ोल्डर की फ़ाइलें सूचीबद्ध करें",
                "列出这个文件夹里的文件",
                "enumera los archivos de esta carpeta",
            ],
        ),
        (
            "calendar_create_event",
            [
                "put a call at 20:00 on Friday",
                "поставь созвон в пятницу на 20:00",
                "शुक्रवार 20:00 पर कॉल रखिए",
                "周五 20:00 安排一次通话",
                "pon una llamada el viernes a las 20:00",
            ],
        ),
    ];
    let mut failures: Vec<String> = Vec::new();
    for (expected, prompts) in matrices {
        for (language, prompt) in ["en", "ru", "hi", "zh", "es"].iter().zip(prompts) {
            let outcome = route(prompt, ADVERTISED);
            if resolved_capability(&outcome) != Some(expected) {
                failures.push(format!(
                    "{language}: {prompt:?} -> {outcome:?}, expected {expected}"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "a verb synonym changed the capability: {failures:?}"
    );
}
