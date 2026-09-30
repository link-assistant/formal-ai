//! User-facing formalization handler (issue #1186, E150).
//!
//! "Formalize in first-order logic: Every student who studies passes the
//! exam." used to fall through to the canned web-search paragraph because
//! no `formalize` surface existed in any routing role (the issue's
//! evidence section names the exact gate in `try_translation`). This
//! handler recognizes formalization/deformalization commands in en/ru/hi/
//! zh/es through cue phrases in `data/seed/formal-targets.lino`, parses
//! the sentence into a quantified clause — quantifier, bound variable,
//! antecedent predicate applications, consequent — and renders it into
//! every target grammar the seed declares: first-order logic, Lean 4,
//! Rocq, and Links Notation. Deformalization runs the same grammar in
//! reverse and checks the round trip structurally (re-parsing the
//! rendered sentence and comparing clause structure, never strings).
//!
//! The parse is structural, not memorized: quantifier words, join words,
//! relative markers, articles, and copula drops all come from the seed,
//! and predicate symbols derive from the sentence's own words. No
//! sentence-keyed FOL string exists anywhere (issue requirement R9).
//!
//! Honesty: no theorem prover is invoked and no generated file is
//! compiled in this answer path. `lean`/`coqc` presence is probed in
//! PATH (never executed) and the answer states which check was not run.
//! The in-process relative-meta-logic export is recorded as follow-up
//! until the crate publishes (link-foundation/relative-meta-logic#185).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::{parse_lino, LinoNode};
use super::finalize_simple;

const TARGETS: &str = include_str!("../../embedded/data/seed/formal-targets.lino");
const RESPONSES: &str =
    include_str!("../../embedded/data/seed/multilingual-responses-formalization.lino");
const INTENT: &str = "formalization";

/// How the bound variable is named; the seed's `variable` record owns it.
const DEFAULT_VARIABLE: &str = "x";

// ---------------------------------------------------------------------------
// Seed grammar
// ---------------------------------------------------------------------------

/// One target formal language's rendering grammar.
#[derive(Debug, Clone)]
struct FormalLanguage {
    slug: String,
    aliases: Vec<String>,
    atom: String,
    atom_with_object: String,
    joins: BTreeMap<String, String>,
    quantifiers: BTreeMap<String, String>,
    clause_conditional: String,
    clause_conjunctive: String,
}

/// One natural language's recognition surfaces and templates.
#[derive(Debug, Clone, Default)]
struct NaturalLanguage {
    language: String,
    /// quantifier word → kind (`forall`/`exists`/`no`).
    quantifiers: Vec<(String, String)>,
    /// join word → kind (`and`/`or`).
    joins: Vec<(String, String)>,
    rel_markers: Vec<String>,
    object_introducers: Vec<String>,
    copula_drops: Vec<String>,
    head_final: bool,
    clause_conditional: String,
    clause_conjunctive: String,
}

struct Grammar {
    formal: Vec<FormalLanguage>,
    natural: Vec<NaturalLanguage>,
}

/// The parsed quantified clause: the one semantic object every direction
/// (natural parse, FOL parse, render, round trip) shares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantifiedClause {
    pub quantifier: String,
    pub variable: String,
    /// For universal readings: the antecedent predicates (head noun plus
    /// relative predicates). For existential/negative readings: every
    /// predicate but the last.
    pub antecedent: Vec<AppliedPredicate>,
    pub consequent: AppliedPredicate,
}

/// One predicate applied to the bound variable, with an optional object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedPredicate {
    pub name: String,
    pub object: Option<String>,
}

impl AppliedPredicate {
    fn from_words(words: &[String]) -> Self {
        let mut name = words.first().cloned().unwrap_or_default();
        name = capitalize(&name);
        let object = if words.len() > 1 {
            Some(words[1..].join(" "))
        } else {
            None
        };
        Self { name, object }
    }
}

/// Capitalize the first ASCII letter so predicate symbols read as FOL
/// predicates; non-Latin scripts pass through unchanged.
fn capitalize(word: &str) -> String {
    let mut characters = word.chars();
    match characters.next() {
        Some(first) if first.is_ascii_lowercase() => {
            first.to_ascii_uppercase().to_string() + characters.as_str()
        }
        _ => word.to_owned(),
    }
}

/// Lowercase the first ASCII letter: the deformalization direction of
/// [`capitalize`].
fn decapitalize(word: &str) -> String {
    let mut characters = word.chars();
    match characters.next() {
        Some(first) if first.is_ascii_uppercase() => {
            first.to_ascii_lowercase().to_string() + characters.as_str()
        }
        _ => word.to_owned(),
    }
}

/// The clause's comparison key: quantifier kind, predicate sequence, and
/// object sequence — the structure a round trip must preserve. String
/// surfaces never take part (issue requirement R5).
fn structure_key(clause: &QuantifiedClause) -> (String, Vec<(String, Option<String>)>) {
    let mut predicates: Vec<(String, Option<String>)> = clause
        .antecedent
        .iter()
        .map(|predicate| (predicate.name.clone(), predicate.object.clone()))
        .collect();
    predicates.push((clause.consequent.name.clone(), clause.consequent.object.clone()));
    (clause.quantifier.clone(), predicates)
}

/// All values of `name` children directly under `node`.
fn child_values(node: &LinoNode, name: &str) -> Vec<String> {
    node.children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.clone())
        .filter(|value| !value.is_empty())
        .collect()
}

/// The first direct child named `name`, as a node.
fn named_child<'a>(node: &'a LinoNode, name: &str) -> Option<&'a LinoNode> {
    node.children.iter().find(|child| child.name == name)
}

/// Cue phrases for `intent`/`role` from the targets seed. The phrases sit
/// under the `role` wrapper child of the `intent` node, one nesting level
/// below the record head — the wrapper descent every `.lino` reader in
/// this family performs.
fn cue_phrases(intent: &str, role: &str) -> Vec<String> {
    let tree = parse_lino(TARGETS);
    let mut out = Vec::new();
    for record in tree.children.iter().filter(|child| child.name == "cues") {
        if record.find_child_value("intent") != intent {
            continue;
        }
        let Some(intent_node) = named_child(record, "intent") else {
            continue;
        };
        for role_node in intent_node
            .children
            .iter()
            .filter(|child| child.name == "role" && child.id == role)
        {
            out.extend(child_values(role_node, "phrase"));
        }
    }
    out
}

/// Parse the whole targets seed into the grammar tables.
fn grammar() -> &'static Grammar {
    static GRAMMAR: OnceLock<Grammar> = OnceLock::new();
    GRAMMAR.get_or_init(|| {
        let tree = parse_lino(TARGETS);
        let mut formal = Vec::new();
        let mut natural = Vec::new();
        for record in tree.children.iter().filter(|child| child.name == "formal_language") {
            formal.push(FormalLanguage {
                slug: record.id.clone(),
                aliases: child_values(record, "alias"),
                atom: record.find_child_value("atom").to_owned(),
                atom_with_object: record.find_child_value("atom_with_object").to_owned(),
                joins: record
                    .children
                    .iter()
                    .filter(|child| child.name == "join")
                    .map(|child| (child.id.clone(), child.find_child_value("text").to_owned()))
                    .collect(),
                quantifiers: record
                    .children
                    .iter()
                    .filter(|child| child.name == "quantifier")
                    .map(|child| (child.id.clone(), child.find_child_value("text").to_owned()))
                    .collect(),
                clause_conditional: record.find_child_value("conditional").to_owned(),
                clause_conjunctive: record.find_child_value("conjunctive").to_owned(),
            });
        }
        for record in tree.children.iter().filter(|child| child.name == "natural_language") {
            natural.push(NaturalLanguage {
                language: record.id.clone(),
                quantifiers: record
                    .children
                    .iter()
                    .filter(|child| child.name == "quantifier")
                    .map(|child| (child.find_child_value("text").to_owned(), child.id.clone()))
                    .collect(),
                joins: record
                    .children
                    .iter()
                    .filter(|child| child.name == "join")
                    .map(|child| (child.find_child_value("text").to_owned(), child.id.clone()))
                    .collect(),
                rel_markers: record
                    .children
                    .iter()
                    .filter(|child| child.name == "rel_marker")
                    .map(|child| child.find_child_value("text").to_owned())
                    .filter(|surface| !surface.is_empty())
                    .collect(),
                object_introducers: child_values(record, "object_introducer"),
                copula_drops: child_values(record, "copula_drop"),
                head_final: named_child(record, "head_final").is_some(),
                clause_conditional: record.find_child_value("clause_conditional").to_owned(),
                clause_conjunctive: record.find_child_value("clause_conjunctive").to_owned(),
            });
        }
        Grammar { formal, natural }
    })
}

/// A localized response template from this handler's response seed.
fn response(intent: &str, language: &str) -> String {
    let tree = parse_lino(RESPONSES);
    let exact = tree.children.iter().find_map(|record| {
        (record.name == "response"
            && record.find_child_value("intent") == intent
            && record.find_child_value("language") == language)
        .then(|| record.find_child_value("text").to_owned())
    });
    exact
        .or_else(|| {
            tree.children.iter().find_map(|record| {
                (record.name == "response"
                    && record.find_child_value("intent") == intent
                    && record.find_child_value("language") == "en")
                .then(|| record.find_child_value("text").to_owned())
            })
        })
        .unwrap_or_default()
}

/// Fill a template's `{placeholder}` slots.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = template.to_owned();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

// ---------------------------------------------------------------------------
// Natural-language parsing
// ---------------------------------------------------------------------------

/// Tokenize on whitespace and punctuation, keeping word characters (any
/// script); commas are kept as standalone tokens because comma languages
/// delimit their relative clauses with them.
fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        match character {
            c if c.is_alphanumeric() || c == '-' || c == '\'' || c == '’' => current.push(c),
            ',' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(",".to_owned());
            }
            _ => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Longest-match quantifier lookup: two-word surfaces (`кто не`, `कोई
/// नहीं`) before single words. Returns the kind and the token span length.
fn quantifier_at(
    language: &NaturalLanguage,
    tokens: &[String],
    index: usize,
) -> Option<(String, usize)> {
    if index + 1 < tokens.len() {
        let bigram = format!("{} {}", tokens[index], tokens[index + 1]);
        if let Some((_, kind)) = language
            .quantifiers
            .iter()
            .find(|(word, _)| *word == bigram)
        {
            return Some((kind.clone(), 2));
        }
    }
    language
        .quantifiers
        .iter()
        .find(|(word, _)| *word == tokens[index])
        .map(|(_, kind)| (kind.clone(), 1))
}

/// Parse a natural-language sentence into a quantified clause using the
/// recognition surfaces of `language`.
pub fn parse_quantified_clause(
    text: &str,
    language: &NaturalLanguage,
) -> Option<QuantifiedClause> {
    let tokens: Vec<String> = tokenize(text)
        .into_iter()
        .filter(|token| !language.copula_drops.contains(token))
        .collect();
    let join_words: Vec<String> = language
        .joins
        .iter()
        .map(|(word, _)| word.clone())
        .collect();
    let variable = DEFAULT_VARIABLE.to_owned();

    if language.head_final {
        // CJK text carries no spaces; spacing the seed's known surfaces
        // turns each into one whitespace-delimited token while the runs
        // between them stay whole words (`每个 学习 的 学生 都 通过考试`).
        let mut spaced = text.to_owned();
        let mut surfaces: Vec<String> = language
            .quantifiers
            .iter()
            .map(|(word, _)| word.clone())
            .collect();
        surfaces.extend(language.rel_markers.iter().cloned());
        surfaces.extend(join_words.iter().cloned());
        surfaces.push("都".to_owned());
        for surface in surfaces {
            spaced = spaced.replace(&surface, &format!(" {surface} "));
        }
        let tokens: Vec<String> = tokenize(&spaced)
            .into_iter()
            .filter(|token| !language.copula_drops.contains(token))
            .collect();
        // Shape: {quantifier} {relatives} 的 {head} [都] {main phrase}; a
        // cue prefix before the quantifier is skipped by scanning.
        let mut quantifier_index = None;
        for index in 0..tokens.len() {
            if quantifier_at(language, &tokens, index).is_some() {
                quantifier_index = Some(index);
                break;
            }
        }
        let quantifier_index = quantifier_index?;
        let (kind, span) = quantifier_at(language, &tokens, quantifier_index)?;
        let marker = tokens[quantifier_index + span..]
            .iter()
            .position(|token| language.rel_markers.contains(token))
            .map(|offset| quantifier_index + span + offset)?;
        let relatives: Vec<String> = tokens[quantifier_index + span..marker]
            .iter()
            .filter(|token| !join_words.contains(token))
            .cloned()
            .collect();
        let after_marker: Vec<String> = tokens[marker + 1..]
            .iter()
            .filter(|token| *token != "都")
            .cloned()
            .collect();
        if after_marker.is_empty() {
            return None;
        }
        let head = after_marker[0].clone();
        let main_words: Vec<String> = after_marker[1..].to_vec();
        let mut antecedent = vec![AppliedPredicate::from_words(&[head])];
        antecedent.extend(
            relatives
                .iter()
                .map(|word| AppliedPredicate::from_words(&[word.clone()])),
        );
        return Some(QuantifiedClause {
            quantifier: kind,
            variable,
            antecedent,
            consequent: AppliedPredicate::from_words(&main_words),
        });
    }

    // Shape: {quantifier} {head} {rel_marker} {relatives}[,] {main phrase}.
    let mut quantifier_index = None;
    for index in 0..tokens.len() {
        if quantifier_at(language, &tokens, index).is_some() {
            quantifier_index = Some(index);
            break;
        }
    }
    let quantifier_index = quantifier_index?;
    let (kind, span) = quantifier_at(language, &tokens, quantifier_index)?;
    let head_index = quantifier_index + span;
    if head_index >= tokens.len() {
        return None;
    }
    let head = tokens[head_index].clone();
    let after_head = &tokens[head_index + 1..];
    let marker_offset = after_head
        .iter()
        .position(|token| language.rel_markers.contains(token))?;

    let span_tokens: Vec<String> = after_head[marker_offset + 1..]
        .iter()
        .filter(|token| *token != ",")
        .cloned()
        .collect();
    // For comma languages the main phrase is what follows the last comma;
    // for article languages an object introducer starts the object span,
    // and otherwise the final token is the main predicate.
    let (relative_words, main_words): (Vec<String>, Vec<String>) =
        if !language.object_introducers.is_empty() {
            if let Some(introducer) = span_tokens
                .iter()
                .position(|token| language.object_introducers.contains(token))
            {
                if introducer == 0 {
                    // An article directly after the marker: no verb to
                    // read; the shape this parser covers does not match.
                    return None;
                }
                let before = &span_tokens[..introducer];
                let object_words = &span_tokens[introducer + 1..];
                let mut main = vec![before
                    .last()
                    .cloned()
                    .unwrap_or_default()];
                main.extend(object_words.iter().cloned());
                (
                    before[..before.len() - 1].to_vec(),
                    main,
                )
            } else {
                let main = vec![span_tokens.last()?.clone()];
                (span_tokens[..span_tokens.len() - 1].to_vec(), main)
            }
        } else {
            let tail = &after_head[marker_offset + 1..];
            let comma_positions: Vec<usize> = tail
                .iter()
                .enumerate()
                .filter_map(|(index, token)| (token == ",").then_some(index))
                .collect();
            if let Some(&last_comma) = comma_positions.last() {
                let relative_words: Vec<String> = tail[..last_comma]
                    .iter()
                    .filter(|token| *token != ",")
                    .cloned()
                    .collect();
                let main_words: Vec<String> = tail[last_comma + 1..]
                    .iter()
                    .filter(|token| *token != ",")
                    .cloned()
                    .collect();
                if main_words.is_empty() {
                    return None;
                }
                (relative_words, main_words)
            } else {
                let main = vec![span_tokens.last()?.clone()];
                (span_tokens[..span_tokens.len() - 1].to_vec(), main)
            }
        };
    let relatives: Vec<AppliedPredicate> = relative_words
        .iter()
        .filter(|word| !join_words.contains(word))
        .map(|word| AppliedPredicate::from_words(&[word.clone()]))
        .collect();
    let mut antecedent = vec![AppliedPredicate::from_words(&[head])];
    antecedent.extend(relatives);
    Some(QuantifiedClause {
        quantifier: kind,
        variable,
        antecedent,
        consequent: AppliedPredicate::from_words(&main_words),
    })
}

// ---------------------------------------------------------------------------
// Formal rendering
// ---------------------------------------------------------------------------

/// Render one atom (`Name(x)` / `Name(x, object)`) in a target language.
fn render_atom(
    language: &FormalLanguage,
    predicate: &AppliedPredicate,
    variable: &str,
) -> String {
    let template = if predicate.object.is_some() {
        &language.atom_with_object
    } else {
        &language.atom
    };
    fill(
        template,
        &[
            ("predicate", &predicate.name),
            ("variable", variable),
            ("object", predicate.object.as_deref().unwrap_or_default()),
        ],
    )
}

/// Render the whole clause in a target formal language.
pub fn render_clause(clause: &QuantifiedClause, slug: &str) -> Option<String> {
    let language = grammar().formal.iter().find(|item| item.slug == slug)?;
    let and = language.joins.get("and").cloned().unwrap_or_default();
    let variable = &clause.variable;
    let antecedent = clause
        .antecedent
        .iter()
        .map(|predicate| render_atom(language, predicate, variable))
        .collect::<Vec<_>>()
        .join(&and);
    let consequent = render_atom(language, &clause.consequent, variable);
    let conditional = clause.quantifier == "forall";
    let template = if conditional {
        &language.clause_conditional
    } else {
        &language.clause_conjunctive
    };
    let quantifier_symbol = language
        .quantifiers
        .get(&clause.quantifier)
        .cloned()
        .unwrap_or_default();
    let joined = [antecedent.clone(), consequent.clone()].join(&and);
    let rendered = fill(
        template,
        &[
            ("quantifier", &quantifier_symbol),
            ("variable", variable),
            ("antecedent", &antecedent),
            ("consequent", &consequent),
            ("joined", &joined),
            ("quantifier_word", &clause.quantifier),
        ],
    );
    Some(rendered.trim_end().to_owned())
}

// ---------------------------------------------------------------------------
// Formal parsing (deformalization input side)
// ---------------------------------------------------------------------------

/// Split `text` on `separator` at paren depth zero.
fn split_top_level(text: &str, separator: char) -> Vec<String> {
    let mut depth = 0usize;
    let mut parts = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        match character {
            '(' => {
                depth += 1;
                current.push(character);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            c if c == separator && depth == 0 => {
                parts.push(current.trim().to_owned());
                current.clear();
            }
            c => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_owned());
    }
    parts
}

/// Parse one FOL atom: `Name(x)` or `Name(x, object)`.
fn parse_atom(text: &str, variable: &str) -> Option<AppliedPredicate> {
    let text = text.trim();
    let open = text.find('(')?;
    let close = text.rfind(')')?;
    let name = capitalize(text[..open].trim());
    let args = text[open + 1..close].split(',').map(str::trim).collect::<Vec<_>>();
    let object = args
        .iter()
        .find(|argument| *argument != variable)
        .map(|argument| argument.trim_matches('"').to_owned());
    Some(AppliedPredicate { name, object })
}

/// Parse FOL text (any quantifier surface the seed declares) back into a
/// clause. `∀x (A(x) ∧ B(x) → C(x))` and `¬∃x (A(x) ∧ B(x) ∧ C(x))` are
/// both accepted; `forall`/`exists`/`~ exists` (Rocq surfaces) too.
pub fn parse_fol_clause(text: &str) -> Option<QuantifiedClause> {
    let text = text.trim();
    let fol = grammar().formal.iter().find(|item| item.slug == "fol")?;
    let mut quantifier = String::new();
    let mut rest = text;
    // Rocq-style ASCII word surfaces first (they are prefixes of nothing
    // the symbol table carries), then the FOL symbol table.
    for (kind, surface) in [("no", "~ exists"), ("forall", "forall"), ("exists", "exists")] {
        if text.starts_with(surface) {
            quantifier = kind.to_owned();
            rest = &text[surface.len()..];
            break;
        }
    }
    if quantifier.is_empty() {
        for (kind, symbol) in &fol.quantifiers {
            if text.starts_with(symbol.as_str()) {
                quantifier = kind.clone();
                rest = &text[symbol.len()..];
                break;
            }
        }
    }
    if quantifier.is_empty() {
        return None;
    }
    let rest = rest.trim();
    let variable: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if variable.is_empty() {
        return None;
    }
    let open = rest.find('(')?;
    let close = rest.rfind(')')?;
    // Rocq's ASCII arrow normalizes to the FOL arrow before splitting.
    let body = rest[open + 1..close].trim().replace("->", "→");
    let conditional_parts = split_top_level(&body, '→');
    let atoms: Vec<AppliedPredicate> = if conditional_parts.len() >= 2 {
        let mut all = split_top_level(&conditional_parts[0], '∧');
        all.extend(split_top_level(&conditional_parts[1], '∧'));
        all
    } else {
        split_top_level(&body, '∧')
    };
    let predicates: Vec<AppliedPredicate> = atoms
        .iter()
        .filter_map(|atom| parse_atom(atom, &variable))
        .collect();
    if predicates.len() < 2 {
        return None;
    }
    let (antecedent, consequent) = if conditional_parts.len() >= 2 {
        let antecedent_atoms: Vec<AppliedPredicate> = split_top_level(&conditional_parts[0], '∧')
            .iter()
            .filter_map(|atom| parse_atom(atom, &variable))
            .collect();
        let consequent = parse_atom(&conditional_parts[1], &variable)?;
        (antecedent_atoms, consequent)
    } else {
        let consequent = predicates[predicates.len() - 1].clone();
        (predicates[..predicates.len() - 1].to_vec(), consequent)
    };
    Some(QuantifiedClause {
        quantifier,
        variable,
        antecedent,
        consequent,
    })
}

// ---------------------------------------------------------------------------
// Natural rendering (deformalization output side)
// ---------------------------------------------------------------------------

/// Render the clause as a natural sentence in `language`.
pub fn render_clause_natural(clause: &QuantifiedClause, language: &str) -> Option<String> {
    let natural = grammar()
        .natural
        .iter()
        .find(|item| item.language == language)?;
    let quantifier_word = natural
        .quantifiers
        .iter()
        .find(|(_, kind)| *kind == clause.quantifier)
        .map(|(word, _)| word.clone())?;
    let join_word = natural
        .joins
        .iter()
        .find(|(_, kind)| *kind == "and")
        .map(|(word, _)| word.clone())
        .unwrap_or_default();
    let rel_marker = natural.rel_markers.first().cloned().unwrap_or_default();
    let conditional = clause.quantifier == "forall";
    let template = if conditional {
        &natural.clause_conditional
    } else {
        &natural.clause_conjunctive
    };
    let words_of = |predicate: &AppliedPredicate| -> String {
        let head = decapitalize(&predicate.name);
        match (&predicate.object, natural.object_introducers.first()) {
            (Some(object), Some(introducer)) => format!("{head} {introducer} {object}"),
            (Some(object), None) => format!("{head} {object}"),
            (None, _) => head,
        }
    };
    let head = decapitalize(&clause.antecedent.first()?.name);
    let main = words_of(&clause.consequent);
    let relatives = clause.antecedent[1..]
        .iter()
        .map(|predicate| decapitalize(&predicate.name))
        .collect::<Vec<_>>()
        .join(&format!(" {join_word} "));
    Some(fill(
        template,
        &[
            ("quantifier", &quantifier_word),
            ("head", &head),
            ("rel_marker", &rel_marker),
            ("relatives", &relatives),
            ("main", &main),
        ],
    ))
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// True when a formal quantifier symbol from any target grammar occurs in
/// the prompt (the deformalization trigger for text already in FOL). Only
/// the unambiguous symbols count: the ASCII words `forall`/`exists` are
/// ordinary English words, so they never trigger on their own.
fn carries_formal_surface(prompt: &str) -> bool {
    ["∀", "∃", "¬∃"].iter().any(|symbol| prompt.contains(symbol))
}

/// The `lean`/`coqc` presence sentence for the honesty block. The binary
/// is looked up in PATH and never executed.
fn prover_check(binary: &str, label: &str) -> String {
    let present = std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|directory| directory.join(binary).is_file())
        })
        .unwrap_or(false);
    if present {
        format!("{label} is present in PATH; compilation was not run in this answer path and is delegated to CI.")
    } else {
        format!("{label} was not found in PATH, so the {label} text was not compiled.")
    }
}

/// The language of a cue role suffix (`command_ru` → `ru`).
fn role_language(role: &str) -> &str {
    role.rsplit('_').next().unwrap_or("en")
}

/// Which cue role families matched the prompt, as (family, language)
/// pairs — `command` for formalize, `direction` for deformalize.
fn matched_roles(prompt: &str, normalized: &str) -> Vec<(String, String)> {
    let lower = prompt.to_lowercase();
    let mut out = Vec::new();
    for record in parse_lino(TARGETS).children.iter().filter(|child| child.name == "cues") {
        let Some(intent_node) = named_child(record, "intent") else {
            continue;
        };
        for role_node in intent_node.children.iter().filter(|child| child.name == "role") {
            let role = role_node.id.clone();
            let hit = child_values(role_node, "phrase").iter().any(|phrase| {
                normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str())
            });
            if hit {
                let family = role
                    .split('_')
                    .next()
                    .unwrap_or("command")
                    .to_owned();
                out.push((family, role_language(&role).to_owned()));
            }
        }
    }
    out
}

/// The formal target the prompt names, if any (an alias surface).
fn named_target(prompt: &str, normalized: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    grammar()
        .formal
        .iter()
        .find(|language| {
            language.aliases.iter().any(|alias| {
                normalized.contains(alias.as_str())
                    || lower.contains(&alias.to_lowercase())
            })
        })
        .map(|language| language.slug.clone())
}

/// The sentence to formalize: the text after a colon separator when
/// present, else the prompt with the cue phrase removed.
fn sentence_under_discussion(prompt: &str) -> String {
    if let Some((_, after)) = prompt.split_once(':') {
        let trimmed = after.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches(['.', '。', '।']).to_owned();
        }
    }
    prompt.trim().to_owned()
}

/// Recognize and answer a formalization or deformalization request.
/// Returns `None` when the prompt is neither, so the dispatch chain
/// continues (issue requirement R1: the canned search paragraph must
/// never answer a formalization prompt again).
pub fn handle_formalization_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let roles = matched_roles(prompt, normalized);
    let formalize = roles.iter().any(|(family, _)| family == "command");
    let deformalize = roles.iter().any(|(family, _)| family == "direction")
        || (!formalize && carries_formal_surface(prompt));
    if !formalize && !deformalize {
        return None;
    }
    let request_language = roles
        .first()
        .map(|(_, language)| language.clone())
        .unwrap_or_else(|| "en".to_owned());
    log.append(
        "formalization:direction",
        if formalize { "formalize" } else { "deformalize" }.to_owned(),
    );
    log.append("formalization:language", request_language.clone());

    let body = if formalize {
        formalize_answer(prompt, &request_language, log)
    } else {
        deformalize_answer(prompt, &request_language, log)
    };
    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:formalization",
        &body.0,
        body.1,
    ))
}

/// Compose the formalize-direction answer body.
fn formalize_answer(
    prompt: &str,
    language: &str,
    log: &mut EventLog,
) -> (String, f32) {
    let sentence = sentence_under_discussion(prompt);
    let natural = grammar()
        .natural
        .iter()
        .find(|item| item.language == language)
        .cloned()
        .unwrap_or_else(|| {
            grammar()
                .natural
                .iter()
                .find(|item| item.language == "en")
                .cloned()
                .unwrap_or_default()
        });
    match parse_quantified_clause(&sentence, &natural) {
        Some(clause) => {
            log.append(
                "formalization:clause",
                format!(
                    "{} {} antecedents {} consequent {}",
                    clause.quantifier,
                    clause.variable,
                    clause.antecedent.len(),
                    clause.consequent.name
                ),
            );
            let slugs: Vec<String> = {
                let mut all: Vec<String> = grammar()
                    .formal
                    .iter()
                    .map(|item| item.slug.clone())
                    .collect();
                // A named target renders first; the others follow, because
                // every declared target is a legitimate rendering of the
                // same parsed clause (issue requirement R4).
                if let Some(one) = named_target(prompt, &sentence.to_lowercase()) {
                    all.sort_by_key(|slug| slug != &one);
                }
                all
            };
            let mut blocks = Vec::new();
            for slug in &slugs {
                if let Some(rendered) = render_clause(&clause, slug) {
                    blocks.push(format!("```{slug}\n{rendered}\n```"));
                }
            }
            let statement_block = blocks.join("\n\n");
            let derivation = [
                format!("sentence: {sentence}"),
                format!("parsed clause: {} {} ({})", clause.quantifier, clause.variable, {
                    let names: Vec<String> = clause.antecedent.iter().map(|p| p.name.clone()).collect();
                    names.join(", ")
                }),
                format!("consequent: {}", clause.consequent.name),
                format!("templates: {}", slugs.join(", ")),
            ]
            .join("\n");
            let honesty = fill(
                &response("formalization_honesty", language),
                &[
                    ("lean_check", &prover_check("lean", "lean")),
                    ("rocq_check", &prover_check("coqc", "coqc")),
                ],
            );
            (
                fill(
                    &response("formalization_result", language),
                    &[
                        ("statement_block", &statement_block),
                        ("honesty", &honesty),
                        ("derivation", &derivation),
                    ],
                ),
                0.7,
            )
        }
        None => {
            log.append(
                "formalization:clause",
                "no quantified clause recognized".to_owned(),
            );
            (
                fill(
                    &response("formalization_unparsed", language),
                    &[("reason", "no quantifier word or symbol matched the seed grammar")],
                ),
                0.4,
            )
        }
    }
}

/// The formal span of a prompt: from the first quantifier symbol to the
/// end, so leading cue words ("Deformalize ∀x (…)") do not block parsing.
fn formal_span(prompt: &str) -> &str {
    let start = ["∀", "∃", "¬∃"]
        .iter()
        .filter_map(|symbol| prompt.find(symbol))
        .min()
        .unwrap_or(0);
    prompt[start..].trim()
}

/// Compose the deformalize-direction answer body, including the
/// structural round-trip check (requirement R5).
fn deformalize_answer(
    prompt: &str,
    language: &str,
    log: &mut EventLog,
) -> (String, f32) {
    let Some(clause) = parse_fol_clause(formal_span(prompt)) else {
        log.append("formalization:clause", "no formal clause recognized".to_owned());
        return (
            fill(
                &response("formalization_unparsed", language),
                &[("reason", "no formal quantifier surface matched the seed grammar")],
            ),
            0.4,
        );
    };
    log.append(
        "formalization:clause",
        format!("{} {} from formal text", clause.quantifier, clause.variable),
    );
    let natural = grammar()
        .natural
        .iter()
        .find(|item| item.language == language)
        .cloned()
        .unwrap_or_default();
    let sentence = render_clause_natural(&clause, language).unwrap_or_default();
    let round_trip = match parse_quantified_clause(&sentence, &natural) {
        Some(reparsed) if structure_key(&reparsed) == structure_key(&clause) => {
            "structure preserved (re-parse matches)"
        }
        Some(_) => "structure drifted (re-parse differs; stated honestly)",
        None => "re-parse failed (stated honestly)",
    };
    log.append("formalization:round_trip", round_trip.to_owned());
    let honesty = fill(
        &response("formalization_honesty", language),
        &[
            ("lean_check", &prover_check("lean", "lean")),
            ("rocq_check", &prover_check("coqc", "coqc")),
        ],
    );
    (
        fill(
            &response("formalization_deformalized", language),
            &[
                ("sentence", &sentence),
                ("round_trip", round_trip),
                ("honesty", &honesty),
            ],
        ),
        0.7,
    )
}
