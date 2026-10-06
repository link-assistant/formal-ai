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
use super::text::{Definition, meta_words};
use super::value::to_fixed2;
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
    /// `grounded`, `defined`, `structural`, `frame` or `open`.
    pub status: &'static str,
    /// `request`, `chunk`, `documentation`, `capture`, `cycle` or `none`.
    pub origin: &'static str,
    /// Operation hypotheses, strongest first.
    pub hypotheses: Vec<Hypothesis>,
    /// Words a lookup would have to open.
    pub needs: Vec<String>,
}

impl Grounding {
    fn open(word: &str, origin: &'static str, needs: Vec<String>) -> Self {
        Self {
            word: word.to_owned(),
            status: "open",
            origin,
            hypotheses: Vec::new(),
            needs,
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
        return Grounding {
            word: word.to_owned(),
            status: if hypotheses.is_empty() {
                "defined"
            } else {
                "grounded"
            },
            origin: "request",
            hypotheses,
            needs: Vec::new(),
        };
    }
    let lemmas = seed.lemmas(word, context.language);
    let chunks = context.chunks;
    for lemma in &lemmas {
        if let Some(chunk) = chunks.get(lemma) {
            context.trace.emit(
                "recall",
                fill(RECALL, &[&indent, word, &chunk.operation, &chunk.via]),
            );
            return Grounding {
                word: word.to_owned(),
                status: "grounded",
                origin: "chunk",
                hypotheses: vec![Hypothesis {
                    operation: chunk.operation.clone(),
                    score: chunk.score,
                    via: chunk.via.clone(),
                }],
                needs: Vec::new(),
            };
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
        return Grounding {
            word: word.to_owned(),
            status: "structural",
            origin: "documentation",
            hypotheses: Vec::new(),
            needs: Vec::new(),
        };
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
        return Grounding {
            word: word.to_owned(),
            status: "grounded",
            origin: "documentation",
            hypotheses: documented
                .into_iter()
                .map(|item| Hypothesis {
                    via: String::from("documentation"),
                    ..item
                })
                .collect(),
            needs: Vec::new(),
        };
    }
    let knowledge = context.knowledge;
    let senses = knowledge.get(word).map(Vec::as_slice).unwrap_or_default();
    // Captures ground only the request's own words. A gloss word is grounded
    // from documentation and the request alone, so meaning cannot drift
    // through chains of loosely related senses.
    if !senses.is_empty() && depth == 0 {
        return ground_through_glosses(word, senses, context, depth, stack, &indent);
    }
    if !knowledge.contains_key(word) && depth == 0 {
        context.trace.emit(
            "impasse",
            meta_seed().note("lookup_needed", &[("word", word)]),
        );
        return Grounding::open(word, "none", vec![word.to_owned()]);
    }
    Grounding::open(word, "none", Vec::new())
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
        return Grounding {
            word: word.to_owned(),
            status: "frame",
            origin: "capture",
            hypotheses: Vec::new(),
            needs: Vec::new(),
        };
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
        return Grounding {
            word: word.to_owned(),
            status: "grounded",
            origin: "capture",
            hypotheses,
            needs: Vec::new(),
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
