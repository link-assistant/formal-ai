//! Resolving the files a requirement names, without ever guessing (#1138 B7).
//!
//! `need` is the plan 00 §4.1 record: its `subject` is the requirement text as
//! written and its `language` is one of en/ru/hi/zh/es, so five held-out prompts
//! are one code path rather than five. Ambiguity resolves to nothing — a guess
//! is never returned.
//!
//! Wave T lands the shapes only; wave I7 leaf 03-L6 fills the bodies in.

use super::{RepositoryWorkspace, WorkspaceError};
use crate::agentic_coding::requirement_resolution;
use crate::needs::Need;
use crate::self_ast_census::WorkspaceCensus;
use std::collections::BTreeSet;

/// One file (and optionally one declaration) a requirement names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// Path relative to the workspace root.
    pub relative_path: String,
    /// The declaration inside it, when one was resolved.
    pub symbol: Option<String>,
    /// Which mechanism found it, for the honesty trace.
    pub how: LocationEvidence,
}

/// Which mechanism resolved a [`Location`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationEvidence {
    /// The self-AST census resolved a declaration (Rust trees).
    Census,
    /// A literal named in the requirement occurs in exactly one file.
    LiteralOccurrence,
    /// A path named verbatim in the requirement exists in the tree.
    NamedPath,
}

/// The candidates a requirement matched when it matched more than one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AmbiguityReport {
    /// Every candidate considered, named rather than silently dropped.
    pub candidates: Vec<Location>,
}

/// Resolve the files `need` names inside `workspace`.
///
/// Rust trees go through the census; every other language goes through a
/// deterministic literal/path scan. Ambiguity resolves to `Vec::new()`.
///
/// # Errors
/// Propagates the workspace's own read failures.
pub fn locate_targets(
    workspace: &RepositoryWorkspace,
    need: &Need,
) -> Result<Vec<Location>, WorkspaceError> {
    let candidates = locate_candidates(workspace, need)?;
    if candidates.named_paths {
        return Ok(candidates.locations);
    }
    Ok(if candidates.locations.len() == 1 {
        candidates.locations
    } else {
        Vec::default()
    })
}

/// The candidates an ambiguous requirement matched, so the protocol can report
/// them instead of guessing one.
///
/// # Errors
/// Propagates the workspace's own read failures.
pub fn locate_ambiguity(
    workspace: &RepositoryWorkspace,
    need: &Need,
) -> Result<AmbiguityReport, WorkspaceError> {
    let candidates = locate_candidates(workspace, need)?;
    Ok(AmbiguityReport {
        candidates: if candidates.locations.len() > 1 {
            candidates.locations
        } else {
            Vec::default()
        },
    })
}

struct CandidateSet {
    locations: Vec<Location>,
    named_paths: bool,
}

fn locate_candidates(
    workspace: &RepositoryWorkspace,
    need: &Need,
) -> Result<CandidateSet, WorkspaceError> {
    let files = workspace.source_files()?;
    let named = named_paths(&files, &need.subject);
    if !named.is_empty() {
        return Ok(CandidateSet {
            locations: named,
            named_paths: true,
        });
    }

    let semantic = semantic_terms(&need.subject, &need.language);
    let mut census = WorkspaceCensus::of_directory(workspace.root()).map_err(|error| {
        WorkspaceError::Observed {
            detail: super::render_protocol_template(
                "workspace_census_error",
                &[("error", &error.to_string())],
            )
            .unwrap_or_else(|| String::from("workspace_census_error")),
        }
    })?;
    // Prefer live package source roots over checked-in transcripts, examples,
    // snapshots, and source fixtures that may contain old copies of the same
    // declaration. This convention is discovered from the path (`src` is a
    // component), so it works for both a root crate and `crates/*/src`
    // workspaces. A non-conventional repository still falls back to every Rust
    // file rather than becoming unsearchable.
    if census
        .modules
        .iter()
        .any(|module| module.path.split('/').any(|part| part == "src"))
    {
        census
            .modules
            .retain(|module| module.path.split('/').any(|part| part == "src"));
    }
    if !census.modules.is_empty() {
        let enriched = semantic.iter().cloned().collect::<Vec<_>>().join(" ");
        if let Some(target) = requirement_resolution::resolve_in(&census, &enriched) {
            return Ok(CandidateSet {
                locations: vec![Location {
                    relative_path: target.module_path,
                    symbol: Some(target.symbol),
                    how: LocationEvidence::Census,
                }],
                named_paths: false,
            });
        }
        let locations = census_candidates(&census, &files, &semantic);
        if !locations.is_empty() {
            return Ok(CandidateSet {
                locations,
                named_paths: false,
            });
        }
    }

    Ok(CandidateSet {
        locations: literal_declarations(&files, &semantic),
        named_paths: false,
    })
}

fn named_paths(files: &[(String, String)], requirement: &str) -> Vec<Location> {
    let lowered = requirement.to_lowercase();
    files
        .iter()
        .filter(|(path, _)| {
            let path = path.to_lowercase();
            (path.contains('/') || path.contains('.')) && lowered.contains(&path)
        })
        .map(|(path, _)| Location {
            relative_path: path.clone(),
            symbol: None,
            how: LocationEvidence::NamedPath,
        })
        .collect()
}

/// Project every evidenced seed meaning into its language-independent id and
/// English surfaces. The resolver therefore consumes concepts rather than a
/// table of translations owned by this module: teaching the lexicon another
/// language immediately teaches repository location the same vocabulary.
fn semantic_terms(requirement: &str, language: &str) -> BTreeSet<String> {
    let normalized = crate::engine::normalize_prompt(requirement);
    let mut terms = words(&normalized);
    let languages = if language == "en" {
        vec!["en"]
    } else {
        vec![language, "en"]
    };
    for meaning in
        crate::seed::lexicon().meanings_with_role(crate::seed::ROLE_CODING_SEARCH_SUBJECT_KIND)
    {
        if !meaning.mentions_in_languages_raw(&normalized, &languages) {
            continue;
        }
        terms.extend(words(&meaning.slug.replace('_', " ")));
        if let Some(surface) = meaning.word_in("en") {
            terms.extend(words(surface));
        }
    }
    terms
}

fn words(text: &str) -> BTreeSet<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.chars().count() > 1)
        .map(|word| {
            let lowered = word.to_lowercase();
            if lowered.len() > 3 && lowered.ends_with('s') && !lowered.ends_with("ss") {
                lowered[..lowered.len() - 1].to_owned()
            } else {
                lowered
            }
        })
        .collect()
}

fn census_candidates(
    census: &WorkspaceCensus,
    files: &[(String, String)],
    terms: &BTreeSet<String>,
) -> Vec<Location> {
    let mut candidates: Vec<CensusCandidate> = Vec::new();
    for module in &census.modules {
        for symbol in &module.symbols {
            if symbol.kind != "const" && symbol.kind != "static" {
                continue;
            }
            let parts = words(&symbol.name);
            let matched = parts.iter().filter(|part| terms.contains(*part)).count();
            if parts.len() < 2 || matched < 2 {
                continue;
            }
            let source = files
                .iter()
                .find(|(path, _)| path == &module.path)
                .map_or("", |(_, source)| source.as_str());
            let path_matches = words(&module.path)
                .iter()
                .filter(|part| terms.contains(*part))
                .count();
            candidates.push(CensusCandidate {
                matched,
                parts: parts.len(),
                path_matches,
                owns_value: owns_value(source, symbol.start_line, symbol.end_line),
                location: Location {
                    relative_path: module.path.clone(),
                    symbol: Some(symbol.name.clone()),
                    how: LocationEvidence::Census,
                },
            });
        }
    }
    let Some(best) = candidates.iter().max_by(|left, right| left.compare(right)) else {
        return Vec::new();
    };
    let best = best.clone();
    candidates
        .into_iter()
        .filter(|candidate| candidate.compare(&best).is_eq())
        .map(|candidate| candidate.location)
        .collect()
}

#[derive(Clone)]
struct CensusCandidate {
    matched: usize,
    parts: usize,
    path_matches: usize,
    owns_value: bool,
    location: Location,
}

impl CensusCandidate {
    /// Rank by identifier coverage first, then by the amount of evidence,
    /// relevant path words and ownership of a value. Equal evidence remains an
    /// ambiguity; this comparator never uses path order as a hidden tiebreaker.
    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        (self.matched * other.parts)
            .cmp(&(other.matched * self.parts))
            .then_with(|| self.matched.cmp(&other.matched))
            .then_with(|| self.path_matches.cmp(&other.path_matches))
            .then_with(|| self.owns_value.cmp(&other.owns_value))
    }
}

fn owns_value(source: &str, start_line: usize, end_line: usize) -> bool {
    let declaration = source
        .lines()
        .skip(start_line.saturating_sub(1))
        .take(end_line.saturating_sub(start_line) + 1)
        .collect::<Vec<_>>()
        .join(" ");
    let Some((_, value)) = declaration.split_once('=') else {
        return false;
    };
    let value = value.trim().trim_end_matches(';').trim();
    value.contains('[')
        || value.contains('{')
        || value.contains('(')
        || value.starts_with(['"', '\'', '&'])
        || value.chars().any(|character| character.is_ascii_digit())
}

fn literal_declarations(files: &[(String, String)], terms: &BTreeSet<String>) -> Vec<Location> {
    let mut locations = Vec::new();
    for (path, source) in files {
        for line in source.lines() {
            let Some((left, _)) = line.split_once('=') else {
                continue;
            };
            for identifier in
                left.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            {
                if !identifier.contains('_')
                    || !identifier.chars().all(|character| {
                        character.is_ascii_uppercase()
                            || character.is_ascii_digit()
                            || character == '_'
                    })
                {
                    continue;
                }
                let parts = words(identifier);
                if parts.len() >= 2 && parts.iter().all(|part| terms.contains(part)) {
                    let location = Location {
                        relative_path: path.clone(),
                        symbol: Some(identifier.to_owned()),
                        how: LocationEvidence::LiteralOccurrence,
                    };
                    if !locations.contains(&location) {
                        locations.push(location);
                    }
                }
            }
        }
    }
    locations
}
