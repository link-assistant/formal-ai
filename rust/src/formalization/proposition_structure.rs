//! The proposition structure a statement keeps beyond its set of terms
//! (issue #1487): the order of the operands of a comparison or an operator,
//! the connectives (and/or), the quantifiers, the place of a negation among
//! them, and an explicit count of repetition.
//!
//! A statement identity made of a sorted set of terms alone reads `2 < 1` as
//! `1 < 2` and `true or false` as `true and false`; this structure keeps them
//! apart. Structure the reader does not support is reported as unresolved,
//! never guessed.
//!
//! Every connective, quantifier, comparison and operator is data: the
//! structural rules `proposition-structure`, `comparison-relation` and
//! `symbolic-operator` of `data/seed/text-formalization.lino` name the grammar
//! vocabulary kinds and lexicon meanings read here. The browser twin is
//! `js/agentic/crate/proposition_structure.mjs`.

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

use super::text_statements::{Word, WordKind};
use super::{grammar_vocabulary, structural_rule_values};

/// The structural rule naming connective, quantifier and repetition words.
pub const PROPOSITION_RULE: &str = "proposition-structure";
/// The structural rule naming the lexicon meanings of comparison words.
pub const COMPARISON_RULE: &str = "comparison-relation";
/// The structural rule naming operator symbols.
pub const OPERATOR_RULE: &str = "symbolic-operator";
/// Relations with a converse wording (`greater` is converse `less`).
pub const ORDERED_RELATIONS: [&str; 2] = ["less", "less-or-equal"];
/// Operators without a converse wording.
pub const PLAIN_OPERATORS: [&str; 6] = ["equal", "not-equal", "minus", "plus", "times", "divide"];
/// The suffix of the converse kind of an ordered relation.
pub const CONVERSE_SUFFIX: &str = "-converse";

/// One token of the connective and quantifier shape of a clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeToken {
    /// A run of content words.
    Content,
    /// A negation cue.
    Negation,
    /// An `and` connective.
    Conjunction,
    /// An `or` connective.
    Disjunction,
    /// A universal quantifier.
    Universal,
    /// An existential quantifier.
    Existential,
}

impl ShapeToken {
    /// The token as the shape writes it.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Content => "_",
            Self::Negation => "not",
            Self::Conjunction => "conjunction",
            Self::Disjunction => "disjunction",
            Self::Universal => "universal",
            Self::Existential => "existential",
        }
    }

    const fn connective(self) -> bool {
        matches!(self, Self::Conjunction | Self::Disjunction)
    }
}

/// Structure the reader does not support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unresolved {
    /// A comparison word without an operand on each side.
    RelationOperand,
    /// Two different operators chained without grouping (`x + y * z`).
    OperatorChain,
}

impl Unresolved {
    /// The slug the structure reports.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::RelationOperand => "relation-operand",
            Self::OperatorChain => "operator-chain",
        }
    }
}

/// What a phrase of the structure vocabulary marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseRole {
    /// A connective.
    Connective(ShapeToken),
    /// An explicit count of repetition.
    Repetition,
}

/// One connective or repetition phrase of a language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructurePhrase {
    /// The phrase, lowercased.
    pub phrase: String,
    /// What it marks.
    pub role: PhraseRole,
    /// The grammar vocabulary kind it comes from.
    pub kind: String,
    /// Its length in characters.
    pub length: usize,
}

/// An operator or comparison relation, with whether its wording is converse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationKind {
    /// The relation (`less`, `minus`, …).
    pub op: &'static str,
    /// Whether the operands are written in the converse order.
    pub converse: bool,
}

/// One seeded operator symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorSymbol {
    /// The symbol as written.
    pub symbol: String,
    /// The relation it writes.
    pub relation: RelationKind,
    /// Its characters.
    pub chars: Vec<char>,
}

/// One relation: its text `op(left,right)` and its operands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    /// `op(left,right)`.
    pub text: String,
    /// The operand ids in relation order.
    pub operands: [String; 2],
}

/// The structure a statement keeps beyond its set of terms.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PropositionStructure {
    /// The connective and quantifier shape, tokens joined by spaces.
    pub shape: String,
    /// Whether the shape tells statements with one set of terms apart.
    pub significant: bool,
    /// The ordered relations, sorted.
    pub relations: Vec<String>,
    /// The operands of the relations, sorted.
    pub operands: Vec<String>,
    /// The repetition kinds, sorted.
    pub repetition: Vec<String>,
    /// The unsupported structure, sorted.
    pub unresolved: Vec<String>,
}

/// The connective and repetition phrases of `language`, longest first.
#[must_use]
pub fn structure_phrases(language: &str) -> Arc<Vec<StructurePhrase>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Vec<StructurePhrase>>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Arc::clone(
        guard
            .entry(language.to_owned())
            .or_insert_with(|| Arc::new(build_structure_phrases(language))),
    )
}

fn build_structure_phrases(language: &str) -> Vec<StructurePhrase> {
    let mut out = Vec::new();
    let mut add = |role: PhraseRole, kind: &str| {
        for phrase in grammar_vocabulary(language, kind) {
            let phrase = phrase.to_lowercase();
            let length = phrase.chars().count();
            out.push(StructurePhrase {
                phrase,
                role,
                kind: kind.to_owned(),
                length,
            });
        }
    };
    for (token, vocabulary) in [
        (ShapeToken::Conjunction, "conjunction-vocabulary"),
        (ShapeToken::Disjunction, "disjunction-vocabulary"),
    ] {
        for kind in structural_rule_values(PROPOSITION_RULE, vocabulary) {
            add(PhraseRole::Connective(token), &kind);
        }
    }
    for kind in structural_rule_values(PROPOSITION_RULE, "repetition-vocabulary") {
        add(PhraseRole::Repetition, &kind);
    }
    // A stable sort keeps declaration order among phrases of one length.
    out.sort_by(|left, right| right.length.cmp(&left.length));
    out
}

/// The quantifier a meaning id reads as.
#[must_use]
pub fn quantifier_of(id: &str) -> Option<ShapeToken> {
    if structural_rule_values(PROPOSITION_RULE, "universal-meaning")
        .iter()
        .any(|value| value == id)
    {
        return Some(ShapeToken::Universal);
    }
    if structural_rule_values(PROPOSITION_RULE, "existential-meaning")
        .iter()
        .any(|value| value == id)
    {
        return Some(ShapeToken::Existential);
    }
    None
}

/// The relation a value of `rule` names: a comparison meaning for
/// [`COMPARISON_RULE`], an operator symbol for [`OPERATOR_RULE`].
#[must_use]
pub fn relation_of(rule: &str, value: &str) -> Option<RelationKind> {
    let kinds: Vec<&'static str> = if rule == COMPARISON_RULE {
        ORDERED_RELATIONS.to_vec()
    } else {
        ORDERED_RELATIONS
            .iter()
            .chain(PLAIN_OPERATORS.iter())
            .copied()
            .collect()
    };
    for op in kinds {
        if structural_rule_values(rule, op)
            .iter()
            .any(|candidate| candidate == value)
        {
            return Some(RelationKind {
                op,
                converse: false,
            });
        }
        if ORDERED_RELATIONS.contains(&op)
            && structural_rule_values(rule, &format!("{op}{CONVERSE_SUFFIX}"))
                .iter()
                .any(|candidate| candidate == value)
        {
            return Some(RelationKind { op, converse: true });
        }
    }
    None
}

/// Every seeded operator symbol, longest first.
#[must_use]
pub fn operator_symbols() -> &'static [OperatorSymbol] {
    static SYMBOLS: OnceLock<Vec<OperatorSymbol>> = OnceLock::new();
    SYMBOLS.get_or_init(|| {
        let mut out = Vec::new();
        let mut add = |symbol: String, op: &'static str, converse: bool| {
            let chars = symbol.chars().collect();
            out.push(OperatorSymbol {
                symbol,
                relation: RelationKind { op, converse },
                chars,
            });
        };
        for &op in ORDERED_RELATIONS.iter().chain(PLAIN_OPERATORS.iter()) {
            for symbol in structural_rule_values(OPERATOR_RULE, op) {
                add(symbol, op, false);
            }
            if ORDERED_RELATIONS.contains(&op) {
                for symbol in
                    structural_rule_values(OPERATOR_RULE, &format!("{op}{CONVERSE_SUFFIX}"))
                {
                    add(symbol, op, true);
                }
            }
        }
        out.sort_by(|left, right| right.chars.len().cmp(&left.chars.len()));
        out
    })
}

/// The relation `op(left,right)`, ordered unless the op commutes.
#[must_use]
pub fn relation(op: &str, converse: bool, left: String, right: String) -> Relation {
    let (first, second) = if converse {
        (right, left)
    } else {
        (left, right)
    };
    let commutes = structural_rule_values(OPERATOR_RULE, "commutative")
        .iter()
        .any(|value| value == op);
    let operands = if commutes && second < first {
        [second, first]
    } else {
        [first, second]
    };
    Relation {
        text: format!("{op}({},{})", operands[0], operands[1]),
        operands,
    }
}

/// The id of the term written as `surface`, else a symbol id.
#[must_use]
pub fn operand_id<'a>(surface: &str, terms: impl IntoIterator<Item = &'a Word>) -> String {
    let key = surface.to_lowercase();
    terms
        .into_iter()
        .find(|term| term.surface.to_lowercase() == key)
        .map_or_else(|| format!("symbol:{key}"), |term| term.id.clone())
}

/// The longest structure phrase opening at `start`, with its width in words.
#[must_use]
pub fn phrase_at<'a>(
    surfaces: &[String],
    start: usize,
    phrases: &'a [StructurePhrase],
) -> Option<(&'a StructurePhrase, usize)> {
    for entry in phrases {
        let mut spaced = String::new();
        let mut solid = String::new();
        for (end, surface) in surfaces.iter().enumerate().skip(start) {
            if end > start {
                spaced.push(' ');
            }
            spaced.push_str(surface);
            solid.push_str(surface);
            if spaced == entry.phrase || solid == entry.phrase {
                return Some((entry, end - start + 1));
            }
            if solid.chars().count() >= entry.length {
                break;
            }
        }
    }
    None
}

/// What one item of a symbolic reading is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolicKind {
    /// A run of letters and digits.
    Operand,
    /// A seeded operator symbol.
    Operator(RelationKind),
    /// Any other character.
    Break,
}

/// One operand run, operator symbol or break of a text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicItem {
    /// What the item is.
    pub kind: SymbolicKind,
    /// Its text.
    pub value: String,
    /// Whether whitespace precedes it.
    pub spaced: bool,
}

/// The operand runs, operator symbols and breaks of `text`.
#[must_use]
pub fn symbolic_items(text: &str) -> Vec<SymbolicItem> {
    let chars: Vec<char> = text.chars().collect();
    let symbols = operator_symbols();
    let mut items = Vec::new();
    let mut spaced = false;
    let mut position = 0;
    while position < chars.len() {
        let character = chars[position];
        if character.is_whitespace() {
            spaced = true;
            position += 1;
            continue;
        }
        if character.is_alphanumeric() {
            let mut end = position;
            while end < chars.len()
                && (chars[end].is_alphanumeric()
                    || ((chars[end] == '.' || chars[end] == ',')
                        && chars[end - 1].is_numeric()
                        && end + 1 < chars.len()
                        && chars[end + 1].is_numeric()))
            {
                end += 1;
            }
            items.push(SymbolicItem {
                kind: SymbolicKind::Operand,
                value: chars[position..end].iter().collect(),
                spaced,
            });
            position = end;
        } else if let Some(symbol) = symbols.iter().find(|entry| {
            entry
                .chars
                .iter()
                .enumerate()
                .all(|(offset, part)| chars.get(position + offset) == Some(part))
        }) {
            items.push(SymbolicItem {
                kind: SymbolicKind::Operator(symbol.relation),
                value: symbol.symbol.clone(),
                spaced,
            });
            position += symbol.chars.len();
        } else {
            items.push(SymbolicItem {
                kind: SymbolicKind::Break,
                value: character.to_string(),
                spaced,
            });
            position += 1;
        }
        spaced = false;
    }
    items
}

/// Whether an operator joins a compound word (`e-mail`, `and/or`) rather
/// than two operands.
#[must_use]
pub fn joins_compound(left: &SymbolicItem, operator: &SymbolicItem, right: &SymbolicItem) -> bool {
    if operator.spaced
        || right.spaced
        || !structural_rule_values(OPERATOR_RULE, "compound-joiner")
            .iter()
            .any(|joiner| *joiner == operator.value)
    {
        return false;
    }
    let alphabetic = |value: &str| value.chars().all(char::is_alphabetic);
    alphabetic(&left.value)
        && alphabetic(&right.value)
        && (left.value.chars().count() > 1 || right.value.chars().count() > 1)
}

/// The operator relations of `text` and the unresolved structure among them.
#[must_use]
pub fn symbolic_relations(text: &str, terms: &[&Word]) -> (Vec<Relation>, Vec<String>) {
    let items = symbolic_items(text);
    let mut relations = Vec::new();
    let mut unresolved = Vec::new();
    let mut previous: Option<(usize, &'static str)> = None;
    for (index, item) in items.iter().enumerate() {
        let SymbolicKind::Operator(kind) = item.kind else {
            continue;
        };
        let (Some(left), Some(right)) = (
            index.checked_sub(1).and_then(|at| items.get(at)),
            items.get(index + 1),
        ) else {
            continue;
        };
        if left.kind != SymbolicKind::Operand
            || right.kind != SymbolicKind::Operand
            || joins_compound(left, item, right)
        {
            continue;
        }
        relations.push(relation(
            kind.op,
            kind.converse,
            operand_id(&left.value, terms.iter().copied()),
            operand_id(&right.value, terms.iter().copied()),
        ));
        if previous.is_some_and(|(at, op)| at + 2 == index && op != kind.op) {
            unresolved.push(Unresolved::OperatorChain.slug().to_owned());
        }
        previous = Some((index, kind.op));
    }
    (relations, unresolved)
}

struct Role {
    token: ShapeToken,
    index: usize,
    comparison: Option<RelationKind>,
}

fn sorted(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The proposition structure of a clause: `words` are its words
/// (`text_statements::read_words`), `terms` its statement terms, whose ids
/// name operands.
#[must_use]
pub fn read_proposition_structure(
    text: &str,
    language: &str,
    words: &[Word],
    terms: &[&Word],
) -> PropositionStructure {
    let phrases = structure_phrases(language);
    let surfaces: Vec<String> = words
        .iter()
        .map(|word| word.surface.to_lowercase())
        .collect();
    let mut roles = Vec::new();
    let mut repetition = Vec::new();
    let mut index = 0;
    while index < words.len() {
        if let Some((entry, width)) = phrase_at(&surfaces, index, &phrases) {
            match entry.role {
                PhraseRole::Repetition => repetition.push(entry.kind.clone()),
                PhraseRole::Connective(token) => roles.push(Role {
                    token,
                    index,
                    comparison: None,
                }),
            }
            index += width;
            continue;
        }
        let word = &words[index];
        let quantifier = if word.kind == WordKind::Meaning {
            quantifier_of(&word.id)
        } else {
            None
        };
        if word.kind == WordKind::Negation {
            roles.push(Role {
                token: ShapeToken::Negation,
                index,
                comparison: None,
            });
        } else if let Some(token) = quantifier {
            roles.push(Role {
                token,
                index,
                comparison: None,
            });
        } else if word.kind.is_term() {
            roles.push(Role {
                token: ShapeToken::Content,
                index,
                comparison: relation_of(COMPARISON_RULE, &word.id),
            });
        }
        index += 1;
    }
    let (mut relations, mut unresolved) = symbolic_relations(text, terms);
    let operand = |position: usize, forward: bool| -> Option<String> {
        let mut at = position;
        loop {
            at = if forward { at + 1 } else { at.checked_sub(1)? };
            let role = roles.get(at)?;
            if role.token == ShapeToken::Content && role.comparison.is_none() {
                let word = &words[role.index];
                return Some(operand_id(
                    &word.surface,
                    terms.iter().copied().chain(std::iter::once(word)),
                ));
            }
        }
    };
    for (position, role) in roles.iter().enumerate() {
        let Some(comparison) = role.comparison else {
            continue;
        };
        match (operand(position, false), operand(position, true)) {
            (Some(left), Some(right)) => {
                relations.push(relation(comparison.op, comparison.converse, left, right));
            }
            _ => unresolved.push(format!(
                "{}:{}",
                Unresolved::RelationOperand.slug(),
                words[role.index].id
            )),
        }
    }
    let mut tokens: Vec<ShapeToken> = Vec::new();
    for role in &roles {
        if !(role.token == ShapeToken::Content && tokens.last() == Some(&ShapeToken::Content)) {
            tokens.push(role.token);
        }
    }
    let leading = tokens
        .iter()
        .take_while(|token| **token == ShapeToken::Conjunction)
        .count();
    tokens = tokens.split_off(leading);
    while tokens.last().is_some_and(|token| token.connective()) {
        tokens.pop();
    }
    let has = |token: ShapeToken| tokens.contains(&token);
    let significant = has(ShapeToken::Disjunction)
        || has(ShapeToken::Universal)
        || has(ShapeToken::Existential)
        || (has(ShapeToken::Negation) && has(ShapeToken::Conjunction));
    let operands = relations
        .iter()
        .flat_map(|entry| entry.operands.iter().cloned())
        .collect();
    PropositionStructure {
        shape: tokens
            .iter()
            .map(|token| token.slug())
            .collect::<Vec<_>>()
            .join(" "),
        significant,
        relations: sorted(relations.into_iter().map(|entry| entry.text).collect()),
        operands: sorted(operands),
        repetition: sorted(repetition),
        unresolved: sorted(unresolved),
    }
}

/// The part of a statement identity the structure adds (empty when none).
#[must_use]
pub fn structure_key(structure: &PropositionStructure) -> String {
    let mut parts = Vec::new();
    if structure.significant {
        parts.push(format!("shape={}", structure.shape));
    }
    if !structure.relations.is_empty() {
        parts.push(format!("relations={}", structure.relations.join(" ")));
    }
    if !structure.repetition.is_empty() {
        parts.push(format!("repetition={}", structure.repetition.join(" ")));
    }
    if !structure.unresolved.is_empty() {
        parts.push(format!("unresolved={}", structure.unresolved.join(" ")));
    }
    parts.join(";")
}

/// Whether the shape is only content joined by conjunctions.
#[must_use]
pub fn conjunctive_shape(structure: &PropositionStructure) -> bool {
    structure.shape.split(' ').all(|token| {
        token == ShapeToken::Content.slug()
            || token == ShapeToken::Conjunction.slug()
            || token.is_empty()
    })
}
