//! The universal proof engine's theorems, plans and wording, loaded from seed
//! data (R379).
//!
//! `data/seed/proof-library.lino` holds every sentence the proof engine shows
//! a reader -- the classical theorems it discharges by lookup, the partial
//! plans it offers when a claim is not yet closed, and the presentation
//! phrases -- in every surface language, so `rust/src/proof_engine` carries
//! structure and no prose. The browser worker reads the same file.

use std::sync::OnceLock;

use super::embedded::PROOF_LIBRARY_LINO;
use super::fill_template_once;
use super::parser::{LinoNode, parse_lino};

/// One phrase in every language the seed spells it in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalizedText {
    /// `(language slug, text)` pairs in seed order.
    pub surfaces: Vec<(String, String)>,
}

impl LocalizedText {
    /// The surface for `language`, falling back to English and then to `""`.
    #[must_use]
    pub fn get(&self, language: &str) -> &str {
        self.find(language)
            .or_else(|| self.find("en"))
            .unwrap_or_default()
    }

    fn find(&self, language: &str) -> Option<&str> {
        self.surfaces
            .iter()
            .find(|(slug, _)| slug == language)
            .map(|(_, text)| text.as_str())
    }

    fn from_node(node: &LinoNode) -> Self {
        Self {
            surfaces: node
                .children
                .iter()
                .filter(|child| child.children.is_empty() && !child.id.is_empty())
                .map(|child| (child.name.clone(), child.id.clone()))
                .collect(),
        }
    }
}

/// One step of a theorem or plan: its kind slug and its localized text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofLibraryStep {
    /// `hypothesis`, `definition`, `axiom`, `inference`, `sub_proof` or
    /// `conclusion`.
    pub kind: String,
    pub text: LocalizedText,
}

/// A `theorem` or `plan` record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProofLibraryEntry {
    pub id: String,
    /// The proof-method slug, e.g. `contradiction`.
    pub method: String,
    /// Lowercase substrings that select a theorem from a normalized claim.
    pub keywords: Vec<String>,
    pub statement: LocalizedText,
    pub steps: Vec<ProofLibraryStep>,
    pub conclusion: LocalizedText,
    /// The inputs a plan still needs from the reader.
    pub missing_inputs: Vec<LocalizedText>,
}

impl ProofLibraryEntry {
    /// Whether one of the theorem's keywords occurs in `normalized`.
    #[must_use]
    pub fn matches(&self, normalized: &str) -> bool {
        self.keywords
            .iter()
            .any(|keyword| normalized.contains(keyword.as_str()))
    }

    fn from_node(node: &LinoNode) -> Self {
        let mut entry = Self {
            id: node.id.clone(),
            method: node.find_child_value("method").to_owned(),
            ..Self::default()
        };
        for child in &node.children {
            match child.name.as_str() {
                "keyword" => entry.keywords.push(child.id.clone()),
                "statement" => entry.statement = LocalizedText::from_node(child),
                "conclusion" => entry.conclusion = LocalizedText::from_node(child),
                "missing_input" => entry.missing_inputs.push(LocalizedText::from_node(child)),
                "step" => entry.steps.push(ProofLibraryStep {
                    kind: child.id.clone(),
                    text: LocalizedText::from_node(child),
                }),
                _ => {}
            }
        }
        entry
    }
}

/// A `template` record: one phrase, or a list of item phrases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofTemplate {
    pub id: String,
    pub text: LocalizedText,
    pub items: Vec<LocalizedText>,
}

/// The parsed proof library.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProofLibrary {
    /// Theorems in lookup order.
    pub theorems: Vec<ProofLibraryEntry>,
    pub plans: Vec<ProofLibraryEntry>,
    pub templates: Vec<ProofTemplate>,
}

impl ProofLibrary {
    /// The first theorem whose keyword occurs in `normalized`.
    #[must_use]
    pub fn theorem_for(&self, normalized: &str) -> Option<&ProofLibraryEntry> {
        self.theorems.iter().find(|entry| entry.matches(normalized))
    }

    /// The plan named `id`.
    #[must_use]
    pub fn plan(&self, id: &str) -> Option<&ProofLibraryEntry> {
        self.plans.iter().find(|plan| plan.id == id)
    }

    fn template(&self, id: &str) -> Option<&ProofTemplate> {
        self.templates.iter().find(|template| template.id == id)
    }

    /// The phrase `id` in `language`, filling each `{name}` slot from
    /// `values` in one pass. A missing record degrades to the id -- a meaning,
    /// never invented prose -- so the gap stays visible.
    #[must_use]
    pub fn text(&self, id: &str, language: &str, values: &[(&str, &str)]) -> String {
        self.template(id).map_or_else(
            || id.to_owned(),
            |template| fill_template_once(template.text.get(language), values),
        )
    }

    /// The item phrases of the list template `id` in `language`.
    #[must_use]
    pub fn items(&self, id: &str, language: &str) -> Vec<String> {
        self.template(id)
            .map(|template| {
                template
                    .items
                    .iter()
                    .map(|item| item.get(language).to_owned())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// The proof library parsed once from `data/seed/proof-library.lino`.
#[must_use]
pub fn proof_library() -> &'static ProofLibrary {
    static LIBRARY: OnceLock<ProofLibrary> = OnceLock::new();
    LIBRARY.get_or_init(|| parse_proof_library(PROOF_LIBRARY_LINO))
}

/// Parse a proof library from Links Notation text.
#[must_use]
pub fn parse_proof_library(text: &str) -> ProofLibrary {
    let tree = parse_lino(text);
    let mut library = ProofLibrary::default();
    let Some(root) = tree
        .children
        .iter()
        .find(|node| node.name == "proof_library")
    else {
        return library;
    };
    for node in &root.children {
        match node.name.as_str() {
            "theorem" => library.theorems.push(ProofLibraryEntry::from_node(node)),
            "plan" => library.plans.push(ProofLibraryEntry::from_node(node)),
            "template" => library.templates.push(ProofTemplate {
                id: node.id.clone(),
                text: LocalizedText::from_node(node),
                items: node
                    .children
                    .iter()
                    .filter(|child| child.name == "item")
                    .map(LocalizedText::from_node)
                    .collect(),
            }),
            _ => {}
        }
    }
    library
}
