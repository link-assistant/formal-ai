//! Issue #1186 R10 (with R5): the per-language formalization probe set.
//!
//! `data/benchmarks/formalization/{en,ru,hi,zh}.lino` carries ten or more
//! prompts per language with the expected quantifier, predicates and
//! first-order clause, covering universal, existential, negative and
//! two-place readings. Every probe must formalize to exactly its expected
//! clause, and every target the answer renders (FOL, Lean 4, Rocq, Links
//! Notation) must deformalize back in the probe's own language with the
//! structure preserved. The browser twin is
//! `rust/tests/web/issue-1186-formalization-probe-set.test.mjs`.

use std::fs;
use std::path::Path;

use formal_ai::event_log::EventLog;
use formal_ai::handle_formalization_request;
use formal_ai::seed::parse_lino;
use formal_ai::web_engine_core::normalize_prompt;

const LANGUAGES: [&str; 4] = ["en", "ru", "hi", "zh"];
const SHAPES: [&str; 4] = ["universal", "existential", "negation", "relation"];
const TARGETS: [&str; 4] = ["fol", "lean", "rocq", "lino"];

/// One probe record, read field by field.
struct Probe {
    id: String,
    language: String,
    shape: String,
    prompt: String,
    quantifier: String,
    predicates: Vec<String>,
    fol: String,
}

fn probes(language: &str) -> Vec<Probe> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("data/benchmarks/formalization")
        .join(format!("{language}.lino"));
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    parse_lino(&text)
        .children
        .iter()
        .map(|record| Probe {
            id: record.find_child_value("id").to_owned(),
            language: record.find_child_value("language").to_owned(),
            shape: record.find_child_value("shape").to_owned(),
            prompt: record.find_child_value("prompt").to_owned(),
            quantifier: record.find_child_value("expected_quantifier").to_owned(),
            predicates: record
                .find_child_value("expected_predicates")
                .split('|')
                .map(str::to_owned)
                .collect(),
            fol: record.find_child_value("expected_fol").to_owned(),
        })
        .collect()
}

/// The deformalize cue of each probe language, from the seed's
/// `direction_*` roles.
fn direction_cue(language: &str) -> &'static str {
    match language {
        "ru" => "Деформализуй",
        "hi" => "सहज भाषा में",
        "zh" => "自然语言",
        _ => "Deformalize in plain English",
    }
}

fn handler_answer(prompt: &str) -> String {
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    handle_formalization_request(prompt, &normalized, &mut log)
        .unwrap_or_else(|| panic!("the formalization handler should answer: {prompt}"))
        .answer
}

fn fenced(answer: &str, tag: &str) -> String {
    let open = ["```", tag, "\n"].concat();
    let start = answer
        .find(open.as_str())
        .unwrap_or_else(|| panic!("answer should carry a `{tag}` fence: {answer}"))
        + open.len();
    let end = start
        + answer[start..]
            .find("```")
            .unwrap_or_else(|| panic!("the `{tag}` fence should be closed: {answer}"));
    let block = &answer[start..end];
    block.strip_suffix('\n').unwrap_or(block).to_owned()
}

#[test]
fn every_language_carries_ten_probes_covering_every_shape() {
    for language in LANGUAGES {
        let records = probes(language);
        assert!(
            records.len() >= 10,
            "{language}: the issue asks for ten or more probes, found {}",
            records.len()
        );
        for shape in SHAPES {
            assert!(
                records.iter().any(|probe| probe.shape == shape),
                "{language} lacks a {shape} probe"
            );
        }
        for probe in &records {
            assert_eq!(probe.language, language, "{}", probe.id);
            let symbol = match probe.quantifier.as_str() {
                "forall" => "∀",
                "exists" => "∃",
                "no" => "¬∃",
                other => panic!("{}: unknown quantifier {other}", probe.id),
            };
            assert!(probe.fol.starts_with(symbol), "{}", probe.id);
            assert_eq!(
                probe.shape == "relation",
                probe.fol.contains(", "),
                "{}: a relation probe, and only one, carries an object",
                probe.id
            );
        }
    }
}

#[test]
fn every_probe_formalizes_to_its_expected_clause() {
    for language in LANGUAGES {
        for probe in probes(language) {
            let answer = handler_answer(&probe.prompt);
            let fol = fenced(&answer, "fol");
            assert_eq!(fol, probe.fol, "{}: {answer}", probe.id);
            for predicate in &probe.predicates {
                assert!(
                    fol.contains(&format!("{predicate}(x")),
                    "{} names {predicate}: {fol}",
                    probe.id
                );
            }
        }
    }
}

#[test]
fn every_rendered_target_deformalizes_with_its_structure_preserved() {
    for language in LANGUAGES {
        for probe in probes(language) {
            let answer = handler_answer(&probe.prompt);
            for target in TARGETS {
                let rendered = fenced(&answer, target);
                let back = handler_answer(&format!("{}:\n{rendered}", direction_cue(language)));
                assert!(
                    back.contains("structure preserved"),
                    "{} from {target}: {back}",
                    probe.id
                );
            }
        }
    }
}
