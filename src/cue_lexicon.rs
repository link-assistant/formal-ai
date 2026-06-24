//! Issue #559 (R341): natural-language recognition cues as first-class link data.
//!
//! The meta core's first step turns a message into a problem frame, and part of
//! that is recognizing which handler family a phrase points at. Historically the
//! cue lists that drive that recognition lived as inline Rust string literals in
//! [`crate::intent_formalization`] — a hardcoded list of arithmetic operators, of
//! web-search verbs, of the fourteen text-manipulation operations, and so on. The
//! issue asks to generalize away from hardcoded specific intents; R97/R103 already
//! moved most surface vocabulary into seed data.
//!
//! This module finishes that migration for the meta core's cue lists. It loads
//! `data/meta/cue-lexicon.lino`, where every cue is a reviewable link grouped into
//! a named [`CueSet`] that declares its [`CueMatch`] mode. The Rust code keeps only
//! the structural glue — digit presence, AND/OR composition, which input each set
//! is tested against — and reads the cue strings from here. A specification test
//! grounds every set the code consults to this data and proves routing is unchanged
//! (R13), so the data can never drift from the behaviour it drives.

use crate::seed::parser::parse_lino;
use std::sync::OnceLock;

/// How a cue is compared against the (already normalized) input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CueMatch {
    /// Whitespace-bounded word match for Latin/Cyrillic; substring for CJK
    /// (mirrors `intent_formalization::contains_token` exactly).
    Token,
    /// Raw `contains` substring match.
    Substring,
    /// `starts_with` prefix match.
    Prefix,
}

impl CueMatch {
    /// The stable slug used in the link data.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Token => "token",
            Self::Substring => "substring",
            Self::Prefix => "prefix",
        }
    }

    /// Parse a slug from the data into a match mode.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "token" => Some(Self::Token),
            "substring" => Some(Self::Substring),
            "prefix" => Some(Self::Prefix),
            _ => None,
        }
    }

    /// Whether `cue` matches `haystack` under this mode.
    #[must_use]
    pub fn matches(self, haystack: &str, cue: &str) -> bool {
        match self {
            Self::Token => contains_token(haystack, cue),
            Self::Substring => haystack.contains(cue),
            Self::Prefix => haystack.starts_with(cue),
        }
    }
}

/// One named group of cues that point at a handler family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueSet {
    /// The set name the Rust code looks the set up by.
    pub name: String,
    /// The handler family these cues recognize (documentation/grounding only).
    pub handler: String,
    /// How each cue is compared against the input.
    pub match_mode: CueMatch,
    /// The cue strings themselves, in declaration order.
    pub cues: Vec<String>,
}

impl CueSet {
    /// Whether any cue in this set matches `haystack` under the set's mode.
    #[must_use]
    pub fn matches(&self, haystack: &str) -> bool {
        self.cues
            .iter()
            .any(|cue| self.match_mode.matches(haystack, cue))
    }
}

const CUE_LEXICON_LINO: &str = include_str!("../data/meta/cue-lexicon.lino");

/// The cue-set catalogue, parsed once from the embedded link data.
#[must_use]
pub fn cue_sets() -> &'static [CueSet] {
    static CELL: OnceLock<Vec<CueSet>> = OnceLock::new();
    CELL.get_or_init(load_cue_sets)
}

fn load_cue_sets() -> Vec<CueSet> {
    let tree = parse_lino(CUE_LEXICON_LINO);
    let mut out = Vec::new();
    for record in &tree.children {
        if record.find_child_value("record_type") != "cue_set" {
            continue;
        }
        let name = record.find_child_value("name").to_owned();
        let Some(match_mode) = CueMatch::from_slug(record.find_child_value("match")) else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let cues: Vec<String> = record
            .children
            .iter()
            .filter(|child| child.name == "cue")
            .map(|child| child.id.clone())
            .collect();
        out.push(CueSet {
            name,
            handler: record.find_child_value("handler").to_owned(),
            match_mode,
            cues,
        });
    }
    out
}

/// Look a cue set up by name.
#[must_use]
pub fn cue_set(name: &str) -> Option<&'static CueSet> {
    cue_sets().iter().find(|set| set.name == name)
}

/// Whether the named cue set matches `haystack`. A missing set never matches; the
/// grounding test guarantees every name the code uses is present in the data.
#[must_use]
pub fn matches(set_name: &str, haystack: &str) -> bool {
    cue_set(set_name).is_some_and(|set| set.matches(haystack))
}

/// The cue strings of the named set, for callers that need to compose the match
/// themselves (e.g. the arithmetic check, which tests two inputs per cue). Returns
/// an empty slice when the set is absent.
#[must_use]
pub fn cues(set_name: &str) -> &'static [String] {
    cue_set(set_name).map_or(&[], |set| set.cues.as_slice())
}

/// Whitespace-bounded word match (CJK by substring) — the exact semantics of
/// `intent_formalization::contains_token`, kept here so token-mode cues behave
/// identically to the inline lists they replaced.
fn contains_token(normalized: &str, expected: &str) -> bool {
    if crate::coding::contains_cjk(expected) {
        return normalized.contains(expected);
    }
    normalized.split_whitespace().any(|token| token == expected)
}
