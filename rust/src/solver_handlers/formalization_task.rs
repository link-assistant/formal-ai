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

/// The records of the targets seed. `formal-targets.lino` wraps every
/// record (`cues`, `variable`, `formal_language`, `natural_language`)
/// under one `formal_targets` root, so the records are the children of the
/// top-level nodes — mirroring `formalTargetRecords` in
/// `js/worker/formal_ai_worker_formalization_request.js`.
fn target_records(tree: &LinoNode) -> impl Iterator<Item = &LinoNode> {
    tree.children.iter().flat_map(|top| top.children.iter())
}

/// Parse the whole targets seed into the grammar tables.
fn grammar() -> &'static Grammar {
    static GRAMMAR: OnceLock<Grammar> = OnceLock::new();
    GRAMMAR.get_or_init(|| {
        let tree = parse_lino(TARGETS);
        let mut formal = Vec::new();
        let mut natural = Vec::new();
        for record in target_records(&tree).filter(|child| child.name == "formal_language") {
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
                clause_conditional: record.find_child_value("clause_conditional").to_owned(),
                clause_conjunctive: record.find_child_value("clause_conjunctive").to_owned(),
            });
        }
        for record in target_records(&tree).filter(|child| child.name == "natural_language") {
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

/// A localized response template from this handler's response seed
/// (`multilingual-responses-formalization.lino`, registered with the
/// shared response tables): the language, else English, else empty.
fn response(intent: &str, language: &str) -> String {
    crate::seed::response_for(intent, language)
        .or_else(|| crate::seed::response_for(intent, "en"))
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

/// A word character: letters, numbers and combining marks. A Devanagari
/// virama or nukta is a mark, not alphanumeric, so without the mark clause
/// "पढ़ता" would split inside its conjunct.
fn is_word_character(character: char) -> bool {
    use unicode_general_category::GeneralCategory;
    character.is_alphanumeric()
        || matches!(
            unicode_general_category::get_general_category(character),
            GeneralCategory::NonspacingMark
                | GeneralCategory::SpacingMark
                | GeneralCategory::EnclosingMark
        )
}

/// Tokenize the lowercased text on whitespace and punctuation, keeping
/// word characters (any script, marks included); commas are kept as
/// standalone tokens because comma languages delimit their relative
/// clauses with them. Lowercasing lets the seed's lowercase quantifier and
/// marker surfaces match a sentence-initial capital ("Every").
fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in text.to_lowercase().chars() {
        match character {
            c if is_word_character(c) || c == '-' || c == '\'' || c == '’' => current.push(c),
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
fn parse_quantified_clause(
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
                .map(|word| AppliedPredicate::from_words(std::slice::from_ref(word))),
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
        if language.object_introducers.is_empty() {
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
        } else if let Some(introducer) = span_tokens
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
        };
    let relatives: Vec<AppliedPredicate> = relative_words
        .iter()
        .filter(|word| !join_words.contains(word))
        .map(|word| AppliedPredicate::from_words(std::slice::from_ref(word)))
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

include!("formalization_task_render.rs");
