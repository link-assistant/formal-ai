//! Dependency summarization (R1188-U21).
//!
//! A summary chosen by what the statements of a text depend on, with no
//! language model. It mirrors `js/agentic/crate/dependency_summarization.mjs`,
//! the JavaScript root.
//!
//! The text is formalized into statements
//! ([`crate::formalization::text_statements`]). A statement whose formal
//! content an earlier statement already holds is a duplicate and is dropped:
//! the same polarity, subject and terms, or a term set inside an earlier
//! one's, whatever the wording. A later statement depends on an earlier one
//! when it mentions the earlier statement's subject, or shares at least
//! [`ELABORATION_SHARED_TERMS`] terms with it. The kept core is the roots that
//! something depends on, then the most depended-on statements, up to one
//! statement in [`KEEP_DIVISOR`], in text order, rendered with the surfaces
//! they were read from.

use crate::formalization::statement_rendering::join_surfaces;
use crate::formalization::text_statements::{
    Statement, content_ids, formalize_sentences, is_word_character, statement_identity,
};

/// Shared terms that make one statement elaborate another.
pub const ELABORATION_SHARED_TERMS: usize = 2;

/// The summary keeps at most one statement in this many.
pub const KEEP_DIVISOR: usize = 3;

/// One statement with the sentence it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The statement.
    pub statement: Statement,
    /// The index of its sentence.
    pub sentence: usize,
    /// The sentence's closing punctuation, or empty.
    pub closing: String,
}

/// One statement in the dependency graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The statement and its sentence.
    pub entry: Entry,
    /// The earlier statements it depends on.
    pub depends_on: Vec<usize>,
    /// The later statements that depend on it.
    pub dependents: Vec<usize>,
}

/// The kept core of a text and its rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreSummary {
    /// The statements left after duplicates were dropped, as graph nodes.
    pub statements: Vec<Node>,
    /// How many statements restated an earlier one.
    pub duplicates: usize,
    /// The statements that restated an earlier one, in text order.
    pub removed: Vec<Entry>,
    /// The indexes of the kept statements, in text order.
    pub kept: Vec<usize>,
    /// The kept statements in the text's own words.
    pub text: String,
}

/// The last character of a sentence when it is punctuation, else empty.
#[must_use]
pub fn closing_of(sentence: &str) -> String {
    sentence
        .chars()
        .last()
        .filter(|last| !is_word_character(*last))
        .map(String::from)
        .unwrap_or_default()
}

/// Every statement of `text` with its sentence and that sentence's closing
/// punctuation.
#[must_use]
pub fn sentence_statements(text: &str, language: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    for (index, sentence) in formalize_sentences(text, language).into_iter().enumerate() {
        let closing = closing_of(&sentence.text);
        for statement in sentence.statements {
            out.push(Entry {
                statement,
                sentence: index,
                closing: closing.clone(),
            });
        }
    }
    out
}

/// Whether `later` adds nothing to `earlier`.
///
/// It has the same formal identity, or its terms all appear in `earlier`'s
/// with the same polarity.
#[must_use]
pub fn restates(later: &Statement, earlier: &Statement) -> bool {
    if statement_identity(later) == statement_identity(earlier) {
        return true;
    }
    let ids = content_ids(later);
    let held = content_ids(earlier);
    !ids.is_empty() && later.polarity == earlier.polarity && ids.iter().all(|id| held.contains(id))
}

/// The statements no earlier kept statement restates, and the ones that
/// restate one (the duplicates removed), each in text order.
#[must_use]
pub fn split_duplicates(entries: Vec<Entry>) -> (Vec<Entry>, Vec<Entry>) {
    let mut unique: Vec<Entry> = Vec::new();
    let mut removed = Vec::new();
    for entry in entries {
        if unique
            .iter()
            .any(|earlier| restates(&entry.statement, &earlier.statement))
        {
            removed.push(entry);
            continue;
        }
        unique.push(entry);
    }
    (unique, removed)
}

/// The statements no earlier kept statement restates, and how many were
/// dropped.
#[must_use]
pub fn without_duplicates(entries: Vec<Entry>) -> (Vec<Entry>, usize) {
    let (unique, removed) = split_duplicates(entries);
    (unique, removed.len())
}

/// How many distinct term ids two statements share.
#[must_use]
pub fn shared_terms(left: &Statement, right: &Statement) -> usize {
    let held = content_ids(right);
    content_ids(left)
        .iter()
        .filter(|id| held.contains(id))
        .count()
}

/// Whether `later` mentions `earlier`'s subject, or shares enough terms with
/// it to elaborate it.
#[must_use]
pub fn depends_on(later: &Statement, earlier: &Statement) -> bool {
    if let Some(subject) = &earlier.subject
        && content_ids(later).contains(&subject.id)
    {
        return true;
    }
    shared_terms(later, earlier) >= ELABORATION_SHARED_TERMS
}

/// For every statement, the earlier statements it depends on and the later
/// ones that depend on it.
#[must_use]
pub fn statement_graph(entries: Vec<Entry>) -> Vec<Node> {
    let mut nodes: Vec<Node> = entries
        .into_iter()
        .map(|entry| Node {
            entry,
            depends_on: Vec::new(),
            dependents: Vec::new(),
        })
        .collect();
    for later in 0..nodes.len() {
        for earlier in 0..later {
            if !depends_on(
                &nodes[later].entry.statement,
                &nodes[earlier].entry.statement,
            ) {
                continue;
            }
            nodes[later].depends_on.push(earlier);
            nodes[earlier].dependents.push(later);
        }
    }
    nodes
}

/// One statement in [`KEEP_DIVISOR`], at least one.
#[must_use]
pub const fn keep_budget(count: usize) -> usize {
    count.div_ceil(KEEP_DIVISOR)
}

/// The indexes of the kept statements, in text order.
///
/// Roots something depends on come first, then the rest, each group by
/// dependents (most first) and text order.
#[must_use]
pub fn kept_core(nodes: &[Node]) -> Vec<usize> {
    let by_dependents = |left: &usize, right: &usize| {
        nodes[*right]
            .dependents
            .len()
            .cmp(&nodes[*left].dependents.len())
            .then(left.cmp(right))
    };
    let (mut roots, mut rest): (Vec<usize>, Vec<usize>) = (0..nodes.len()).partition(|index| {
        nodes[*index].depends_on.is_empty() && !nodes[*index].dependents.is_empty()
    });
    roots.sort_by(by_dependents);
    rest.sort_by(by_dependents);
    let mut kept: Vec<usize> = roots
        .into_iter()
        .chain(rest)
        .take(keep_budget(nodes.len()))
        .collect();
    kept.sort_unstable();
    kept
}

/// The kept statements in their own words.
///
/// A fragment that ends its sentence here is closed with the sentence's
/// punctuation.
#[must_use]
pub fn render_kept(nodes: &[Node], kept: &[usize]) -> String {
    let pieces: Vec<String> = kept
        .iter()
        .enumerate()
        .map(|(position, index)| {
            let entry = &nodes[*index].entry;
            let ends_sentence = kept
                .get(position + 1)
                .is_none_or(|next| nodes[*next].entry.sentence != entry.sentence);
            let text = &entry.statement.text;
            if ends_sentence && !entry.closing.is_empty() && !text.ends_with(entry.closing.as_str())
            {
                format!("{text}{}", entry.closing)
            } else {
                text.clone()
            }
        })
        .collect();
    join_surfaces(&pieces)
}

/// The kept core of `text` and its rendering in the text's own words.
#[must_use]
pub fn summarize_by_dependency(text: &str, language: &str) -> CoreSummary {
    let (unique, removed) = split_duplicates(sentence_statements(text, language));
    let statements = statement_graph(unique);
    let kept = kept_core(&statements);
    let text = render_kept(&statements, &kept);
    CoreSummary {
        statements,
        duplicates: removed.len(),
        removed,
        kept,
        text,
    }
}

/// The gold facts of a text: the distinct term ids of the first sentence that
/// yields a statement (its subject and the facts that define it).
#[must_use]
pub fn key_facts(text: &str, language: &str) -> Vec<String> {
    let entries = sentence_statements(text, language);
    let Some(first) = entries.first().map(|entry| entry.sentence) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = entries
        .iter()
        .filter(|entry| entry.sentence == first)
        .flat_map(|entry| content_ids(&entry.statement))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// How many of `facts` the kept statements still hold.
#[must_use]
pub fn retained_facts(summary: &CoreSummary, facts: &[String]) -> usize {
    let held: Vec<String> = summary
        .kept
        .iter()
        .flat_map(|index| content_ids(&summary.statements[*index].entry.statement))
        .collect();
    facts.iter().filter(|fact| held.contains(fact)).count()
}
