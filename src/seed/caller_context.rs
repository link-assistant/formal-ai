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
//!   date"*;
//! * **request verbs** — the verbs that make a sentence an order rather than a
//!   statement (*"**print** the current directory"*, *"**покажи** дату"*);
//! * **policy leads** — the words that open a clause governing *how* a class of
//!   actions is handled rather than requesting one (*"**when** running sudo
//!   commands, run them in the background"*). Caller workflow policy is not the
//!   user's request, so nothing inside such a clause may become a command.
//!
//! This is the single home for *"is this sentence asking, or telling?"*: every
//! router that has to tell a request from the framing around it reads these
//! lists rather than growing a copy of its own.
//!
//! Like every other trigger vocabulary the natural language lives in seed data
//! rather than in the solver, so a maintainer adds a client or a language by
//! editing a `.lino` file.

use super::CALLER_CONTEXT_LINO;
use super::parser::{LinoNode, parse_lino};

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
    /// Verbs that make a sentence an order rather than a statement
    /// ("**print** the current directory").
    pub request_verbs: Vec<String>,
    /// Words that open a clause governing *how* a class of actions is to be
    /// treated rather than asking for one ("**when** running sudo commands, run
    /// them in the background"). Such a clause is caller policy: nothing inside
    /// it names an action to take now, so no command token it mentions may be
    /// selected (issue #907, follow-up).
    ///
    /// Only words that are *unambiguously* conditional, temporal, or
    /// prohibitive belong here — a bare negation particle (Russian `не`) or a
    /// word that doubles as an affirmation (Spanish `si`) opens direct
    /// instructions too, and silencing those would cost the user a command they
    /// really did ask for.
    pub policy_leads: Vec<String>,
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
    /// even when the client dropped the question mark (*"what is the date"*),
    /// or a request verb, which makes it an order (*"print the date"*). Either
    /// way the sentence asks for something, so it is not a statement of fact.
    #[must_use]
    pub fn asks_a_question(&self, sentence: &str) -> bool {
        self.question_words
            .iter()
            .chain(self.request_verbs.iter())
            .any(|word| {
                if word.chars().any(is_unspaced_script) {
                    return sentence.contains(word.as_str());
                }
                // The apostrophe stays inside the token so a contracted question
                // word ("what's the date") is one word, not two.
                sentence
                    .split(|character: char| {
                        !character.is_alphanumeric() && !matches!(character, '\'' | '’')
                    })
                    .any(|token| token == word)
            })
    }

    /// The clause a [policy lead](Self::policy_leads) opens `lowercased` with.
    ///
    /// This is the strictest reading of a lead, and the one a route wants once
    /// it has already isolated the text an action marker introduced. *"Use web
    /// research when it materially improves factual accuracy"* introduces
    /// *"when it materially improves factual accuracy"*; the lead is `when` and
    /// the clause it governs is *"it materially improves factual accuracy"*.
    ///
    /// Returning the clause rather than a yes/no lets the caller inspect what
    /// the condition is *about*, which is what separates a rule from a request
    /// that merely opens the same way — *"look up when the next release ships"*
    /// opens with the same word and is an ordinary question.
    #[must_use]
    pub fn policy_lead_clause<'a>(&self, lowercased: &'a str) -> Option<&'a str> {
        let text = lowercased.trim_start();
        self.policy_leads.iter().find_map(|lead| {
            text.strip_prefix(lead.as_str())
                .filter(|rest| rest.is_empty() || !starts_with_word_character(rest))
                .map(str::trim_start)
        })
    }

    /// Whether `lowercased` *opens* with a [policy lead](Self::policy_leads).
    #[must_use]
    pub fn opens_with_policy_lead(&self, lowercased: &str) -> bool {
        self.policy_lead_clause(lowercased).is_some()
    }

    /// Whether `lowercased` carries a [policy lead](Self::policy_leads) at its
    /// start or from the middle of the clause.
    ///
    /// *"When running sudo commands, …"* opens with one; *"Run commands with
    /// sudo only when necessary"* qualifies from the middle. Both name a class
    /// rather than an instance, so both are the caller's rule.
    ///
    /// Carrying a lead is necessary but not sufficient: a genuine request can
    /// carry one too (*"If the build fails, run cargo test."*), so each caller
    /// pairs this with the test that tells its own orders from its own rules.
    #[must_use]
    pub fn carries_policy_lead(&self, lowercased: &str) -> bool {
        self.opens_with_policy_lead(lowercased)
            || self
                .policy_leads
                .iter()
                .any(|lead| lowercased.contains(&format!(" {lead} ")))
    }
}

/// Whether `text` begins with a character that continues a word, so a lead
/// matched by prefix would only be the head of a longer one (`if` in `iffy`).
fn starts_with_word_character(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(|character| character.is_alphanumeric() || character == '_')
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
            "request_verbs" => {
                vocab.request_verbs = collect_language_values(group, "verb");
            }
            "policy_leads" => {
                vocab.policy_leads = collect_language_values(group, "lead");
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
