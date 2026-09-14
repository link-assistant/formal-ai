//! Resolve a requirement that names behaviour or a declaration, not a file
//! (issue #1085 D2.2).
//!
//! The self-AST census is the links network over the workspace's own source.
//! A requirement such as "add wikiquote as a web search provider" is resolved
//! by querying it for the `const` or `static` whose identifier words the
//! requirement contains, or for the identifier the requirement names, so the
//! prompt never has to name `src/web_search_core.rs`.

use std::cmp::Ordering;

use crate::self_ast_census::{ModuleCensus, SymbolSpan, WorkspaceCensus, workspace};

/// Where a requirement lands: the module and the declaration it changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequirementTarget {
    pub module_path: String,
    pub symbol: String,
    pub kind: String,
}

/// Resolve `requirement` against the live workspace census.
#[must_use]
pub fn resolve_requirement_target(requirement: &str) -> Option<RequirementTarget> {
    resolve_in(workspace(), requirement)
}

/// Resolve `requirement` against `census`.
///
/// An identifier the requirement names verbatim wins; otherwise the `const` or
/// `static` whose identifier words (singularised) all occur in the requirement
/// wins, longest identifier first. A tie between modules is broken by the
/// module path words the requirement mentions; a remaining tie is ambiguity and
/// resolves to nothing rather than to a guess.
#[must_use]
pub fn resolve_in(census: &WorkspaceCensus, requirement: &str) -> Option<RequirementTarget> {
    let tokens = tokens_of(requirement);
    for token in &tokens {
        if !looks_like_declared_name(token) {
            continue;
        }
        let named: Vec<(&ModuleCensus, &SymbolSpan)> = census
            .modules_declaring(token)
            .into_iter()
            .filter_map(|module| module.symbol(token).map(|symbol| (module, symbol)))
            .collect();
        if !named.is_empty() {
            return unique(&named, &tokens);
        }
    }

    let words: Vec<String> = tokens
        .iter()
        .map(|token| singular(&token.to_lowercase()))
        .collect();
    let mut best: Vec<(&ModuleCensus, &SymbolSpan)> = Vec::new();
    let mut best_length = 0;
    for module in &census.modules {
        for symbol in &module.symbols {
            if symbol.kind != "const" && symbol.kind != "static" {
                continue;
            }
            let parts: Vec<String> = symbol
                .name
                .split('_')
                .filter(|part| !part.is_empty())
                .map(|part| singular(&part.to_lowercase()))
                .collect();
            if parts.len() < 2 || !parts.iter().all(|part| words.contains(part)) {
                continue;
            }
            match parts.len().cmp(&best_length) {
                Ordering::Greater => {
                    best_length = parts.len();
                    best = vec![(module, symbol)];
                }
                Ordering::Equal => best.push((module, symbol)),
                Ordering::Less => {}
            }
        }
    }
    if best.is_empty() {
        return None;
    }
    unique(&best, &tokens)
}

fn unique(
    candidates: &[(&ModuleCensus, &SymbolSpan)],
    tokens: &[String],
) -> Option<RequirementTarget> {
    if let [(module, symbol)] = candidates {
        return Some(target(module, symbol));
    }
    let words: Vec<String> = tokens
        .iter()
        .map(|token| singular(&token.to_lowercase()))
        .collect();
    let scored: Vec<(usize, &ModuleCensus, &SymbolSpan)> = candidates
        .iter()
        .map(|(module, symbol)| {
            let score = path_words(&module.path)
                .iter()
                .filter(|word| words.contains(word))
                .count();
            (score, *module, *symbol)
        })
        .collect();
    let top = scored.iter().map(|(score, _, _)| *score).max()?;
    let mut winners = scored
        .iter()
        .filter(|(score, _, _)| *score == top)
        .map(|(_, module, symbol)| (*module, *symbol));
    let (module, symbol) = winners.next()?;
    if winners.next().is_some() {
        return None;
    }
    Some(target(module, symbol))
}

fn target(module: &ModuleCensus, symbol: &SymbolSpan) -> RequirementTarget {
    RequirementTarget {
        module_path: module.path.clone(),
        symbol: symbol.name.clone(),
        kind: symbol.kind.clone(),
    }
}

fn path_words(path: &str) -> Vec<String> {
    path.split(['/', '_', '.'])
        .filter(|part| !part.is_empty() && *part != "src" && *part != "rs" && *part != "mod")
        .map(|part| singular(&part.to_lowercase()))
        .collect()
}

fn tokens_of(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn looks_like_declared_name(token: &str) -> bool {
    token.len() >= 2
        && token
            .chars()
            .any(|character| character.is_ascii_uppercase())
        && token.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
}

fn singular(word: &str) -> String {
    if word.len() > 3 && word.ends_with('s') && !word.ends_with("ss") {
        word[..word.len() - 1].to_owned()
    } else {
        word.to_owned()
    }
}
