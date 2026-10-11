//! Grounding.
//!
//! Every word of a request starts as an unknown and is grounded from a
//! definition the request gives, a learned chunk, the instruction set's
//! documentation, then the glosses of its dictionary captures, whose own words
//! are grounded one level deeper.

use std::collections::BTreeMap;

use super::phrases::{
    ACTING_ON, ALTERNATIVES, CHAIN, CLAUSE_HEAD, CYCLE, DOCUMENTED, GROUNDED_VIA, MODIFIERS,
    RECALL, REQUEST_DEFINITION, UNDERSTAND, fill,
};
use super::seed::{Hypothesis, meta_seed, sort_hypotheses};
use super::text::{Definition, is_letter, is_number, meta_words, utf16_len};
use super::value::{json_string, to_fixed2};
use super::{BOUNDS, Trace};

/// One dictionary sense of a word.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sense {
    /// The gloss text.
    pub gloss: String,
    /// The URL the gloss was read from.
    pub source_url: String,
}

/// Dictionary senses already fetched, by word. The solver fills it from the
/// concept lookup; tests inject captures.
pub type Knowledge = BTreeMap<String, Vec<Sense>>;

/// A word learned on an earlier turn.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    /// The operation the word denotes.
    pub operation: String,
    /// The evidence it was learned with.
    pub score: f64,
    /// The source it was learned from.
    pub via: String,
}

/// How one word was grounded.
#[derive(Debug, Clone, PartialEq)]
pub struct Grounding {
    /// The word.
    pub word: String,
    /// `grounded`, `defined`, `structural`, `frame`, `value` or `open`.
    pub status: &'static str,
    /// `request`, `chunk`, `documentation`, `degree`, `capture`, `cycle` or
    /// `none`.
    pub origin: &'static str,
    /// Operation hypotheses, strongest first.
    pub hypotheses: Vec<Hypothesis>,
    /// Words a lookup would have to open.
    pub needs: Vec<String>,
    /// The symbol a `value` word names (its gloss quotes it).
    pub value: Option<String>,
    /// The measures a word in a degree of comparison names through its stem.
    pub measures: Vec<Hypothesis>,
}

impl Grounding {
    /// A grounding with the given status and hypotheses, naming no value
    /// and no measure.
    #[must_use]
    pub fn new(
        word: &str,
        status: &'static str,
        origin: &'static str,
        hypotheses: Vec<Hypothesis>,
    ) -> Self {
        Self {
            word: word.to_owned(),
            status,
            origin,
            hypotheses,
            needs: Vec::new(),
            value: None,
            measures: Vec::new(),
        }
    }

    fn open(word: &str, origin: &'static str, needs: Vec<String>) -> Self {
        Self {
            needs,
            ..Self::new(word, "open", origin, Vec::new())
        }
    }

    /// The hypotheses tied with the strongest one.
    ///
    /// Mirrors `top` in `metaClauses` (`js/worker/formal_ai_worker_meta_reasoner.js`).
    #[must_use]
    pub fn top(&self) -> Vec<&Hypothesis> {
        let Some(first) = self.hypotheses.first() else {
            return Vec::new();
        };
        self.hypotheses
            .iter()
            .filter(|hypothesis| hypothesis.score >= first.score * 0.99)
            .collect()
    }
}

/// What grounding reads and where it records its decisions.
pub struct Context<'a> {
    /// The trace recorder.
    pub trace: &'a mut Trace,
    /// The definitions the request gives itself.
    pub definitions: &'a [Definition],
    /// Dictionary senses already fetched.
    pub knowledge: &'a Knowledge,
    /// The request's language.
    pub language: &'a str,
    /// Words a lookup would have to open.
    pub needs: Vec<String>,
    /// Chunks learned on earlier turns.
    pub chunks: &'a BTreeMap<String, Chunk>,
}

fn push_unique(sink: &mut Vec<String>, values: &[String]) {
    for value in values {
        if !sink.contains(value) {
            sink.push(value.clone());
        }
    }
}

fn indent_for(depth: usize) -> String {
    if depth > 0 {
        ["↳".repeat(depth), String::from(" ")].concat()
    } else {
        String::new()
    }
}

/// Ground one word: request definition, learned chunk, documentation, then
/// the glosses of its dictionary captures, whose own words are grounded one
/// level deeper.
///
/// Mirrors `metaGround` in `js/worker/formal_ai_worker_meta_reasoner.js`.
pub fn ground(word: &str, context: &mut Context<'_>, depth: usize, stack: &[String]) -> Grounding {
    let seed = meta_seed();
    let indent = indent_for(depth);
    if stack.iter().any(|item| item == word) {
        let mut chain = stack.to_vec();
        chain.push(word.to_owned());
        context
            .trace
            .emit("impasse", fill(CYCLE, &[&indent, &chain.join(CHAIN)]));
        return Grounding::open(word, "cycle", Vec::new());
    }
    let definitions = context.definitions;
    if let Some(definition) = definitions.iter().find(|item| item.term == word) {
        context.trace.emit(
            "hypothesis",
            fill(REQUEST_DEFINITION, &[&indent, word, &definition.definition]),
        );
        let mut deeper = stack.to_vec();
        deeper.push(word.to_owned());
        let hypotheses = ground_text(&definition.definition, context, depth + 1, &deeper);
        let status = if hypotheses.is_empty() {
            "defined"
        } else {
            "grounded"
        };
        return Grounding::new(word, status, "request", hypotheses);
    }
    let lemmas = seed.lemmas(word, context.language);
    let chunks = context.chunks;
    for lemma in &lemmas {
        if let Some(chunk) = chunks.get(lemma) {
            context.trace.emit(
                "recall",
                fill(RECALL, &[&indent, word, &chunk.operation, &chunk.via]),
            );
            let recalled = Hypothesis {
                operation: chunk.operation.clone(),
                score: chunk.score,
                via: chunk.via.clone(),
            };
            return Grounding::new(word, "grounded", "chunk", vec![recalled]);
        }
    }
    let structural_limit = seed.structural_limit();
    if lemmas
        .iter()
        .any(|lemma| seed.frequency_of(lemma) > structural_limit)
    {
        context.trace.emit(
            "known",
            meta_seed().note("structural", &[("indent", &indent), ("word", word)]),
        );
        return Grounding::new(word, "structural", "documentation", Vec::new());
    }
    let documented = seed.doc_hypotheses(word, context.language);
    if !documented.is_empty() {
        let shown = documented
            .iter()
            .take(3)
            .map(|item| [item.operation.as_str(), ":", &to_fixed2(item.score)].concat())
            .collect::<Vec<_>>()
            .join(ALTERNATIVES);
        context
            .trace
            .emit("hypothesis", fill(DOCUMENTED, &[&indent, word, &shown]));
        let hypotheses = documented
            .into_iter()
            .map(|item| Hypothesis {
                via: String::from("documentation"),
                ..item
            })
            .collect();
        return Grounding::new(word, "grounded", "documentation", hypotheses);
    }
    // A word in a degree of comparison is its stem's measure and the
    // degree's selection ("longest": long, superlative), read before its own
    // entry.
    let (degree, degree_needs) = if depth == 0 {
        ground_degree(word, context, stack, &indent)
    } else {
        (None, Vec::new())
    };
    if let Some(grounding) = degree {
        return grounding;
    }
    let knowledge = context.knowledge;
    let senses = knowledge.get(word).map(Vec::as_slice).unwrap_or_default();
    // Captures ground only the request's own words. A gloss word is grounded
    // from documentation and the request alone, so meaning cannot drift
    // through chains of loosely related senses.
    if !senses.is_empty() && depth == 0 {
        let mut grounding = ground_through_glosses(word, senses, context, depth, stack, &indent);
        if grounding.status == "open" {
            push_unique(&mut grounding.needs, &degree_needs);
        }
        return grounding;
    }
    if depth > 0 {
        return Grounding::open(word, "none", Vec::new());
    }
    if !knowledge.contains_key(word) {
        context.trace.emit(
            "impasse",
            meta_seed().note("lookup_needed", &[("word", word)]),
        );
        let mut needs = vec![word.to_owned()];
        needs.extend(degree_needs);
        return Grounding::open(word, "none", needs);
    }
    Grounding::open(word, "none", degree_needs)
}

/// The first sentence of a gloss: the text before the first `.` or `;`
/// followed by a space or the end.
///
/// Mirrors `gloss.split(/[.;](?:\s|$)/u)[0]` in `metaGlossSymbol`
/// (`js/worker/formal_ai_worker_meta_reasoner.js`).
fn first_sentence(gloss: &str) -> &str {
    let mut characters = gloss.char_indices().peekable();
    while let Some((index, character)) = characters.next() {
        if matches!(character, '.' | ';')
            && characters
                .peek()
                .is_none_or(|(_, next)| next.is_whitespace())
        {
            return &gloss[..index];
        }
    }
    gloss
}

/// The symbol a dictionary gloss defines its word as: a token of the
/// gloss's first sentence that, stripped of enclosing brackets and quotation
/// marks, is one character that is neither a letter, a digit nor a space.
///
/// Mirrors `metaGlossSymbol` in `js/worker/formal_ai_worker_meta_reasoner.js`.
fn gloss_symbol(senses: &[Sense]) -> Option<String> {
    let enclosing = |character: char| {
        matches!(
            character,
            '(' | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '"'
                | '\''
                | '“'
                | '”'
                | '‘'
                | '’'
                | '«'
                | '»'
                | '⟨'
                | '⟩'
        )
    };
    for sense in senses.iter().take(BOUNDS.glosses_per_word) {
        for token in first_sentence(&sense.gloss).split_whitespace() {
            let mut characters = token.trim_matches(enclosing).chars();
            if let (Some(only), None) = (characters.next(), characters.next())
                && !is_letter(only)
                && !is_number(only)
                && !only.is_whitespace()
            {
                return Some(only.to_string());
            }
        }
    }
    None
}

/// A word in a degree of comparison: its stem names a measure, its degree's
/// seeded gloss names the selection by it. Returns the grounding, or the
/// stems a lookup would have to open.
///
/// Mirrors `metaGroundDegree` in `js/worker/formal_ai_worker_meta_reasoner.js`.
fn ground_degree(
    word: &str,
    context: &mut Context<'_>,
    stack: &[String],
    indent: &str,
) -> (Option<Grounding>, Vec<String>) {
    let seed = meta_seed();
    let mut needs: Vec<String> = Vec::new();
    let mut deeper = stack.to_vec();
    deeper.push(word.to_owned());
    for degree in seed.degrees_of(context.language) {
        let gloss = seed.degree_gloss(degree);
        for (ending, replacement) in &degree.endings {
            if gloss.is_empty()
                || utf16_len(word) <= utf16_len(ending) + 2
                || !word.ends_with(ending.as_str())
            {
                continue;
            }
            let stem = [&word[..word.len() - ending.len()], replacement.as_str()].concat();
            if stem == word || seed.is_grammatical(&stem) || stack.contains(&stem) {
                continue;
            }
            let Some(measures) = ground_measure(&stem, context, &deeper, indent) else {
                if !needs.contains(&stem) {
                    needs.push(stem);
                }
                continue;
            };
            if measures.is_empty() {
                continue;
            }
            let hypotheses = ground_text(gloss, context, 1, &deeper);
            if hypotheses.is_empty() {
                continue;
            }
            context.trace.emit(
                "hypothesis",
                seed.note(
                    "degree",
                    &[("word", word), ("stem", &stem), ("degree", &degree.id)],
                ),
            );
            let grounding = Grounding {
                measures,
                ..Grounding::new(word, "grounded", "degree", hypotheses)
            };
            return (Some(grounding), Vec::new());
        }
    }
    (None, needs)
}

/// The measures a word names: its documentation, or the words of its first
/// gloss that reaches one, kept only where they are measures. `None` when
/// the word has neither and was not looked up yet.
///
/// Mirrors `metaGroundMeasure` in `js/worker/formal_ai_worker_meta_reasoner.js`.
fn ground_measure(
    stem: &str,
    context: &mut Context<'_>,
    stack: &[String],
    indent: &str,
) -> Option<Vec<Hypothesis>> {
    let seed = meta_seed();
    let documented: Vec<Hypothesis> = seed
        .doc_hypotheses(stem, context.language)
        .into_iter()
        .filter(|item| seed.is_measure(&item.operation))
        .map(|item| Hypothesis {
            via: String::from("documentation"),
            ..item
        })
        .collect();
    if !documented.is_empty() {
        return Some(documented);
    }
    let knowledge = context.knowledge;
    let senses = knowledge.get(stem)?;
    let mut deeper = stack.to_vec();
    deeper.push(stem.to_owned());
    for sense in senses.iter().take(BOUNDS.glosses_per_word) {
        let mut scores: Vec<(String, f64)> = Vec::new();
        let mut visited_tokens: Vec<String> = Vec::new();
        for token in meta_words(&sense.gloss) {
            if seed.is_grammatical(&token) || visited_tokens.contains(&token) {
                continue;
            }
            for hypothesis in ground(&token, context, 1, &deeper).hypotheses {
                if hypothesis.score < BOUNDS.specific_gloss
                    || !seed.is_measure(&hypothesis.operation)
                {
                    continue;
                }
                if let Some(entry) = scores
                    .iter_mut()
                    .find(|(id, _)| *id == hypothesis.operation)
                {
                    entry.1 = entry.1.max(hypothesis.score);
                } else {
                    scores.push((hypothesis.operation, hypothesis.score));
                }
            }
            visited_tokens.push(token);
        }
        if scores.is_empty() {
            continue;
        }
        let via = if sense.source_url.is_empty() {
            sense.gloss.clone()
        } else {
            sense.source_url.clone()
        };
        let mut measures: Vec<Hypothesis> = scores
            .into_iter()
            .map(|(operation, score)| Hypothesis {
                operation,
                score,
                via: via.clone(),
            })
            .collect();
        sort_hypotheses(&mut measures);
        let shown = measures
            .iter()
            .map(|item| item.operation.as_str())
            .collect::<Vec<_>>()
            .join(ALTERNATIVES);
        context.trace.emit(
            "grounded",
            seed.note(
                "degree_measure",
                &[
                    ("indent", &[indent, "↳ "].concat()),
                    ("stem", stem),
                    ("operations", &shown),
                ],
            ),
        );
        return Some(measures);
    }
    Some(Vec::new())
}

/// The capture branch of `metaGround`: the word names the artifact's frame,
/// or its glosses vouch for operations (Lesk-style support).
///
/// Mirrors the `senses.length && depth === 0` branch of `metaGround` in
/// `js/worker/formal_ai_worker_meta_reasoner.js`.
#[allow(clippy::cast_precision_loss)]
fn ground_through_glosses(
    word: &str,
    senses: &[Sense],
    context: &mut Context<'_>,
    depth: usize,
    stack: &[String],
    indent: &str,
) -> Grounding {
    let seed = meta_seed();
    context.trace.emit(
        "subgoal",
        fill(UNDERSTAND, &[indent, word, &senses.len().to_string()]),
    );
    let frame_markers = seed.cue_markers("artifact");
    let salient = &senses[..senses.len().min(BOUNDS.glosses_per_word)];
    // The word names the artifact's frame when a salient gloss is mostly
    // about it, not when one word of a long gloss happens to match.
    let frame = salient.iter().find(|sense| {
        let gloss_words: Vec<String> = meta_words(&sense.gloss)
            .into_iter()
            .filter(|token| !seed.is_grammatical(token))
            .collect();
        let framing = gloss_words
            .iter()
            .filter(|token| {
                frame_markers
                    .iter()
                    .any(|marker| token.starts_with(marker.trim()))
            })
            .count();
        !gloss_words.is_empty() && framing as f64 / gloss_words.len() as f64 >= BOUNDS.gloss_support
    });
    if let Some(frame) = frame {
        context.trace.emit(
            "grounded",
            meta_seed().note(
                "frame",
                &[("indent", indent), ("word", word), ("gloss", &frame.gloss)],
            ),
        );
        return Grounding::new(word, "frame", "capture", Vec::new());
    }
    let mut scores: Vec<(String, f64)> = Vec::new();
    let mut deeper_needs: Vec<String> = Vec::new();
    let mut via = String::new();
    let mut deeper = stack.to_vec();
    deeper.push(word.to_owned());
    for sense in salient {
        // Lesk-style support: the share of the gloss's gloss_words words whose
        // specific meaning is the operation.
        let gloss_words: Vec<String> = meta_words(&sense.gloss)
            .into_iter()
            .filter(|token| !seed.is_grammatical(token))
            .collect();
        if gloss_words.is_empty() {
            continue;
        }
        let mut unique: Vec<&String> = Vec::new();
        for token in &gloss_words {
            if !unique.contains(&token) {
                unique.push(token);
            }
        }
        let mut support: Vec<(String, usize)> = Vec::new();
        let mut grounded = 0_usize;
        for token in unique {
            let grounding = ground(token, context, depth + 1, &deeper);
            push_unique(&mut deeper_needs, &grounding.needs);
            let mut specific: Vec<&str> = Vec::new();
            for hypothesis in &grounding.hypotheses {
                if hypothesis.score >= BOUNDS.specific_gloss
                    && !specific.contains(&hypothesis.operation.as_str())
                {
                    specific.push(&hypothesis.operation);
                }
            }
            if !specific.is_empty() {
                grounded += 1;
            }
            for operation in specific {
                if let Some(entry) = support.iter_mut().find(|(id, _)| id == operation) {
                    entry.1 += 1;
                } else {
                    support.push((operation.to_owned(), 1));
                }
            }
        }
        for (operation, count) in support {
            // A majority of the gloss's grounded words, and a real share of
            // all its words; words grounded nowhere are neutral.
            let score = count as f64 / gloss_words.len() as f64;
            if score < BOUNDS.gloss_support || count * 2 <= grounded {
                continue;
            }
            let previous = scores
                .iter()
                .find(|(id, _)| *id == operation)
                .map_or(0.0, |(_, value)| *value);
            if score > previous {
                if let Some(entry) = scores.iter_mut().find(|(id, _)| *id == operation) {
                    entry.1 = score;
                } else {
                    scores.push((operation, score));
                }
                let best = scores
                    .iter()
                    .map(|(_, value)| *value)
                    .fold(f64::NEG_INFINITY, f64::max);
                if via.is_empty() || score >= best {
                    via = if sense.source_url.is_empty() {
                        sense.gloss.clone()
                    } else {
                        sense.source_url.clone()
                    };
                }
            }
        }
    }
    let mut hypotheses: Vec<Hypothesis> = scores
        .into_iter()
        .map(|(operation, score)| Hypothesis {
            operation,
            score,
            via: via.clone(),
        })
        .collect();
    sort_hypotheses(&mut hypotheses);
    if let Some(best) = hypotheses.first() {
        context.trace.emit(
            "grounded",
            fill(GROUNDED_VIA, &[indent, word, &best.operation, &via]),
        );
        return Grounding::new(word, "grounded", "capture", hypotheses);
    }
    // A gloss that quotes a symbol defines the word as that symbol: the word
    // names a value, not an operation.
    if let Some(symbol) = gloss_symbol(senses) {
        context.trace.emit(
            "grounded",
            meta_seed().note(
                "symbol",
                &[
                    ("indent", indent),
                    ("word", word),
                    ("value", &json_string(&symbol)),
                ],
            ),
        );
        return Grounding {
            value: Some(symbol),
            ..Grounding::new(word, "value", "capture", Vec::new())
        };
    }
    // Lazy grounding: a gloss word is worth a lookup only when no gloss of
    // its parent reached an operation.
    context.trace.emit(
        "impasse",
        meta_seed().note("no_gloss", &[("indent", indent), ("word", word)]),
    );
    Grounding::open(word, "capture", deeper_needs)
}

/// Ground every content word of a text and merge the operation evidence.
///
/// Mirrors `metaGroundText` in `js/worker/formal_ai_worker_meta_reasoner.js`.
pub fn ground_text(
    text: &str,
    context: &mut Context<'_>,
    depth: usize,
    stack: &[String],
) -> Vec<Hypothesis> {
    let seed = meta_seed();
    let mut scores: Vec<Hypothesis> = Vec::new();
    for word in meta_words(text) {
        if seed.is_grammatical(&word) {
            continue;
        }
        let grounding = ground(&word, context, depth, stack);
        push_unique(&mut context.needs, &grounding.needs);
        for hypothesis in grounding.hypotheses {
            if let Some(previous) = scores
                .iter_mut()
                .find(|item| item.operation == hypothesis.operation)
            {
                if hypothesis.score > previous.score {
                    *previous = hypothesis;
                }
            } else {
                scores.push(hypothesis);
            }
        }
    }
    sort_hypotheses(&mut scores);
    scores
}

/// One coordinated clause of the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    /// The head action's operations.
    pub head: Vec<String>,
    /// The operations of the clause's other words.
    pub others: Vec<String>,
    /// The type the head's object noun denotes.
    pub object_type: Option<String>,
}

/// The request's coordinated clauses, each with its head action (the first
/// grounded word's operations) and the type its object noun denotes.
///
/// Mirrors `metaClauses` in `js/worker/formal_ai_worker_meta_reasoner.js`.
pub fn meta_clauses(text: &str, groundings: &[Grounding], trace: &mut Trace) -> Vec<Clause> {
    let seed = meta_seed();
    let mut parts = vec![[" ", &text.to_lowercase(), " "].concat()];
    for marker in seed.cue_markers("sequence") {
        parts = parts
            .iter()
            .flat_map(|part| {
                part.split(marker.as_str())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect();
    }
    let mut by_word: BTreeMap<&str, &Grounding> = BTreeMap::new();
    for grounding in groundings {
        by_word.insert(&grounding.word, grounding);
    }
    let top = |grounding: &Grounding| -> Vec<String> {
        grounding
            .top()
            .into_iter()
            .map(|hypothesis| hypothesis.operation.clone())
            .collect()
    };
    let mut clauses = Vec::new();
    for part in &parts {
        let grounded: Vec<&Grounding> = meta_words(part)
            .iter()
            .filter_map(|word| by_word.get(word.as_str()).copied())
            .filter(|grounding| !grounding.hypotheses.is_empty())
            .collect();
        if grounded.is_empty() {
            continue;
        }
        // The head is the first specific action: not a representation
        // change, and not a word tied across many operations.
        let Some(head_at) = grounded.iter().position(|grounding| {
            let ids = top(grounding);
            !ids.iter().all(|id| seed.is_view(id)) && ids.len() <= BOUNDS.required_group_size
        }) else {
            continue;
        };
        let head = top(grounded[head_at]);
        let others: Vec<String> = grounded
            .iter()
            .enumerate()
            .filter(|(position, grounding)| {
                *position != head_at && top(grounding).len() <= BOUNDS.required_group_size
            })
            .flat_map(|(_, grounding)| top(grounding))
            .filter(|id| !head.contains(id))
            .collect();
        let noun = grounded[head_at + 1..]
            .iter()
            .find(|grounding| top(grounding).iter().all(|id| seed.is_view(id)));
        let object_type = noun.map(|noun| {
            let plural = seed
                .lemmas(&noun.word, "en")
                .iter()
                .any(|lemma| *lemma != noun.word);
            let split = top(noun)
                .iter()
                .filter_map(|id| seed.primitive(id))
                .find(|primitive| primitive.from == "text")
                .map(|primitive| primitive.to.clone());
            match split {
                Some(to) if plural => to,
                _ => String::from("text"),
            }
        });
        let acting = object_type
            .as_deref()
            .map_or_else(String::new, |kind| fill(ACTING_ON, &[kind]));
        let modifiers = if others.is_empty() {
            String::new()
        } else {
            fill(MODIFIERS, &[&others.join("|")])
        };
        trace.emit(
            "clause",
            fill(CLAUSE_HEAD, &[&head.join("|"), &acting, &modifiers]),
        );
        clauses.push(Clause {
            head,
            others,
            object_type,
        });
    }
    clauses
}
