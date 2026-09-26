//! The tree-sitter kind inventory of the committed corpora (issue #1138,
//! plan 16 L8), measured rather than remembered.
//!
//! The meta-language engine parses each grammar into a leaf-up network —
//! every node link references its parent, the node kind rides
//! `metadata.term()`, and leaves carry one token child holding the source
//! text — so the kind inventory is the exact rule checklist the
//! grammar-projection seed must cover: every kind a committed module parses
//! into is either ruled for the target or refused-by-name, and the corpus
//! ratchet test holds the seed to that list.
//!
//! `dump_grammar_kind_inventory` (an example) prints the inventory per
//! corpus; the L8 tests call [`corpus_inventory`] on the same derived
//! corpus sets so the checklist and the check can never drift apart.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One committed corpus the cycle owns.
///
/// Carries the grammar label, directory spelling, owned extension, and
/// whether the file set is derived from the self-AST census (the rust
/// corpus) rather than walked by extension.
#[derive(Debug, Clone, Copy)]
pub struct Corpus {
    /// The grammar label `LinkNetwork::parse` accepts.
    pub label: &'static str,
    /// The repository directory the corpus lives in.
    pub directory: &'static str,
    /// The owned extension the directory's files carry.
    pub extension: &'static str,
    /// True when the file set is the census set (rust), not a directory walk.
    pub census_derived: bool,
}

/// The corpora the three-source-root cycle owns.
pub const CORPORA: [Corpus; 3] = [
    Corpus {
        label: "rust",
        directory: "rust/src",
        extension: "rs",
        census_derived: true,
    },
    Corpus {
        label: "javascript",
        directory: "js",
        extension: "js",
        census_derived: false,
    },
    Corpus {
        label: "typescript",
        directory: "ts",
        extension: "ts",
        census_derived: false,
    },
];

/// The corpus whose grammar label matches, if any.
#[must_use]
pub fn corpus_by_label(label: &str) -> Option<&'static Corpus> {
    CORPORA.iter().find(|corpus| corpus.label == label)
}

/// The corpus's committed source files under `root`, in path order.
///
/// The rust corpus is the census set (the same derived set the round-trip
/// proof walks); the ES corpora are the committed `js/` and `ts/` trees
/// walked by extension. The inventory example, the seed generator and the
/// tests all call this, so the checklist and the check measure one set.
#[must_use]
pub fn corpus_sources(root: &Path, corpus: Corpus) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    if corpus.census_derived {
        let census_root = root.join("data").join("meta").join("self-ast");
        collect(&census_root, "lino", &mut sources);
        sources = sources
            .into_iter()
            .map(|census_path| {
                let relative = census_path
                    .strip_prefix(&census_root)
                    .expect("collected under the census tree")
                    .with_extension("rs");
                root.join("rust").join(relative)
            })
            .collect();
    } else {
        collect(&root.join(corpus.directory), corpus.extension, &mut sources);
    }
    sources
}

fn collect(directory: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, extension, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            out.push(path);
        }
    }
    out.sort();
}

/// Parse every readable source with its grammar and count each syntax-link
/// kind's occurrences.
///
/// Files that do not read are skipped silently — the corpus walk that
/// built the list already selected owned files.
#[must_use]
pub fn corpus_inventory(label: &str, sources: &[PathBuf]) -> BTreeMap<String, usize> {
    let mut inventory = BTreeMap::new();
    for source in sources {
        let Ok(text) = std::fs::read_to_string(source) else {
            continue;
        };
        for (kind, count) in source_kinds(label, &text) {
            *inventory.entry(kind).or_default() += count;
        }
    }
    inventory
}

/// The named-only inventory the projection coverage tables measure: a
/// template carries the anonymous keyword leaves and punctuation of the
/// kind it rules, so those kinds never need a row of their own.
#[must_use]
pub fn named_corpus_inventory(label: &str, sources: &[PathBuf]) -> BTreeMap<String, usize> {
    let mut inventory = BTreeMap::new();
    for source in sources {
        let Ok(text) = std::fs::read_to_string(source) else {
            continue;
        };
        for (kind, count) in named_source_kinds(label, &text) {
            *inventory.entry(kind).or_default() += count;
        }
    }
    inventory
}

/// The syntax-link kind histogram of one document under one grammar.
#[must_use]
pub fn source_kinds(label: &str, text: &str) -> Vec<(String, usize)> {
    kind_histogram(&parse_network(label, text))
}

/// The named-only histogram behind [`named_corpus_inventory`].
#[cfg(feature = "meta-language")]
#[must_use]
pub fn named_source_kinds(label: &str, text: &str) -> Vec<(String, usize)> {
    named_kind_histogram(&parse_network(label, text))
}

/// Without the engine there are no named kinds either.
#[cfg(not(feature = "meta-language"))]
#[must_use]
pub fn named_source_kinds(label: &str, text: &str) -> Vec<(String, usize)> {
    let _ = (label, text);
    Vec::new()
}

/// Parse `text` under the grammar `label` through the meta-language engine.
#[cfg(feature = "meta-language")]
#[must_use]
pub fn parse_network(label: &str, text: &str) -> meta_language::LinkNetwork {
    meta_language::LinkNetwork::parse(text, label, meta_language::ParseConfiguration::default())
}

/// Without the engine the inventory is empty and every caller reports zero —
/// the same degraded posture as the other grammar-backed surfaces.
#[cfg(not(feature = "meta-language"))]
#[must_use]
pub fn parse_network(label: &str, text: &str) -> GrammarlessNetwork {
    let _ = (label, text);
    GrammarlessNetwork
}

/// The no-engine stand-in for a parsed network: it holds no links.
#[cfg(not(feature = "meta-language"))]
#[derive(Debug, Clone, Copy)]
pub struct GrammarlessNetwork;

#[cfg(not(feature = "meta-language"))]
impl GrammarlessNetwork {
    /// There are no links to walk.
    #[must_use]
    pub const fn links(&self) -> std::iter::Empty<()> {
        std::iter::empty()
    }
}

/// The syntax-link kind histogram of a parsed network, in kind order.
#[cfg(feature = "meta-language")]
#[must_use]
pub fn kind_histogram(network: &meta_language::LinkNetwork) -> Vec<(String, usize)> {
    let mut histogram: BTreeMap<String, usize> = BTreeMap::new();
    for link in network.links() {
        let metadata = link.metadata();
        if metadata.link_type() == Some(meta_language::LinkType::Syntax) {
            *histogram
                .entry(metadata.term().unwrap_or("?").to_owned())
                .or_default() += 1;
        }
    }
    histogram.into_iter().collect()
}

/// The histogram of named syntax kinds only — the set a projection's
/// coverage table is accountable for.
#[cfg(feature = "meta-language")]
#[must_use]
pub fn named_kind_histogram(network: &meta_language::LinkNetwork) -> Vec<(String, usize)> {
    let mut histogram: BTreeMap<String, usize> = BTreeMap::new();
    for link in network.links() {
        let metadata = link.metadata();
        if metadata.link_type() == Some(meta_language::LinkType::Syntax) && metadata.is_named() {
            *histogram
                .entry(metadata.term().unwrap_or("?").to_owned())
                .or_default() += 1;
        }
    }
    histogram.into_iter().collect()
}

/// The histogram of a network that cannot exist without the engine.
#[cfg(not(feature = "meta-language"))]
#[must_use]
pub fn kind_histogram(network: &GrammarlessNetwork) -> Vec<(String, usize)> {
    let _ = network;
    Vec::new()
}
