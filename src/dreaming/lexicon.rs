//! Data-grounded lexicon for dreaming's event detection (issues #540/#494).
//!
//! Task detection, topic derivation, and durability classification used to be
//! gated on hardcoded English keyword lists, which made non-English memories
//! (e.g. an event with intent `решить` or kind `проверка`) invisible to the
//! learning pipeline. The cue lists now live in
//! `data/meta/dreaming-lexicon.lino`. At runtime the file is loaded from disk
//! (`FORMAL_AI_DATA_DIR` first, then the repository-relative path) so operators
//! can extend the lexicon without recompiling; the compiled-in copy remains the
//! fallback so the binary stays self-contained. The parse is cached process-wide
//! instead of being re-done per event.

use std::path::PathBuf;
use std::sync::OnceLock;

const EMBEDDED_LEXICON: &str = include_str!("../../data/meta/dreaming-lexicon.lino");

/// The parsed cue lists dreaming consults.
#[derive(Debug, Default, Clone)]
pub struct DreamingLexicon {
    pub task_kind_cues: Vec<String>,
    pub task_intent_cues: Vec<String>,
    pub topic_stopwords: Vec<String>,
    pub learning_kind_cues: Vec<String>,
    pub learning_content_cues: Vec<String>,
    pub cache_kind_cues: Vec<String>,
    pub cache_tool_cues: Vec<String>,
    pub intermediate_kind_cues: Vec<String>,
}

impl DreamingLexicon {
    fn push(&mut self, key: &str, value: String) {
        match key {
            "task_kind_cue" => self.task_kind_cues.push(value),
            "task_intent_cue" => self.task_intent_cues.push(value),
            "topic_stopword" => self.topic_stopwords.push(value),
            "learning_kind_cue" => self.learning_kind_cues.push(value),
            "learning_content_cue" => self.learning_content_cues.push(value),
            "cache_kind_cue" => self.cache_kind_cues.push(value),
            "cache_tool_cue" => self.cache_tool_cues.push(value),
            "intermediate_kind_cue" => self.intermediate_kind_cues.push(value),
            _ => {}
        }
    }
}

/// Parse a `dreaming_lexicon` Links-Notation document into cue lists.
#[must_use]
pub fn parse_lexicon(text: &str) -> DreamingLexicon {
    let mut lexicon = DreamingLexicon::default();
    for line in text.lines() {
        let line = line.trim();
        let Some((key, rest)) = line.split_once(' ') else {
            continue;
        };
        let Some(value) = rest
            .trim()
            .strip_prefix('"')
            .and_then(|tail| tail.strip_suffix('"'))
        else {
            continue;
        };
        lexicon.push(key, value.to_lowercase());
    }
    lexicon
}

/// Resolve the on-disk location of a `data/meta` document, honouring
/// `FORMAL_AI_DATA_DIR` so deployments can override shipped lexicons/cues.
#[must_use]
pub fn data_document_path(file_name: &str) -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("FORMAL_AI_DATA_DIR") {
        let trimmed = dir.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed).join(file_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let repo_relative = PathBuf::from("data/meta").join(file_name);
    repo_relative.is_file().then_some(repo_relative)
}

/// Load a `data/meta` document from disk with a compiled-in fallback.
#[must_use]
pub fn load_data_document(file_name: &str, embedded: &'static str) -> String {
    data_document_path(file_name)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .unwrap_or_else(|| embedded.to_owned())
}

/// The process-wide lexicon, parsed once.
pub fn lexicon() -> &'static DreamingLexicon {
    static LEXICON: OnceLock<DreamingLexicon> = OnceLock::new();
    LEXICON.get_or_init(|| {
        parse_lexicon(&load_data_document(
            "dreaming-lexicon.lino",
            EMBEDDED_LEXICON,
        ))
    })
}
