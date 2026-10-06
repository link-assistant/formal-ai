//! R1012: the Rust port of the recursive meta reasoner
//! (js/worker/formal_ai_worker_meta_reasoner.js,
//! js/worker/formal_ai_worker_meta_synthesis.js). These mirror
//! rust/tests/web/meta-reasoner.test.mjs: every prompt is absent from the code
//! and the seed; the answers are derived — grounded, enumerated, verified.
//! Dictionary knowledge comes only from the committed captures under
//! rust/tests/fixtures/meta-reasoner and the pre-cached Wiktionary entries
//! under data/cache/wiktionary/en, injected through the lookup callback.

use std::path::PathBuf;

use formal_ai::meta_reasoner::catalog::catalog;
use formal_ai::meta_reasoner::interpreter::{Param, Runtime, run_program};
use formal_ai::meta_reasoner::value::{js_number, to_fixed2};
use formal_ai::meta_reasoner::{
    Knowledge, MetaResult, Sense, Value, import_learned, meta_answer, meta_is_impasse_intent,
    meta_reason, meta_sub_requests,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The Free Dictionary API (Wiktionary) payload read the way the registry's
/// `wiktionary_entry_v1` extractor reads it: entries, meanings, definitions.
fn senses_from(body: &str, url: &str) -> Vec<Sense> {
    let json: serde_json::Value = serde_json::from_str(body.trim()).unwrap_or_default();
    let mut out = Vec::new();
    for entry in json.as_array().map(Vec::as_slice).unwrap_or_default() {
        for meaning in entry["meanings"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
        {
            for definition in meaning["definitions"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default()
            {
                if let Some(gloss) = definition["definition"].as_str() {
                    out.push(Sense {
                        gloss: gloss.split_whitespace().collect::<Vec<_>>().join(" "),
                        source_url: url.to_owned(),
                    });
                }
            }
        }
    }
    out.truncate(12);
    out
}

/// True when a real capture of `word` is committed or pre-cached.
fn captured(word: &str) -> bool {
    let file = format!("{word}.json");
    manifest_dir()
        .join("tests/fixtures/meta-reasoner/captures")
        .join(&file)
        .exists()
        || manifest_dir()
            .join("../data/cache/wiktionary/en")
            .join(&file)
            .exists()
}

/// The dictionary the JavaScript test replays: committed captures first,
/// then the repository's pre-cached Wiktionary entries.
fn dictionary(word: &str, _language: &str) -> Vec<Sense> {
    if word.is_empty() || !word.chars().all(|character| character.is_ascii_lowercase()) {
        return Vec::new();
    }
    let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{word}");
    let file = format!("{word}.json");
    let candidates = [
        manifest_dir()
            .join("tests/fixtures/meta-reasoner/captures")
            .join(&file),
        manifest_dir()
            .join("../data/cache/wiktionary/en")
            .join(&file),
    ];
    for path in candidates {
        if let Ok(body) = std::fs::read_to_string(&path) {
            return senses_from(&body, &url);
        }
    }
    Vec::new()
}

/// The solver's order: the core without lookups first, then the loop with
/// discovery when the core did not solve the request.
fn reason(prompt: &str) -> MetaResult {
    let mut knowledge = Knowledge::new();
    let core = meta_reason(prompt, "en", &mut knowledge, None);
    if core.status == "solved" && core.program.is_some() {
        return core;
    }
    let mut lookup = |word: &str, language: &str| dictionary(word, language);
    let mut knowledge = Knowledge::new();
    meta_reason(prompt, "en", &mut knowledge, Some(&mut lookup))
}

fn events(result: &MetaResult) -> Vec<String> {
    result
        .trace
        .events
        .iter()
        .map(|event| [event.kind.as_str(), ": ", &event.detail].concat())
        .collect()
}

fn steps(result: &MetaResult) -> Vec<String> {
    result
        .program
        .as_ref()
        .map(|program| program.steps.clone())
        .unwrap_or_default()
}

#[test]
fn examples_alone_are_a_goal_state() {
    let result = reason("'ab cd' -> 'ba dc', 'hello' -> 'olleh'");
    assert_eq!(result.goal, "synthesize_from_examples");
    assert_eq!(result.status, "solved", "{}", events(&result).join("\n"));
    assert_eq!(
        steps(&result),
        ["split_words", "each(reverse_text)", "join_words"]
    );
    let verification = result.verification.as_ref().expect("verified");
    assert_eq!((verification.passed, verification.total), (2, 2));
    assert!(
        events(&result)
            .iter()
            .any(|event| event.starts_with("search: length 3")),
        "{}",
        events(&result).join("\n")
    );
}

#[test]
fn a_parameter_is_inferred_from_the_example() {
    let result = reason("[1, 2, 3] -> [3, 6, 9]");
    assert_eq!(steps(&result), ["each(multiply_by)"]);
    let program = result.program.as_ref().expect("program");
    assert_eq!(program.parameter, Param::Number(3.0));
    assert!(program.source.contains("input * 3"), "{}", program.source);
}

#[test]
fn the_rendered_source_is_the_seeded_javascript() {
    let result = reason("'a b c' -> 3");
    let program = result.program.as_ref().expect("program");
    assert!(
        program
            .source
            .starts_with("function solution(input) {\n  let value = input;\n")
    );
    assert!(program.source.ends_with("\n  return value;\n}"));
    for label in &program.steps {
        assert!(
            program.source.contains(&format!("// {label}")),
            "{}",
            program.source
        );
    }
    let answer = meta_answer(&result, false).expect("answer");
    assert_eq!(answer.intent, "meta_reasoned_program");
    assert!(
        answer
            .content
            .contains("```javascript\nfunction solution(input) {")
    );
}

#[test]
fn the_same_request_yields_the_same_derivation() {
    let one = reason("[3, 1, 2] -> [1, 2, 3]");
    let two = reason("[3, 1, 2] -> [1, 2, 3]");
    assert_eq!(one.derivation_lino, two.derivation_lino);
    assert!(one.derivation_lino.starts_with("derivation\n  goal "));
    assert!(
        one.derivation_lino
            .contains("\n  event 1\n    kind impulse\n    detail \"[3, 1, 2] -> [1, 2, 3]\"")
    );
}

#[test]
fn every_rung_of_the_task_ladder_is_derived() {
    let ladder =
        std::fs::read_to_string(manifest_dir().join("tests/fixtures/meta-reasoner/ladder.lino"))
            .expect("ladder fixture");
    let lines: Vec<&str> = ladder.lines().collect();
    let mut rungs = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some(quoted) = line
            .strip_prefix("  rung \"")
            .and_then(|rest| rest.strip_suffix('"'))
        else {
            continue;
        };
        let Some(expected) = lines
            .get(index + 1)
            .and_then(|next| next.strip_prefix("    steps \""))
            .and_then(|rest| rest.strip_suffix('"'))
        else {
            continue;
        };
        // A rung that awaits a word's capture is walked once it is committed.
        let awaited = lines
            .get(index + 2)
            .and_then(|next| next.strip_prefix("    awaits "));
        if awaited.is_some_and(|word| !captured(word)) {
            continue;
        }
        let prompt: String = serde_json::from_str(&format!("\"{quoted}\"")).expect("rung prompt");
        rungs.push((prompt, expected.to_owned()));
    }
    assert!(rungs.len() >= 10, "{} rungs", rungs.len());
    for (prompt, expected) in rungs {
        let result = reason(&prompt);
        let answer = meta_answer(&result, true);
        let intent = answer
            .as_ref()
            .map(|answer| answer.intent.clone())
            .unwrap_or_default();
        assert!(
            intent.starts_with("meta_reasoned_program"),
            "{prompt}: {intent}\n{}",
            events(&result).join("\n")
        );
        assert_eq!(
            steps(&result).join(" "),
            expected,
            "{prompt}\n{}",
            events(&result).join("\n")
        );
        let program_line = format!(
            "\n  program {}",
            serde_json::to_string(&steps(&result).join(" ∘ ")).unwrap_or_default()
        );
        assert!(result.derivation_lino.contains(&program_line), "{prompt}");
    }
}

#[test]
fn a_question_word_is_never_a_defined_term() {
    let asked = reason("What is the capital of Australia?");
    assert_eq!(asked.goal, "explain");
    assert_ne!(asked.status, "solved", "{}", events(&asked).join("\n"));
    let defined = reason("A zorp is a number times two. What is a zorp?");
    assert_eq!(defined.status, "solved", "{}", events(&defined).join("\n"));
}

#[test]
fn a_symbol_a_gloss_defines_is_a_value_the_program_reads() {
    if !captured("colon") {
        return;
    }
    let result = reason("replace every colon with a dash");
    assert_eq!(
        steps(&result),
        ["replace_text"],
        "{}",
        events(&result).join("\n")
    );
    let probe = result.probe.as_ref().expect("probe");
    assert_eq!(probe.input.to_json(), "\"hello:world\"");
}

#[test]
fn a_superlative_is_its_stems_measure_and_the_degrees_selection() {
    if !captured("long") {
        return;
    }
    let result = reason("write a function that returns the longest word");
    assert_eq!(
        steps(&result),
        ["split_words", "maximum_by(text_length)"],
        "{}",
        events(&result).join("\n")
    );
    let probe = result.probe.as_ref().expect("probe");
    assert_eq!(probe.output.to_json(), "\"hello\"");
}

#[test]
fn a_derived_file_program_runs_and_agrees_with_a_direct_count() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let folder = std::env::temp_dir().join(format!("meta-ladder-{}-{unique}", std::process::id()));
    std::fs::create_dir_all(&folder).expect("temp folder");
    std::fs::write(folder.join("long.txt"), "x\n".repeat(5)).expect("long file");
    std::fs::write(folder.join("short.txt"), "x\n").expect("short file");
    let result =
        reason("Write a Node.js script that prints every file in a folder with more than 3 lines");
    let program = result.program.clone().expect("program");
    let folder_text = folder.to_string_lossy().into_owned();
    let output = run_program(
        catalog(),
        &program.step_refs,
        &Value::Path(folder_text.clone()),
        &program.parameter,
        Runtime::Node,
    );
    let _ = std::fs::remove_dir_all(&folder);
    let expected = Value::List(vec![Value::Path(format!("{folder_text}/long.txt"))]);
    assert_eq!(
        output.map(|value| value.to_json()),
        Some(expected.to_json())
    );
}

#[test]
fn a_specialized_handler_impasse_is_recognised_from_the_seed() {
    assert!(meta_is_impasse_intent("calculation_error"));
    assert!(meta_is_impasse_intent("unknown"));
    assert!(meta_is_impasse_intent("write_program_skill_gap"));
    assert!(!meta_is_impasse_intent("greeting"));
}

#[test]
fn a_message_of_several_requests_is_decomposed() {
    let requests = meta_sub_requests(
        "Write a function that reverses each word. Write a function that counts the lines of a text.",
    );
    assert_eq!(requests.len(), 2, "{requests:?}");
    assert_eq!(
        meta_sub_requests("write a function that reverses each word").len(),
        1
    );
}

#[test]
fn learned_chunks_come_back_from_memory() {
    let statement = String::from(
        "meta_learned_chunk\n  word \"qzorpify\"\n  operation reverse_text\n  score 0.5\n  via \"memory\"",
    );
    assert_eq!(import_learned(&[statement.clone()]), 1);
    assert_eq!(import_learned(&[statement]), 0);
}

#[test]
fn learned_chunks_survive_the_memory_log() {
    let statement = String::from(
        "meta_learned_chunk\n  word \"vrelkify\"\n  operation upper_case\n  score 0.5\n  via \"memory\"",
    );
    let event = formal_ai::memory::MemoryEvent {
        id: String::from("learned-1"),
        kind: Some(String::from(formal_ai::memory_sync::LEARNED_CHUNK_KIND)),
        role: Some(String::from("assistant")),
        content: Some(statement.clone()),
        ..formal_ai::memory::MemoryEvent::default()
    };
    let log = formal_ai::memory::export_links_notation(&[event]);
    let events = formal_ai::memory::parse_links_notation(&log);
    let statements = formal_ai::memory_sync::learned_statements(&events);
    assert_eq!(statements, vec![statement], "{log}");
    assert_eq!(import_learned(&statements), 1);
}

#[test]
fn numbers_print_as_javascript_prints_them() {
    assert_eq!(js_number(3.0), "3");
    assert_eq!(js_number(0.1 + 0.2), "0.30000000000000004");
    assert_eq!(js_number(1e21), "1e+21");
    assert_eq!(js_number(-1.5e-7), "-1.5e-7");
    assert_eq!(js_number(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(to_fixed2(0.125), "0.13");
    assert_eq!(to_fixed2(1.0 / 3.0), "0.33");
}
