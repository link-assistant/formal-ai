//! Word-by-word sentence translation (issue #1174).
//!
//! A sentence with no lexical route through
//! [`TranslationPipeline::translate`](super::pipeline::TranslationPipeline::translate)
//! is answered word by word instead of declined: each content word resolves
//! on its own, function words named by the seed's stop-word role are dropped,
//! numerals pass through unchanged, and words that resolve to no target
//! surface keep their source form and are reported by
//! [`SentenceTranslation::unknown_words`]. The split keeps
//! [`super::pipeline`] under the file-size gate; the sentence arm lives here.

use super::http::HttpClient;
use super::pipeline::{TranslationPipeline, capitalize_ascii_first};

/// The role whose words a word-by-word sentence rendering leaves out
/// (articles, auxiliaries, pronouns) instead of echoing them in the source
/// script. The stop-word lists per language live in
/// `data/seed/meanings-summarization.lino`; this names only the role.
const ROLE_TRANSLATION_STOP_WORD: &str = "translation_stop_word";

/// One word of a [`SentenceTranslation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordTranslation {
    /// The source token as it appeared in the sentence.
    pub source: String,
    /// The target-language surface, or `None` when no translation resolved.
    pub target: Option<String>,
    /// The meaning id (Wikidata item or seed slug) the word formalized to.
    pub meaning: Option<String>,
    /// A function word: present in the source, deliberately absent from the
    /// rendered surface.
    pub dropped_function_word: bool,
    /// The source token began with a capital letter.
    pub capitalized: bool,
}

/// A word-by-word translation of one sentence (issue #1174).
///
/// Partial by design and honest about it: each content word resolves through
/// [`TranslationPipeline::translate`] on its own, function words named by
/// `ROLE_TRANSLATION_STOP_WORD` are dropped, numerals pass through
/// unchanged, and words that resolve to no target surface keep their source
/// form and are reported by [`SentenceTranslation::unknown_words`]. The
/// rendering joins words with the separator the target language's statement
/// template in `data/seed/formal-language-projections.lino` implies; no
/// part-of-speech reordering is attempted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceTranslation {
    /// The source sentence.
    pub source: String,
    /// Source language slug.
    pub source_lang: String,
    /// Target language slug.
    pub target_lang: String,
    /// The words in source order.
    pub words: Vec<WordTranslation>,
    /// The target statement template the surface was rendered with.
    pub target_order: String,
    /// The source sentence began with a capital letter.
    pub leads_capitalized: bool,
}

impl SentenceTranslation {
    /// Render the sentence in the target language: dropped function words
    /// are absent, unknown words keep their source form, capitalized source
    /// words stay capitalized, and the leading word is capitalized when the
    /// source sentence was.
    #[must_use]
    pub fn surface(&self) -> String {
        let separator = template_separator(&self.target_order);
        let rendered: Vec<String> = self
            .words
            .iter()
            .filter(|word| !word.dropped_function_word)
            .map(|word| {
                let text = word.target.as_deref().unwrap_or(&word.source);
                if word.capitalized {
                    capitalize_ascii_first(text)
                } else {
                    text.to_owned()
                }
            })
            .collect();
        if rendered.is_empty() {
            return String::new();
        }
        let mut out = rendered.join(&separator);
        if self.leads_capitalized {
            out = capitalize_ascii_first(&out);
        }
        out
    }

    /// The content words that resolved to no target surface, in source
    /// order. Callers report them instead of hiding the gap.
    #[must_use]
    pub fn unknown_words(&self) -> Vec<&str> {
        self.words
            .iter()
            .filter(|word| !word.dropped_function_word && word.target.is_none())
            .map(|word| word.source.as_str())
            .collect()
    }
}

impl<T: HttpClient + ?Sized> TranslationPipeline<'_, T> {
    /// Translate one sentence word by word (issue #1174).
    ///
    /// Segmentation belongs to the caller (`crate::formalization::segment`);
    /// this method walks the sentence's whitespace tokens, drops the function
    /// words `ROLE_TRANSLATION_STOP_WORD` names in the source language,
    /// passes numerals through unchanged, and translates every remaining
    /// content word through [`Self::translate`] on its own. A word that
    /// resolves to no target surface stays in the source language and is
    /// reported by [`SentenceTranslation::unknown_words`].
    #[must_use]
    pub fn translate_sentence(
        &self,
        source: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> SentenceTranslation {
        let stop_words = crate::seed::lexicon()
            .words_for_role_in_languages(ROLE_TRANSLATION_STOP_WORD, &[source_lang]);
        let leads_capitalized = source.chars().next().is_some_and(char::is_uppercase);
        let mut words = Vec::new();
        for core in sentence_tokens(source) {
            let lower = core.to_lowercase();
            let capitalized = core.chars().next().is_some_and(char::is_uppercase);
            if stop_words.iter().any(|word| word == &lower) {
                words.push(WordTranslation {
                    source: core,
                    target: None,
                    meaning: None,
                    dropped_function_word: true,
                    capitalized,
                });
                continue;
            }
            if !lower.is_empty() && lower.chars().all(char::is_numeric) {
                words.push(WordTranslation {
                    source: core.clone(),
                    target: Some(core),
                    meaning: None,
                    dropped_function_word: false,
                    capitalized,
                });
                continue;
            }
            let resolved = self
                .translate(&lower, source_lang, target_lang)
                .ok()
                .and_then(|translation| {
                    translation
                        .primary_surface()
                        .map(|surface| (surface.to_owned(), translation.meaning.slug()))
                });
            let (target, meaning) = match resolved {
                Some((surface, meaning)) => (Some(surface), Some(meaning)),
                None => (None, None),
            };
            words.push(WordTranslation {
                source: core,
                target,
                meaning,
                dropped_function_word: false,
                capitalized,
            });
        }
        SentenceTranslation {
            source: source.trim().to_owned(),
            source_lang: source_lang.to_owned(),
            target_lang: target_lang.to_owned(),
            words,
            target_order: target_statement_template(target_lang),
            leads_capitalized,
        }
    }
}

/// The content cores of a sentence's whitespace tokens: leading and trailing
/// punctuation is trimmed (apostrophes stay, so contractions keep their
/// shape), and tokens with no alphanumeric core are dropped.
fn sentence_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| {
            token.trim_matches(|character: char| {
                !character.is_alphanumeric() && character != '\'' && character != '’'
            })
        })
        .filter(|core| !core.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The word separator a statement template implies: the text between the
/// `{subject}` and `{predicate}` slots — a space for English-like orders,
/// nothing for Chinese.
fn template_separator(template: &str) -> String {
    template
        .strip_prefix("{subject}")
        .and_then(|rest| rest.split("{predicate}").next())
        .unwrap_or(" ")
        .to_owned()
}

/// The target language's statement template from
/// `data/seed/formal-language-projections.lino` — the word order a
/// [`SentenceTranslation`] renders in. Falls back to the English subject
/// predicate object order when the language has no projection seeded.
fn target_statement_template(target_lang: &str) -> String {
    for (registered, text) in crate::seed::seed_files() {
        if registered != "data/seed/formal-language-projections.lino" {
            continue;
        }
        let tree = crate::seed::parser::parse_lino(text);
        for language in tree.children.iter().flat_map(|root| root.children.iter()) {
            if language.name == "natural_language" && language.id.eq_ignore_ascii_case(target_lang)
            {
                let statement = language.find_child_value("statement");
                if !statement.is_empty() {
                    return statement.to_owned();
                }
            }
        }
    }
    String::from("{subject} {predicate} {object}")
}
