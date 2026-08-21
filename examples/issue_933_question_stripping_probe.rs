//! Report every seed response text that `enforce_questions` rewrites.
//!
//! Issue #933's floor check requires every recorded prompt to show the answer
//! it produces, and three groups came back empty. `seed::localized_response`
//! returns the right text for all of them, so the loss happens later, in the
//! question-necessity pass. This probe replays that pass over every value in
//! `data/seed/multilingual-responses.lino` and prints the ones it does not
//! return byte for byte, which is the list of answers users never see whole.
//!
//!     cargo run --example issue_933_question_stripping_probe

use std::fs;
use std::path::Path;

use formal_ai::event_log::EventLog;
use formal_ai::question_necessity::enforce_questions;

fn unquote(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.starts_with('(') {
        return None;
    }
    let value = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map_or_else(|| trimmed.to_owned(), |value| value.replace("\"\"", "\""));
    // The seed writes multi-paragraph answers with `\n` escapes. The pass
    // treats a newline as a sentence boundary, so leaving them encoded would
    // report paragraphs that are fine at runtime.
    Some(value.replace("\\n", "\n").replace("\\\"", "\""))
}

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/seed/multilingual-responses.lino");
    let seed = fs::read_to_string(&path).expect("multilingual response seed");

    let mut intent = String::new();
    let mut language = String::new();
    let mut rewritten = 0usize;
    for line in seed.lines() {
        let Some((name, raw)) = line.trim().split_once(' ') else {
            continue;
        };
        match name {
            "intent" => raw.trim().trim_matches('"').clone_into(&mut intent),
            "language" => raw.trim().trim_matches('"').clone_into(&mut language),
            "text" | "variant" | "ack_variant" | "follow_up_variant" => {
                let Some(value) = unquote(raw) else { continue };
                let enforced = enforce_questions(&value, &mut EventLog::new());
                if enforced != value {
                    rewritten += 1;
                    println!("{intent}\t{language}\t{name}");
                    println!("  seed:     {value:?}");
                    println!("  enforced: {enforced:?}");
                }
            }
            _ => {}
        }
    }
    println!("{rewritten} seed value(s) rewritten by the question-necessity pass");
}
