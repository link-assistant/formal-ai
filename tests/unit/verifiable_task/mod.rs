//! Issue #1138 B8 (plan 08): one verifiable-task route, in any domain.
//!
//! Every prompt these tests use lives in
//! `data/benchmarks/verifiable-task-paraphrases.lino`, never in `src/` or
//! `data/seed/`, and is read back from there so the corpus and the suite cannot
//! drift apart.

mod execution;
mod identity;
mod ledger;
mod no_memorization;
mod quantities;
mod recognition;
mod routing;

use std::fs;
use std::path::{Path, PathBuf};

pub const CORPUS: &str = "data/benchmarks/verifiable-task-paraphrases.lino";

/// The five languages the doctrine requires of every surface.
pub const LANGUAGES: &[&str] = &["en", "ru", "hi", "zh", "es"];

/// One held-out paraphrase: which family it belongs to, its language and the
/// prompt verbatim.
#[derive(Debug, Clone)]
pub struct Paraphrase {
    pub family: String,
    pub expectation: String,
    pub shape: String,
    pub language: String,
    pub prompt: String,
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn unquote(raw: &str) -> String {
    raw.trim()
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw.trim())
        .to_owned()
}

/// Every paraphrase in the corpus, in file order.
pub fn corpus() -> Vec<Paraphrase> {
    let text = fs::read_to_string(repo_root().join(CORPUS))
        .unwrap_or_else(|error| panic!("{CORPUS} should be readable: {error}"));

    let mut out = Vec::new();
    let mut family = String::new();
    let mut expectation = String::new();
    let mut shape = String::new();
    let mut language = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) {
            family = trimmed.to_owned();
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("expectation ") {
            expectation = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("shape ") {
            shape = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("language ") {
            language = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("prompt ") {
            out.push(Paraphrase {
                family: family.clone(),
                expectation: expectation.clone(),
                shape: shape.clone(),
                language: language.clone(),
                prompt: unquote(value),
            });
        }
    }
    out
}

/// Every paraphrase of one corpus family, keyed by the family's `id`.
pub fn family(id: &str) -> Vec<Paraphrase> {
    corpus()
        .into_iter()
        .filter(|case| case.family.ends_with(id))
        .collect()
}

/// One paraphrase of a family, in a given language.
pub fn case(id: &str, language: &str) -> Paraphrase {
    family(id)
        .into_iter()
        .find(|case| case.language == language)
        .unwrap_or_else(|| panic!("corpus family `{id}` must carry a {language} paraphrase"))
}
