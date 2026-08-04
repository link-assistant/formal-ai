//! Caller-framing vocabulary loaded from `data/seed/caller-context.lino`
//! (issue #907).
//!
//! A CLI wraps the user's request in framing of its own: the gemini CLI opens
//! every turn with a `<session_context>` block whose second line is *"Today's
//! date is Sunday, August 2, 2026 (formatted according to the user's locale)."*,
//! Claude Code and Qwen Code use `<system-reminder>`, codex uses
//! `<environment_context>`, agent and opencode use `<env>`. None of it is the
//! user talking, so none of it may decide what the turn asks for.
//!
//! Two vocabularies serve that separation:
//!
//! * **injected blocks** — the marker tags whose contents are the client
//!   describing itself, removed before the request is read;
//! * **fact-statement copulas** — the linking words that make a sentence a
//!   *statement about* something (*"today's date **is** Sunday"*) rather than a
//!   request for it (*"what is today's date?"*). A statement carries no intent;
//! * **question words** — the words that make a sentence a question even when
//!   the client dropped the question mark (*"what day is it"*, *"什么"*);
//! * **subject leads** — the determiners a subject may open with, so *"**the**
//!   current date is 2026-08-02"* is read as the same subject as *"current
//!   date"*.
//!
//! Like every other trigger vocabulary the natural language lives in seed data
//! rather than in the solver, so a maintainer adds a client or a language by
//! editing a `.lino` file.

use super::parser::{parse_lino, LinoNode};
use super::CALLER_CONTEXT_LINO;

/// One client-injected block: the marker tag and the clients that send it.
#[derive(Debug, Clone, Default)]
pub struct InjectedBlock {
    /// Marker tag name, e.g. `session_context`.
    pub tag: String,
    /// Clients observed sending this block, for documentation and tests.
    pub clients: Vec<String>,
}

impl InjectedBlock {
    /// The opening marker as it appears in a request body.
    #[must_use]
    pub fn open(&self) -> String {
        format!("<{}>", self.tag)
    }

    /// The closing marker as it appears in a request body.
    #[must_use]
    pub fn close(&self) -> String {
        format!("</{}>", self.tag)
    }
}

/// The caller-framing vocabulary: injected block markers and fact-statement
/// copulas, pooled across languages.
#[derive(Debug, Clone, Default)]
pub struct CallerContextVocabulary {
    /// Blocks a client wraps around its own context.
    pub injected_blocks: Vec<InjectedBlock>,
    /// Linking words that turn a phrase into a statement about its subject.
    pub fact_statement_copulas: Vec<String>,
    /// Words that make a sentence a question even without a question mark.
    pub question_words: Vec<String>,
    /// Determiners a subject may open with ("**the** current date is …").
    pub subject_leads: Vec<String>,
}

impl CallerContextVocabulary {
    /// The fact-statement copula `token` carries, if it carries one.
    ///
    /// Space-separated scripts match a whole token; a copula written without
    /// word spacing (Chinese `是`) matches as a substring, because the sentence
    /// it links carries no space to tokenize on.
    #[must_use]
    pub fn copula_in(&self, token: &str) -> Option<&str> {
        let token = token.trim_matches(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '—' | '–' | '-')
        });
        self.fact_statement_copulas
            .iter()
            .find(|copula| {
                copula.as_str() == token
                    || (copula.chars().any(is_unspaced_script) && token.contains(copula.as_str()))
            })
            .map(String::as_str)
    }

    /// Whether `sentence` carries a question word, which makes it a question
    /// even when the client dropped the question mark (*"what is the date"*).
    #[must_use]
    pub fn asks_a_question(&self, sentence: &str) -> bool {
        self.question_words.iter().any(|word| {
            if word.chars().any(is_unspaced_script) {
                return sentence.contains(word.as_str());
            }
            sentence
                .split(|character: char| !character.is_alphanumeric())
                .any(|token| token == word)
        })
    }
}

/// Whether `character` belongs to a script written without spaces between words.
const fn is_unspaced_script(character: char) -> bool {
    matches!(character, '\u{3400}'..='\u{9fff}' | '\u{f900}'..='\u{faff}')
}

/// Parse `data/seed/caller-context.lino` into the caller-framing vocabulary.
#[must_use]
pub fn caller_context_vocabulary() -> CallerContextVocabulary {
    let tree = parse_lino(CALLER_CONTEXT_LINO);
    let mut vocab = CallerContextVocabulary::default();
    let Some(root) = tree.children.first() else {
        return vocab;
    };
    for group in &root.children {
        match group.name.as_str() {
            "injected_blocks" => {
                vocab.injected_blocks = group
                    .children
                    .iter()
                    .filter(|child| child.name == "block")
                    .map(|block| InjectedBlock {
                        tag: block.id.clone(),
                        clients: block
                            .children
                            .iter()
                            .filter(|child| child.name == "client")
                            .map(|child| child.id.clone())
                            .collect(),
                    })
                    .collect();
            }
            "fact_statement_copulas" => {
                vocab.fact_statement_copulas = collect_language_values(group, "copula");
            }
            "question_words" => {
                vocab.question_words = collect_language_values(group, "word");
            }
            "subject_leads" => {
                vocab.subject_leads = collect_language_values(group, "lead");
            }
            _ => {}
        }
    }
    vocab
}

/// Collect every `<child_name>` id nested under the `language` children of
/// `group`, pooled across all languages (framing detection is language-agnostic).
fn collect_language_values(group: &LinoNode, child_name: &str) -> Vec<String> {
    group
        .children
        .iter()
        .filter(|child| child.name == "language")
        .flat_map(|language| {
            language
                .children
                .iter()
                .filter(|child| child.name == child_name)
                .map(|child| child.id.to_lowercase())
        })
        .collect()
}
