//! Questions over prompt-supplied text (issue #1172 R7).
//!
//! "Given the text: '…'. What is X?" and "Read this and answer: «…» When and
//! where is the meeting?" carry their own answer. The handler reads the
//! longest quoted passage, splits it into sentences, and quotes the sentence
//! whose words cover the question's content words (the words left after the
//! seed's function words, interrogatives and text-cue words are removed;
//! CJK questions compare character pairs). No source is consulted and no
//! network call is made. When no sentence covers at least half of the
//! content words, the answer says honestly that the text does not say.
//!
//! An uncued passage counts only when it opens the prompt and is followed by
//! a question, so a quoted phrase inside an instruction ("Translate '…' into
//! French") is never read as a text to answer from. Cues are the
//! `prompt_text_question_cue` meaning of `data/seed/meanings-facts.lino`;
//! the wording is `data/seed/multilingual-responses-entities.lino`. The
//! browser twin is `tryPromptTextQuestion` in
//! `js/worker/formal_ai_worker_factual_qa.js`.

use super::factual_qa::{locate, render_template, role_surfaces};
use super::finalize_simple;
use crate::coding::contains_cjk;
use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;

/// Semantic role: a phrase announcing that the prompt supplies the text.
const ROLE_PROMPT_TEXT_QUESTION_CUE: &str = "prompt_text_question_cue";
/// Fewest words an uncued quoted passage needs to count as a text to read.
const MIN_PASSAGE_WORDS: usize = 6;
/// Shortest shared stem two inflected tokens need to count as one word.
const MIN_STEM: usize = 4;
/// Opening and closing quote marks a passage may be wrapped in.
const QUOTE_PAIRS: [(char, char); 7] = [
    ('«', '»'),
    ('“', '”'),
    ('„', '“'),
    ('「', '」'),
    ('『', '』'),
    ('"', '"'),
    ('\'', '\''),
];
/// Punctuation stripped between the passage and its question.
const QUESTION_LEAD: [char; 16] = [
    '.', ',', ';', ':', '!', '?', '。', '，', '；', '：', '！', '？', ')', ']', '—', '-',
];

/// A quoted passage: its trimmed body and its byte span in the prompt.
struct Passage<'a> {
    body: &'a str,
    start: usize,
    end: usize,
}

/// The first `close` at or after `from`; an ASCII apostrophe followed by a
/// letter ("don't") is part of a word, not a closing quote.
fn closing_quote(text: &str, from: usize, close: char, apostrophe: bool) -> Option<usize> {
    let mut cursor = from;
    loop {
        let end = cursor + text[cursor..].find(close)?;
        let after = end + close.len_utf8();
        if !apostrophe
            || !text[after..]
                .chars()
                .next()
                .is_some_and(char::is_alphabetic)
        {
            return Some(end);
        }
        cursor = after;
    }
}

/// The longest quoted passage of the prompt.
fn longest_passage(prompt: &str) -> Option<Passage<'_>> {
    let mut best: Option<Passage<'_>> = None;
    for (open, close) in QUOTE_PAIRS {
        let apostrophe = open == '\'';
        let mut cursor = 0;
        while let Some(offset) = prompt[cursor..].find(open) {
            let start = cursor + offset;
            let body_start = start + open.len_utf8();
            let Some(end) = closing_quote(prompt, body_start, close, apostrophe) else {
                break;
            };
            let body = prompt[body_start..end].trim();
            let letter_before = prompt[..start]
                .chars()
                .next_back()
                .is_some_and(char::is_alphabetic);
            if (!apostrophe || !letter_before)
                && !body.is_empty()
                && best
                    .as_ref()
                    .is_none_or(|known| body.chars().count() > known.body.chars().count())
            {
                best = Some(Passage {
                    body,
                    start,
                    end: end + close.len_utf8(),
                });
            }
            cursor = end + close.len_utf8();
        }
    }
    best
}

/// The passage's sentences, terminators kept: a Latin terminator ends a
/// sentence only before whitespace, a CJK one always does.
fn sentences(passage: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut characters = passage.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        let end = index + character.len_utf8();
        let boundary = matches!(character, '。' | '！' | '？')
            || (matches!(character, '.' | '!' | '?')
                && characters
                    .peek()
                    .is_some_and(|(_, next)| next.is_whitespace()));
        if boundary {
            out.push(passage[start..end].trim());
            start = end;
        }
    }
    out.push(passage[start..].trim());
    out.retain(|sentence| !sentence.is_empty());
    out
}

/// Do two tokens name the same word (equal, or one inflected stem)?
fn tokens_match(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let shorter = left.chars().count().min(right.chars().count());
    if shorter < MIN_STEM {
        return false;
    }
    let common = left
        .chars()
        .zip(right.chars())
        .take_while(|(a, b)| a == b)
        .count();
    common >= MIN_STEM.max(shorter - 2)
}

/// The question's content units: words (CJK: character pairs) left after
/// the function words, interrogatives and text-cue words are removed.
fn content_units(question: &str) -> Vec<String> {
    let mut stop = role_surfaces(seed::ROLE_STATEMENT_FUNCTION_WORD);
    stop.extend(role_surfaces(seed::ROLE_INTERROGATIVE_OPENER));
    for cue in role_surfaces(ROLE_PROMPT_TEXT_QUESTION_CUE) {
        stop.extend(cue.split(' ').map(str::to_owned));
    }
    let mut normalized = normalize_prompt(question);
    if !contains_cjk(&normalized) {
        return normalized
            .split(' ')
            .filter(|word| !word.is_empty() && !stop.iter().any(|known| known.as_str() == *word))
            .map(str::to_owned)
            .collect();
    }
    for surface in stop.iter().filter(|surface| contains_cjk(surface)) {
        normalized = normalized.replace(surface.as_str(), " ");
    }
    let mut units = Vec::new();
    for run in normalized.split_whitespace() {
        let characters: Vec<char> = run.chars().collect();
        if characters.len() == 1 {
            units.push(run.to_owned());
        }
        for pair in characters.windows(2) {
            units.push(pair.iter().collect());
        }
    }
    units
}

/// The content units a sentence covers.
fn covered<'a>(sentence: &str, units: &'a [String]) -> Vec<&'a str> {
    let normalized = normalize_prompt(sentence);
    units
        .iter()
        .filter(|unit| {
            if contains_cjk(unit) {
                normalized.contains(unit.as_str())
            } else {
                normalized.split(' ').any(|token| tokens_match(token, unit))
            }
        })
        .map(String::as_str)
        .collect()
}

/// Answer a question over a quoted, prompt-supplied passage from the
/// passage's own sentences — never from a source, never over the network.
pub fn try_prompt_text_question(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let passage = longest_passage(prompt)?;
    let lead = normalize_prompt(&prompt[..passage.start]);
    let question = prompt[passage.end..]
        .trim_start_matches(|character: char| {
            character.is_whitespace() || QUESTION_LEAD.contains(&character)
        })
        .trim();
    if question.is_empty() {
        return None;
    }
    let cued = role_surfaces(ROLE_PROMPT_TEXT_QUESTION_CUE)
        .iter()
        .any(|surface| locate(&lead, surface).is_some());
    let question_normalized = normalize_prompt(question);
    let asks = question.ends_with(['?', '？'])
        || role_surfaces(seed::ROLE_INTERROGATIVE_OPENER)
            .iter()
            .any(|surface| locate(&question_normalized, surface).is_some());
    let passage_normalized = normalize_prompt(passage.body);
    let passage_words = if contains_cjk(&passage_normalized) {
        passage_normalized.chars().count() / 2
    } else {
        passage_normalized.split(' ').count()
    };
    if !asks || (!cued && (!lead.is_empty() || passage_words < MIN_PASSAGE_WORDS)) {
        return None;
    }
    let language = detect_language(question).slug();
    let sentences = sentences(passage.body);
    let units = content_units(question);
    let mut best: Option<(usize, Vec<&str>)> = None;
    for (index, sentence) in sentences.iter().enumerate() {
        let hits = covered(sentence, &units);
        if best
            .as_ref()
            .is_none_or(|(_, known)| hits.len() > known.len())
        {
            best = Some((index, hits));
        }
    }
    log.append("prompt_text:sentences", sentences.len().to_string());
    log.append("prompt_text:network", String::from("none"));
    let answered = best.as_ref().filter(|(_, hits)| {
        if units.is_empty() {
            sentences.len() == 1
        } else {
            !hits.is_empty() && hits.len() * 2 >= units.len()
        }
    });
    let Some((index, hits)) = answered else {
        let focus = if units.is_empty() {
            question.to_owned()
        } else {
            units.join(", ")
        };
        log.append("prompt_text:gap", focus.clone());
        let body = render_template("prompt_text_gap", language, &[("focus", focus.as_str())]);
        return Some(finalize_simple(
            prompt,
            log,
            "prompt_text_gap",
            "response:prompt_text_gap",
            &body,
            0.7,
        ));
    };
    log.append("prompt_text:selected", (index + 1).to_string());
    log.append("prompt_text:covered", hits.join(","));
    let body = render_template(
        "prompt_text_answer",
        language,
        &[("sentence", sentences[*index])],
    );
    Some(finalize_simple(
        prompt,
        log,
        "prompt_text_answer",
        "response:prompt_text_answer",
        &body,
        0.85,
    ))
}
