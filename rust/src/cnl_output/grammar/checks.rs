//! Sentence splitting, built-in evidence, closed-world truth and prompt analysis.
//! JavaScript root: js/agentic/crate/cnl_grammar.mjs.
use super::{
    Check, CompiledGrammar, Graph, NUMBER, Reading, SharedPolicy, frame_of, graph_key, js_space,
    parse_utterance, raw_tokens, utterance_graphs,
};
use regex::{Captures, Regex};
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, LazyLock};

static PLAIN_SLOT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([a-z]+)\}").expect("fixed CNL logic slot grammar"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Space,
    Opaque,
    Sentence,
}

/// One span of an output text: whitespace, an enumerator, or a sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub text: String,
}

/// The length in characters of a leading `^[0-9]+\)\s+` within `window`.
fn enumerator(window: &[char]) -> usize {
    let digits = window.iter().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 || window.get(digits) != Some(&')') {
        return 0;
    }
    let spaces = window[digits + 1..]
        .iter()
        .take_while(|c| js_space(**c))
        .count();
    if spaces == 0 { 0 } else { digits + 1 + spaces }
}

/// Sentences of an output text with the exact separators between them.
#[must_use]
pub fn output_sentences(text: &str) -> Vec<Piece> {
    const STRONG: &str = "。！？।";
    const ENDS: &str = ".!?。！？।";
    let characters: Vec<char> = text.chars().collect();
    let length = characters.len();
    let mut pieces = Vec::new();
    let mut index = 0;
    let take = |pieces: &mut Vec<Piece>, kind, from: usize, index: usize| {
        if index > from {
            pieces.push(Piece {
                kind,
                text: characters[from..index].iter().collect(),
            });
        }
    };
    while index < length {
        let from = index;
        while index < length && js_space(characters[index]) {
            index += 1;
        }
        take(&mut pieces, PieceKind::Space, from, index);
        let from = index;
        index += enumerator(&characters[index..length.min(index + 8)]);
        take(&mut pieces, PieceKind::Opaque, from, index);
        let from = index;
        while index < length {
            let character = characters[index];
            index += 1;
            let closes = STRONG.contains(character)
                || (".!?".contains(character)
                    && (index == length
                        || js_space(characters[index])
                        || ".!?।".contains(characters[index])));
            if closes {
                while index < length && ENDS.contains(characters[index]) {
                    index += 1;
                }
                if index == length || js_space(characters[index]) || STRONG.contains(character) {
                    break;
                }
            }
        }
        take(&mut pieces, PieceKind::Sentence, from, index);
    }
    pieces
}

fn words(text: &str) -> Vec<String> {
    raw_tokens(text)
        .into_iter()
        .filter(|token| token != "...")
        .collect()
}

fn allowed_by(names: &[String], grammar: &CompiledGrammar, word: &str) -> bool {
    NUMBER.is_match(word)
        || grammar.grammar_words.contains(word)
        || names.iter().any(|name| {
            grammar
                .list_words
                .get(name)
                .is_some_and(|set| set.contains(word))
        })
}

fn readings_allowed(readings: &[Reading], check: &Check, grammar: &CompiledGrammar) -> bool {
    readings.iter().any(|reading| {
        let styled = if check.styles.is_empty() {
            grammar.output_styles.contains(&reading.style)
        } else {
            check.styles.contains(&reading.style)
        };
        styled
            && reading.registers.iter().all(|(_, registers)| {
                check.registers.is_empty()
                    || check
                        .registers
                        .iter()
                        .any(|register| registers.contains(register))
            })
    })
}

/// What a built-in check may ask of the caller.
pub struct CheckContext<'a> {
    /// The frequency tier; `None` compares like JavaScript `null` (zero).
    pub tier: Option<u64>,
    pub count: &'a dyn Fn(&str) -> usize,
    pub minimum: &'a dyn Fn(&Graph) -> Option<usize>,
}

/// Built-in evidence for one named check: `Some(true)` (pass), `Some(false)`
/// (a definite violation) or `None` (outside the closed fragment; unchecked).
#[must_use]
pub fn builtin_check(
    check: &Check,
    text: &str,
    grammar: &CompiledGrammar,
    context: &CheckContext,
) -> Option<bool> {
    let sentences: Vec<String> = output_sentences(text)
        .into_iter()
        .filter(|piece| piece.kind == PieceKind::Sentence)
        .map(|piece| piece.text)
        .collect();
    if sentences.is_empty() {
        return None;
    }
    let parsed: Vec<_> = sentences
        .iter()
        .map(|sentence| parse_utterance(sentence, grammar))
        .collect();
    let distinct: Vec<usize> = parsed
        .iter()
        .map(|readings| {
            let keys: HashSet<String> = readings
                .iter()
                .map(|reading| graph_key(&reading.graph))
                .collect();
            keys.len()
        })
        .collect();
    let all_allowed = || {
        parsed
            .iter()
            .all(|readings| readings_allowed(readings, check, grammar))
    };
    let floor = |word: &str| allowed_by(&["primes".to_owned()], grammar, word);
    match check.kind.as_str() {
        "grammar" => all_allowed().then_some(true),
        "interpretation" => parsed
            .iter()
            .enumerate()
            .all(|(index, readings)| {
                distinct[index] == 1
                    && readings_allowed(readings, check, grammar)
                    && !frame_of(&readings[0].graph.frame, &grammar.shared)
                        .logic
                        .is_empty()
            })
            .then_some(true),
        "references" => {
            let output = parsed.iter().all(|readings| {
                readings
                    .iter()
                    .any(|reading| grammar.output_styles.contains(&reading.style))
            });
            if output {
                return Some(true);
            }
            sentences
                .iter()
                .any(|sentence| {
                    words(sentence)
                        .iter()
                        .any(|word| grammar.pronoun_words.contains(word))
                })
                .then_some(false)
        }
        "ambiguity" => {
            if distinct.iter().any(|count| *count > 1) {
                return Some(false);
            }
            distinct.iter().all(|count| *count == 1).then_some(true)
        }
        "vocabulary" => {
            let pass = sentences.iter().all(|sentence| {
                words(sentence)
                    .iter()
                    .all(|word| allowed_by(&check.lists, grammar, word))
            });
            if pass {
                Some(true)
            } else {
                check.closed.then_some(false)
            }
        }
        "primes" => Some(
            sentences
                .iter()
                .all(|sentence| words(sentence).iter().all(|word| floor(word))),
        ),
        "forbidden" => {
            if grammar.copula_words.is_empty() {
                return None;
            }
            let mut tokens = sentences.iter().flat_map(|sentence| words(sentence));
            if tokens.any(|word| {
                grammar.copula_words.contains(&word)
                    || word.ends_with("'m")
                    || word.ends_with("'re")
            }) {
                return Some(false);
            }
            all_allowed().then_some(true)
        }
        "frequency" => {
            #[allow(clippy::cast_precision_loss)]
            let tier = context.tier.unwrap_or(0) as f64;
            let tokens: Vec<String> = sentences
                .iter()
                .flat_map(|sentence| words(sentence))
                .collect();
            if tokens
                .iter()
                .any(|word| !floor(word) && grammar.rank.get(word).is_some_and(|rank| *rank > tier))
            {
                return Some(false);
            }
            tokens
                .iter()
                .all(|word| floor(word) || grammar.rank.get(word).is_some_and(|rank| *rank <= tier))
                .then_some(true)
        }
        "minimum" => {
            let mut verdict = true;
            for (index, sentence) in sentences.iter().enumerate() {
                if distinct[index] != 1 {
                    return None;
                }
                let least = (context.minimum)(&parsed[index][0].graph)?;
                if (context.count)(sentence) > least {
                    verdict = false;
                }
            }
            Some(verdict)
        }
        _ => None,
    }
}

/// Closed-world truth over declared facts; transitive frames follow chains.
#[must_use]
pub fn graph_holds(graph: &Graph, shared: &SharedPolicy) -> bool {
    let frame = frame_of(&graph.frame, shared);
    let slot_value = |index: usize| {
        frame
            .slots
            .get(index)
            .and_then(|slot| graph.slots.get(slot))
    };
    let start = slot_value(0);
    let goal = slot_value(1);
    let edges: Vec<_> = shared
        .facts
        .iter()
        .filter(|fact| fact.frame == graph.frame)
        .collect();
    let mut seen: HashSet<Option<&str>> = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(current) = queue.pop_front() {
        for fact in &edges {
            if fact.args.first().map(String::as_str) != current {
                continue;
            }
            let next = fact.args.get(1).map(String::as_str);
            if next == goal {
                return true;
            }
            if frame.transitive && seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    false
}

/// First-order reading of a graph for the Links Notation record, or ''.
#[must_use]
pub fn graph_logic(graph: &Graph, shared: &SharedPolicy) -> String {
    let frame = frame_of(&graph.frame, shared);
    let logic = if graph.polarity == "negative" {
        &frame.negative_logic
    } else {
        &frame.logic
    };
    PLAIN_SLOT
        .replace_all(logic, |found: &Captures| {
            graph.slots.get(&found[1]).unwrap_or(&found[1]).to_owned()
        })
        .into_owned()
}

/// The language and graphs of a whole prompt.
#[derive(Debug, Clone)]
pub struct Analysis {
    pub language: String,
    pub grammar: Arc<CompiledGrammar>,
    pub graphs: Vec<Graph>,
}

/// Whole-prompt analysis in the first language with any reading.
#[must_use]
pub fn prompt_analysis(prompt: &str, grammars: &[Arc<CompiledGrammar>]) -> Option<Analysis> {
    let start = prompt.trim_start_matches(js_space);
    if start.starts_with(['`', '"', '\'', '«', '>']) {
        return None;
    }
    grammars.iter().find_map(|grammar| {
        let graphs = utterance_graphs(prompt, grammar);
        (!graphs.is_empty()).then(|| Analysis {
            language: grammar.language().to_owned(),
            grammar: Arc::clone(grammar),
            graphs,
        })
    })
}

/// A task request's canonical solver prompt, filled from its number slots.
#[must_use]
pub fn canonical_prompt(graph: &Graph, shared: &SharedPolicy) -> Option<String> {
    let frame = frame_of(&graph.frame, shared);
    (!frame.canonical.is_empty()).then(|| {
        // JavaScript interpolates a missing slot as "undefined"; keep that identical.
        PLAIN_SLOT
            .replace_all(&frame.canonical, |found: &Captures| {
                graph.slots.get(&found[1]).unwrap_or("undefined").to_owned()
            })
            .into_owned()
    })
}

#[must_use]
pub fn answerable_questions(graphs: &[Graph], shared: &SharedPolicy) -> Vec<Graph> {
    graphs
        .iter()
        .filter(|graph| graph.mood == "question" && frame_of(&graph.frame, shared).answerable)
        .cloned()
        .collect()
}
