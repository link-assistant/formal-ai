//! Controlled-language grammar (issue #1584). One declarative description per
//! language drives realization, parsing (its exact inverse) and validation.
//! JavaScript root: js/agentic/crate/cnl_grammar.mjs.
mod checks;
mod graph;

pub use checks::{
    Analysis, CheckContext, Piece, PieceKind, answerable_questions, builtin_check,
    canonical_prompt, graph_holds, graph_logic, output_sentences, prompt_analysis,
};
pub use graph::{Graph, Slots, graph_key};

use crate::seed::parser::{LinoNode, parse_lino};
use regex::Regex;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

pub const CNL_LANGUAGE_PATH: &str = "data/seed/cnl-language-{language}.lino";
/// Term register of input-only surfaces; realization never selects it.
pub const INPUT_REGISTER: &str = "input";
const MAXIMUM_SPAN: usize = 24;
const PARSE_CACHE_LIMIT: usize = 4096;

pub(crate) static TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\.\.\.|…|\p{Script=Han}|[0-9]+(?:\.[0-9]+)?|[\p{L}\p{M}\p{N}]+(?:['-][\p{L}\p{M}\p{N}]+)*",
    )
    .expect("fixed CNL token grammar")
});
pub(crate) static NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9]+(?:\.[0-9]+)?$").expect("fixed CNL number grammar"));
static SLOT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([a-z]+)\.([a-z-]+)\}").expect("fixed CNL slot grammar"));
static LETTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\p{L}").expect("fixed CNL letter grammar"));

/// JavaScript `\s` (and `String.prototype.trim`): differs from Rust's
/// `char::is_whitespace` on U+0085 (not space) and U+FEFF (space).
#[must_use]
pub fn js_space(character: char) -> bool {
    matches!(
        character,
        '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// JavaScript `String.prototype.trim`.
#[must_use]
pub fn js_trim(text: &str) -> &str {
    text.trim_matches(js_space)
}

pub(crate) fn list(value: &str) -> Vec<String> {
    value
        .split('|')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

/// JavaScript `Number(text)` for decimal text: blank is zero, junk is NaN.
fn js_number(text: &str) -> f64 {
    let trimmed = js_trim(text);
    if trimmed.is_empty() {
        0.0
    } else {
        trimmed.parse().unwrap_or(f64::NAN)
    }
}

fn keyed(value: &str) -> (String, Vec<String>) {
    value.find(": ").map_or_else(
        || (value.to_owned(), Vec::new()),
        |at| (value[..at].to_owned(), list(&value[at + 2..])),
    )
}

fn children_named<'a>(node: &'a LinoNode, name: &'a str) -> impl Iterator<Item = &'a LinoNode> {
    node.children.iter().filter(move |child| child.name == name)
}

/// A shared meaning frame: its typed slots and first-order reading.
#[derive(Debug, Clone, Default)]
pub struct Frame {
    pub id: String,
    pub slots: Vec<String>,
    pub types: Vec<String>,
    pub transitive: bool,
    pub answerable: bool,
    pub expands: Vec<String>,
    pub canonical: String,
    pub logic: String,
    pub negative_logic: String,
}

#[derive(Debug, Clone)]
pub struct Fact {
    pub frame: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Check {
    pub id: String,
    pub kind: String,
    pub styles: Vec<String>,
    pub registers: Vec<String>,
    pub lists: Vec<String>,
    pub closed: bool,
}

/// Shared frames, facts and checks; independent of any language.
#[derive(Debug, Clone, Default)]
pub struct SharedPolicy {
    pub frames: Vec<Frame>,
    pub facts: Vec<Fact>,
    pub checks: Vec<Check>,
    pub registers: Vec<String>,
    pub input_styles: Vec<String>,
    pub precision_registers: Vec<String>,
    pub precision_styles: Vec<String>,
}

#[must_use]
pub fn parse_grammar_policy(root: &LinoNode) -> SharedPolicy {
    let value = |node: &LinoNode, name: &str| node.find_child_value(name).to_owned();
    SharedPolicy {
        frames: children_named(root, "frame")
            .map(|node| Frame {
                id: node.id.clone(),
                slots: list(node.find_child_value("slots")),
                types: list(node.find_child_value("types")),
                transitive: node.find_child_value("transitive") == "true",
                answerable: node.find_child_value("answerable") == "true",
                expands: list(node.find_child_value("expands")),
                canonical: value(node, "canonical"),
                logic: value(node, "logic"),
                negative_logic: value(node, "negative-logic"),
            })
            .collect(),
        facts: children_named(root, "fact")
            .map(|node| {
                let mut words = node.id.split(' ').map(str::to_owned);
                Fact {
                    frame: words.next().unwrap_or_default(),
                    args: words.collect(),
                }
            })
            .collect(),
        checks: children_named(root, "check")
            .map(|node| Check {
                id: node.id.clone(),
                kind: value(node, "kind"),
                styles: list(node.find_child_value("styles")),
                registers: list(node.find_child_value("registers")),
                lists: list(node.find_child_value("lists")),
                closed: node.find_child_value("closed") == "true",
            })
            .collect(),
        registers: list(root.find_child_value("registers")),
        input_styles: list(root.find_child_value("input-styles")),
        precision_registers: list(root.find_child_value("precision-registers")),
        precision_styles: list(root.find_child_value("precision-styles")),
    }
}

/// One concept's forms per register, each list indexed by the language's keys.
#[derive(Debug, Clone)]
pub struct Term {
    pub id: String,
    pub registers: Vec<(String, Vec<String>)>,
}

impl Term {
    /// Like `Object.fromEntries`: a repeated register name keeps its last value.
    #[must_use]
    pub fn register(&self, name: &str) -> Option<&[String]> {
        self.registers
            .iter()
            .rev()
            .find(|(register, _)| register == name)
            .map(|(_, forms)| forms.as_slice())
    }
}

#[derive(Debug, Clone)]
pub struct TemplateData {
    pub frame: String,
    pub styles: Vec<String>,
    pub mood: String,
    pub positive: Vec<String>,
    pub negative: Vec<String>,
}

/// One `cnl-language-<id>.lino` file as plain data.
#[derive(Debug, Clone)]
pub struct LanguageGrammar {
    pub id: String,
    pub keys: Vec<String>,
    pub grammar: Vec<String>,
    pub copula: Vec<String>,
    pub pronouns: Vec<String>,
    pub clause_close: String,
    /// Between two sentences of one reply: a space, or nothing (Chinese 。).
    pub sentence_space: String,
    pub fillers: Vec<String>,
    pub typos: Vec<Vec<String>>,
    pub loose: Vec<Vec<String>>,
    pub primes: Vec<(String, Vec<String>)>,
    pub lists: Vec<(String, Vec<String>)>,
    pub tiers: Vec<(f64, Vec<String>)>,
    pub terms: Vec<Term>,
    pub templates: Vec<TemplateData>,
}

fn pairs(value: &str) -> Vec<Vec<String>> {
    list(value)
        .iter()
        .map(|pair| pair.split('=').map(str::to_owned).collect())
        .collect()
}

/// Parse one cnl-language-<id>.lino file into plain data.
pub fn parse_language_grammar(text: &str) -> Result<LanguageGrammar, &'static str> {
    let parsed = parse_lino(text);
    let root = if parsed.name == "cnl-language" {
        &parsed
    } else {
        children_named(&parsed, "cnl-language")
            .next()
            .ok_or("cnl_language_missing")?
    };
    let keyed_children =
        |name: &'static str| children_named(root, name).map(|node| keyed(&node.id));
    Ok(LanguageGrammar {
        id: root.id.clone(),
        keys: list(root.find_child_value("keys")),
        grammar: list(root.find_child_value("grammar")),
        copula: list(root.find_child_value("copula")),
        pronouns: list(root.find_child_value("pronouns")),
        clause_close: root.find_child_value("clause-close").to_owned(),
        sentence_space: if children_named(root, "sentence-space").next().is_some() {
            root.find_child_value("sentence-space").to_owned()
        } else {
            " ".to_owned()
        },
        fillers: list(root.find_child_value("fillers")),
        typos: pairs(root.find_child_value("typos")),
        loose: pairs(root.find_child_value("loose")),
        primes: keyed_children("prime").collect(),
        lists: keyed_children("list").collect(),
        tiers: keyed_children("tier")
            .map(|(bound, words)| (js_number(&bound), words))
            .collect(),
        terms: children_named(root, "term")
            .map(|node| Term {
                id: node.id.clone(),
                registers: node
                    .children
                    .iter()
                    .map(|child| {
                        let forms = child.id.split('|').map(str::to_owned).collect();
                        (child.name.clone(), forms)
                    })
                    .collect(),
            })
            .collect(),
        templates: children_named(root, "template")
            .map(|node| {
                let mut head = node.id.split(' ');
                let texts = |name| children_named(node, name).map(|item| item.id.clone());
                TemplateData {
                    frame: head.next().unwrap_or_default().to_owned(),
                    styles: list(head.next().unwrap_or_default()),
                    mood: head.next().unwrap_or_default().to_owned(),
                    positive: texts("positive").collect(),
                    negative: texts("negative").collect(),
                }
            })
            .collect(),
    })
}

#[derive(Debug, Clone)]
enum Part {
    Literal(Vec<String>),
    Slot { slot: String, key: String },
}

#[derive(Debug, Clone)]
pub struct Template {
    pub data: TemplateData,
    positive: Vec<Vec<Part>>,
    negative: Vec<Vec<Part>>,
}

#[derive(Debug, Clone)]
struct Entry {
    concept: String,
    key: Option<String>,
    register: String,
}

/// One reading of an utterance: its graph, template style and slot registers.
#[derive(Debug, Clone)]
pub struct Reading {
    pub graph: Graph,
    pub style: String,
    pub registers: Vec<(String, Vec<String>)>,
}

/// Lookup tables built once per language; caches live on the compiled grammar.
#[derive(Debug)]
pub struct CompiledGrammar {
    pub data: LanguageGrammar,
    pub shared: Arc<SharedPolicy>,
    typos: HashMap<String, Vec<String>>,
    fillers: Vec<Vec<String>>,
    index: HashMap<String, Vec<Entry>>,
    loose_index: HashMap<String, Vec<Entry>>,
    loose_words: HashMap<String, Option<String>>,
    pub prime_words: HashSet<String>,
    pub grammar_words: HashSet<String>,
    pub copula_words: HashSet<String>,
    pub pronoun_words: HashSet<String>,
    pub list_words: HashMap<String, HashSet<String>>,
    pub rank: HashMap<String, f64>,
    term_by_id: HashMap<String, usize>,
    pub templates: Vec<Template>,
    pub output_styles: Vec<String>,
    parse_cache: Mutex<HashMap<String, Arc<Vec<Reading>>>>,
    pub(crate) minimum_cache: Mutex<HashMap<String, Option<usize>>>,
}

/// Lowercased tokens without typo or filler handling (the checks' word view).
pub(crate) fn raw_tokens(text: &str) -> Vec<String> {
    let lower = text.to_lowercase().replace('’', "'");
    TOKEN
        .find_iter(&lower)
        .map(|item| item.as_str().to_owned())
        .collect()
}

impl CompiledGrammar {
    #[must_use]
    pub fn language(&self) -> &str {
        &self.data.id
    }

    /// Normalize like the parser: lowercase, typographic apostrophes, typos, fillers.
    #[must_use]
    pub fn tokens(&self, text: &str) -> Vec<String> {
        let mut fixed = Vec::new();
        for token in raw_tokens(text) {
            let token = if token == "…" {
                "...".to_owned()
            } else {
                token
            };
            match self.typos.get(&token) {
                Some(words) => fixed.extend(words.iter().cloned()),
                None => fixed.push(token),
            }
        }
        let mut tokens = Vec::new();
        let mut index = 0;
        while index < fixed.len() {
            let filler = self.fillers.iter().find(|words| {
                words
                    .iter()
                    .enumerate()
                    .all(|(offset, word)| fixed.get(index + offset) == Some(word))
            });
            if let Some(filler) = filler {
                index += filler.len();
            } else {
                tokens.push(fixed[index].clone());
                index += 1;
            }
        }
        tokens
    }

    fn loose_word<'a>(&'a self, token: &'a str) -> &'a str {
        match self.loose_words.get(token) {
            Some(Some(word)) => word,
            _ => token,
        }
    }

    /// Input-only spelling equivalences (for example English "a"/"an").
    fn loose_key<S: AsRef<str>>(&self, tokens: &[S]) -> String {
        tokens
            .iter()
            .map(|token| self.loose_word(token.as_ref()))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn compile_template(&self, text: &str) -> Vec<Part> {
        let mut parts = Vec::new();
        let mut last = 0;
        for found in SLOT.captures_iter(text) {
            let whole = found.get(0).expect("whole match");
            parts.push(Part::Literal(self.tokens(&text[last..whole.start()])));
            parts.push(Part::Slot {
                slot: found[1].to_owned(),
                key: found[2].to_owned(),
            });
            last = whole.end();
        }
        parts.push(Part::Literal(self.tokens(&text[last..])));
        parts
    }

    #[must_use]
    pub fn term(&self, id: &str) -> Option<&Term> {
        self.term_by_id
            .get(id)
            .map(|index| &self.data.terms[*index])
    }
}

/// Build lookup tables once per language.
#[must_use]
pub fn compile_grammar(data: LanguageGrammar, shared: Arc<SharedPolicy>) -> CompiledGrammar {
    let words = |text: &String| raw_tokens(text);
    let mut grammar = CompiledGrammar {
        typos: HashMap::new(),
        fillers: Vec::new(),
        index: HashMap::new(),
        loose_index: HashMap::new(),
        loose_words: data
            .loose
            .iter()
            .map(|pair| (pair[0].clone(), pair.get(1).cloned()))
            .collect(),
        prime_words: data
            .primes
            .iter()
            .flat_map(|(_, forms)| forms.iter().flat_map(words))
            .collect(),
        grammar_words: data.grammar.iter().flat_map(words).collect(),
        copula_words: data.copula.iter().cloned().collect(),
        pronoun_words: data.pronouns.iter().cloned().collect(),
        list_words: HashMap::new(),
        rank: HashMap::new(),
        term_by_id: data
            .terms
            .iter()
            .enumerate()
            .map(|(index, term)| (term.id.clone(), index))
            .collect(),
        templates: Vec::new(),
        output_styles: Vec::new(),
        parse_cache: Mutex::new(HashMap::new()),
        minimum_cache: Mutex::new(HashMap::new()),
        shared,
        data,
    };
    for pair in &grammar.data.typos {
        if let Some(right) = pair.get(1) {
            let right = right.split(' ').map(str::to_owned).collect();
            grammar.typos.insert(pair[0].clone(), right);
        }
    }
    let mut fillers: Vec<Vec<String>> = grammar
        .data
        .fillers
        .iter()
        .map(|filler| filler.split(' ').map(str::to_owned).collect())
        .collect();
    fillers.sort_by_key(|filler| std::cmp::Reverse(filler.len()));
    grammar.fillers = fillers;
    for (name, forms) in &grammar.data.lists {
        let set = forms.iter().flat_map(words).collect();
        grammar.list_words.insert(name.clone(), set);
    }
    grammar
        .list_words
        .insert("primes".to_owned(), grammar.prime_words.clone());
    let mut tiers = grammar.data.tiers.clone();
    tiers.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (bound, forms) in &tiers {
        for word in forms.iter().flat_map(words) {
            grammar.rank.entry(word).or_insert(*bound);
        }
    }
    let mut index: HashMap<String, Vec<Entry>> = HashMap::new();
    let mut loose_index: HashMap<String, Vec<Entry>> = HashMap::new();
    // The input register lists accepted surfaces that are never realized: a
    // shared colloquial word (Chinese 鼠, Hindi मूषक) reads as every term that
    // declares it, so a question with it gets a clarification.
    let registers: Vec<&str> = grammar
        .shared
        .registers
        .iter()
        .map(String::as_str)
        .chain([INPUT_REGISTER])
        .collect();
    for term in &grammar.data.terms {
        for register in &registers {
            let forms = if *register == INPUT_REGISTER {
                term.register(INPUT_REGISTER).unwrap_or_default()
            } else {
                term.register(register)
                    .or_else(|| term.register("forms"))
                    .unwrap_or_default()
            };
            for (position, form) in forms.iter().enumerate() {
                let key = grammar.tokens(form).join(" ");
                let entry = Entry {
                    concept: term.id.clone(),
                    key: grammar.data.keys.get(position).cloned(),
                    register: (*register).to_owned(),
                };
                let loose = grammar.loose_key(&key.split(' ').collect::<Vec<_>>());
                index.entry(key).or_default().push(entry.clone());
                loose_index.entry(loose).or_default().push(entry);
            }
        }
    }
    grammar.index = index;
    grammar.loose_index = loose_index;
    grammar.templates = grammar
        .data
        .templates
        .iter()
        .map(|template| Template {
            data: template.clone(),
            positive: template
                .positive
                .iter()
                .map(|text| grammar.compile_template(text))
                .collect(),
            negative: template
                .negative
                .iter()
                .map(|text| grammar.compile_template(text))
                .collect(),
        })
        .collect();
    let mut styles: Vec<String> = Vec::new();
    for style in grammar.data.templates.iter().flat_map(|item| &item.styles) {
        if !styles.contains(style) && !grammar.shared.input_styles.contains(style) {
            styles.push(style.clone());
        }
    }
    grammar.output_styles = styles;
    grammar
}

/// The declared frame, or an empty frame carrying only the id.
pub fn frame_of<'a>(id: &str, shared: &'a SharedPolicy) -> Cow<'a, Frame> {
    shared
        .frames
        .iter()
        .find(|frame| frame.id == id)
        .map_or_else(
            || {
                Cow::Owned(Frame {
                    id: id.to_owned(),
                    ..Frame::default()
                })
            },
            Cow::Borrowed,
        )
}

#[derive(Debug, Clone)]
enum Bound {
    Number(String),
    Concept {
        concept: String,
        registers: Vec<String>,
    },
}

type Binding = Vec<(String, Bound)>;

fn bind(bound: &Binding, slot: &str, value: Bound) -> Binding {
    let mut next = bound.clone();
    match next.iter_mut().find(|(name, _)| name == slot) {
        Some(entry) => entry.1 = value,
        None => next.push((slot.to_owned(), value)),
    }
    next
}

fn bound_of<'a>(bound: &'a Binding, slot: &str) -> Option<&'a Bound> {
    bound
        .iter()
        .find(|(name, _)| name == slot)
        .map(|(_, value)| value)
}

struct Matcher<'a> {
    parts: &'a [Part],
    tokens: &'a [String],
    grammar: &'a CompiledGrammar,
    lenient: bool,
    results: Vec<Binding>,
}

impl Matcher<'_> {
    fn same(&self, left: &str, right: Option<&String>) -> bool {
        if right.is_some_and(|right| right == left) {
            return true;
        }
        self.lenient
            && self.grammar.loose_word(left)
                == self.grammar.loose_word(right.map_or("", String::as_str))
    }

    /// Bind template parts to tokens. Lenient (input-only) templates accept any
    /// form key and the declared loose equivalences; output templates are exact.
    fn walk(&mut self, part: usize, at: usize, bound: &Binding) {
        if part == self.parts.len() {
            if at == self.tokens.len() {
                self.results.push(bound.clone());
            }
            return;
        }
        match &self.parts[part] {
            Part::Literal(literal) => {
                let matches = literal
                    .iter()
                    .enumerate()
                    .all(|(offset, word)| self.same(word, self.tokens.get(at + offset)));
                if matches {
                    self.walk(part + 1, at + literal.len(), bound);
                }
            }
            Part::Slot { slot, key } if key == "number" => {
                let token = self.tokens.get(at).map_or("", String::as_str);
                if NUMBER.is_match(token) {
                    let next = bind(bound, slot, Bound::Number(token.to_owned()));
                    self.walk(part + 1, at + 1, &next);
                }
            }
            Part::Slot { slot, key } => {
                let last = self.tokens.len().min(at + MAXIMUM_SPAN);
                for end in at + 1..=last {
                    let span = &self.tokens[at..end];
                    let entries: Vec<&Entry> = if self.lenient {
                        let name = self.grammar.loose_key(span);
                        self.grammar
                            .loose_index
                            .get(&name)
                            .into_iter()
                            .flatten()
                            .collect()
                    } else {
                        let name = span.join(" ");
                        self.grammar
                            .index
                            .get(&name)
                            .into_iter()
                            .flatten()
                            .filter(|entry| entry.key.as_deref() == Some(key.as_str()))
                            .collect()
                    };
                    let mut concepts: Vec<(String, Vec<String>)> = Vec::new();
                    for entry in entries {
                        match concepts.iter_mut().find(|(id, _)| *id == entry.concept) {
                            Some((_, registers)) => {
                                if !registers.contains(&entry.register) {
                                    registers.push(entry.register.clone());
                                }
                            }
                            None => {
                                concepts.push((entry.concept.clone(), vec![entry.register.clone()]))
                            }
                        }
                    }
                    for (concept, registers) in concepts {
                        let next = bind(bound, slot, Bound::Concept { concept, registers });
                        self.walk(part + 1, end, &next);
                    }
                }
            }
        }
    }
}

fn slots_typed(frame: &Frame, bound: &Binding) -> bool {
    frame.slots.iter().enumerate().all(|(index, slot)| {
        let number = frame.types.get(index).is_some_and(|kind| kind == "number");
        match bound_of(bound, slot) {
            Some(Bound::Number(_)) => number,
            Some(Bound::Concept { .. }) => !number,
            None => false,
        }
    })
}

/// Every reading of one utterance. A statement template ending in a question
/// mark reads as the corresponding question.
pub fn parse_utterance(text: &str, grammar: &CompiledGrammar) -> Arc<Vec<Reading>> {
    if let Some(cached) = grammar.parse_cache.lock().expect("parse cache").get(text) {
        return Arc::clone(cached);
    }
    let tokens = grammar.tokens(text);
    let question = text.contains(['?', '？']);
    let shared = &*grammar.shared;
    let mut readings = Vec::new();
    if !tokens.is_empty() {
        for template in &grammar.templates {
            let data = &template.data;
            let lenient = data
                .styles
                .iter()
                .all(|style| shared.input_styles.contains(style));
            for (polarity, variants) in [
                ("positive", &template.positive),
                ("negative", &template.negative),
            ] {
                for parts in variants {
                    let mut matcher = Matcher {
                        parts,
                        tokens: &tokens,
                        grammar,
                        lenient,
                        results: Vec::new(),
                    };
                    matcher.walk(0, 0, &Vec::new());
                    for bound in matcher.results {
                        let frame = frame_of(&data.frame, shared);
                        let mood = if data.mood == "statement" && question {
                            "question"
                        } else {
                            data.mood.as_str()
                        };
                        let expanded_frames = if frame.expands.is_empty() {
                            std::slice::from_ref(&frame.id)
                        } else {
                            frame.expands.as_slice()
                        };
                        for expanded in expanded_frames {
                            let target = frame_of(expanded, shared);
                            if !slots_typed(&target, &bound) {
                                continue;
                            }
                            let mut slots = Slots::default();
                            let mut registers = Vec::new();
                            for slot in &target.slots {
                                match bound_of(&bound, slot) {
                                    Some(Bound::Number(number)) => slots.insert(slot, number),
                                    Some(Bound::Concept {
                                        concept,
                                        registers: found,
                                    }) => {
                                        slots.insert(slot, concept);
                                        registers.push((slot.clone(), found.clone()));
                                    }
                                    None => {}
                                }
                            }
                            for style in &data.styles {
                                readings.push(Reading {
                                    graph: Graph {
                                        frame: expanded.clone(),
                                        slots: slots.clone(),
                                        polarity: polarity.to_owned(),
                                        mood: mood.to_owned(),
                                    },
                                    style: style.clone(),
                                    registers: registers.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    let readings = Arc::new(readings);
    let mut cache = grammar.parse_cache.lock().expect("parse cache");
    if cache.len() > PARSE_CACHE_LIMIT {
        cache.clear();
    }
    cache.insert(text.to_owned(), Arc::clone(&readings));
    readings
}

/// Strings in JavaScript's default sort order (UTF-16 code units).
pub(crate) fn js_order(left: &str, right: &str) -> std::cmp::Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

/// Distinct meaning graphs of one utterance, in a stable order.
pub fn utterance_graphs(text: &str, grammar: &CompiledGrammar) -> Vec<Graph> {
    let mut graphs: Vec<(String, Graph)> = Vec::new();
    for reading in parse_utterance(text, grammar).iter() {
        let key = graph_key(&reading.graph);
        match graphs.iter_mut().find(|(seen, _)| *seen == key) {
            Some(entry) => entry.1 = reading.graph.clone(),
            None => graphs.push((key, reading.graph.clone())),
        }
    }
    graphs.sort_by(|left, right| js_order(&left.0, &right.0));
    graphs.into_iter().map(|(_, graph)| graph).collect()
}

fn capitalized(text: &str) -> String {
    match LETTER.find(text) {
        Some(letter) => {
            let upper: String = letter.as_str().to_uppercase();
            format!(
                "{}{upper}{}",
                &text[..letter.start()],
                &text[letter.end()..]
            )
        }
        None => text.to_owned(),
    }
}

/// Declared variants of a frame/style/mood/polarity, preferred first.
fn variants_of<'a>(graph: &Graph, style: &str, grammar: &'a CompiledGrammar) -> Vec<&'a String> {
    grammar
        .data
        .templates
        .iter()
        .filter(|item| {
            item.frame == graph.frame
                && item.mood == graph.mood
                && item.styles.iter().any(|known| known == style)
        })
        .flat_map(|item| match graph.polarity.as_str() {
            "positive" => item.positive.iter(),
            "negative" => item.negative.iter(),
            _ => [].iter(),
        })
        .collect()
}

// Punctuation that already closes an inner clause after a slot.
const CLOSING: &[char] = &[
    '.', ',', ';', ':', '?', '!', '…', '»', ')', '"', '。', '，', '？', '！', '、',
];

/// One declared variant (the preferred one by default) in one register, or `None`.
#[must_use]
pub fn realize_graph(
    graph: &Graph,
    style: &str,
    register: &str,
    grammar: &CompiledGrammar,
    variant: usize,
) -> Option<String> {
    let source = *variants_of(graph, style, grammar).get(variant)?;
    let mut missing = false;
    let mut text = String::new();
    let mut last = 0;
    for found in SLOT.captures_iter(source) {
        let whole = found.get(0).expect("whole match");
        text.push_str(&source[last..whole.start()]);
        last = whole.end();
        let Some(value) = graph.slots.get(&found[1]) else {
            missing = true;
            continue;
        };
        let key = &found[2];
        if key == "number" {
            text.push_str(value);
            continue;
        }
        let forms = grammar
            .term(value)
            .and_then(|term| term.register(register).or_else(|| term.register("forms")));
        let position = grammar.data.keys.iter().position(|item| item == key);
        let Some(form) = forms.zip(position).and_then(|(forms, at)| forms.get(at)) else {
            missing = true;
            continue;
        };
        // A form with an inner clause (for example Russian "..., которая может
        // двигаться") is closed by the language's declared mark unless punctuation
        // or the end of the sentence already follows it. Parsing ignores the mark.
        let rest = source[whole.end()..].trim_start_matches(js_space);
        text.push_str(form);
        if !grammar.data.clause_close.is_empty()
            && form.contains(',')
            && rest
                .chars()
                .next()
                .is_some_and(|next| !CLOSING.contains(&next))
        {
            text.push_str(&grammar.data.clause_close);
        }
    }
    text.push_str(&source[last..]);
    (!missing).then(|| capitalized(&text))
}

/// Every distinct realization of every variant, in the order of the style/register pairs.
#[must_use]
pub fn realizations(
    graph: &Graph,
    pairs: &[(String, String)],
    grammar: &CompiledGrammar,
) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    for (style, register) in pairs {
        for variant in 0..variants_of(graph, style, grammar).len() {
            if let Some(text) = realize_graph(graph, style, register, grammar, variant) {
                if !texts.contains(&text) {
                    texts.push(text);
                }
            }
        }
    }
    texts
}

#[must_use]
pub fn all_pairs(grammar: &CompiledGrammar) -> Vec<(String, String)> {
    grammar
        .output_styles
        .iter()
        .flat_map(|style| {
            grammar
                .shared
                .registers
                .iter()
                .map(move |register| (style.clone(), register.clone()))
        })
        .collect()
}
