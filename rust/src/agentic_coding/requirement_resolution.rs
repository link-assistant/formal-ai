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
/// resolves to nothing rather than to a guess. When identifier words do not bind,
/// an actual canonical seed surface and its semantic role can bind its declared owner.
#[must_use]
pub fn resolve_in(census: &WorkspaceCensus, requirement: &str) -> Option<RequirementTarget> {
    let modules = explicit_module_scope(census, requirement)?;
    if modules.is_empty() {
        resolve_scoped(census, requirement)
    } else {
        let scoped = WorkspaceCensus { modules };
        resolve_scoped(&scoped, requirement)
            .or_else(|| resolve_scoped_literal(&scoped, requirement))
    }
}

/// Resolve an unnamed scalar initializer only within an explicit source scope.
fn resolve_scoped_literal(
    census: &WorkspaceCensus,
    requirement: &str,
) -> Option<RequirementTarget> {
    let (_, old, _) = super::write_request::compose_edit_clauses(requirement)?.edit;
    let module = census.modules.first()?;
    let candidates: Vec<_> = module
        .symbols
        .iter()
        .filter(|symbol| {
            if symbol.kind != "const" && symbol.kind != "static" {
                return false;
            }
            let declaration: String = module
                .source()
                .split_inclusive('\n')
                .skip(symbol.start_line.saturating_sub(1))
                .take(symbol.end_line.saturating_sub(symbol.start_line) + 1)
                .collect();
            let Some((_, initializer)) = declaration.split_once('=') else {
                return false;
            };
            let initializer = initializer.trim();
            let segments = crate::normal_markov::quoted_segment_spans(initializer);
            let [segment] = segments.as_slice() else {
                return false;
            };
            initializer.starts_with('"')
                && segment.start == 0
                && initializer[segment.end..].trim() == ";"
                && segment.text == old
        })
        .collect();
    if let [symbol] = candidates.as_slice() {
        Some(target(module, symbol))
    } else {
        None
    }
}

/// Bind exact observed module paths before declaration ranking.
fn explicit_module_scope(census: &WorkspaceCensus, requirement: &str) -> Option<Vec<ModuleCensus>> {
    let references: Vec<_> = requirement
        .split(|character: char| {
            !character.is_alphanumeric() && !matches!(character, '_' | '.' | '/' | '-')
        })
        .map(|token| token.trim_end_matches('.'))
        .filter(|token| {
            token.contains('/')
                && std::path::Path::new(token)
                    .extension()
                    .is_some_and(|extension| extension == "rs")
        })
        .collect();
    let mut selected = Vec::new();
    for reference in references {
        let relative = reference.strip_prefix("./").unwrap_or(reference);
        let path = relative.strip_prefix("rust/").unwrap_or(relative);
        if path.split('/').any(|part| matches!(part, "." | "..")) {
            return None;
        }
        let module = census.module(path)?;
        if selected
            .iter()
            .all(|candidate: &ModuleCensus| candidate.path != module.path)
        {
            selected.push(module.clone());
        }
    }
    (selected.len() <= 1).then_some(selected)
}

/// Rank declarations only within the observed module scope.
fn resolve_scoped(census: &WorkspaceCensus, requirement: &str) -> Option<RequirementTarget> {
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
        let sources: Vec<_> = crate::seed::seed_files()
            .into_iter()
            .filter(|(_, text)| crate::seed::MEANING_FILES.contains(text))
            .collect();
        return resolve_seed_target(census, requirement, &sources);
    }
    unique(&best, &tokens)
}

/// Bind a concrete canonical surface and semantic role to a declared seed constant.
///
/// Constant spelling follows the seed registry generator. No seed is privileged;
/// equally supported distinct targets remain unresolved.
#[must_use]
pub fn resolve_seed_target(
    census: &WorkspaceCensus,
    requirement: &str,
    sources: &[(&str, &str)],
) -> Option<RequirementTarget> {
    let tokens: Vec<_> = tokens_of(requirement)
        .into_iter()
        .map(|word| word.to_lowercase())
        .collect();
    let role_words: Vec<_> = tokens.iter().map(|word| singular(word)).collect();
    let mut best_length = 0;
    let mut best: Vec<RequirementTarget> = Vec::new();
    for (path, source) in sources {
        let Some(stem) = path
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".lino"))
        else {
            continue;
        };
        if stem.is_empty()
            || !stem.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
            })
        {
            continue;
        }
        let name = format!("{}_LINO", stem.to_ascii_uppercase().replace('-', "_"));
        let candidates: Vec<_> = census
            .modules_declaring(&name)
            .into_iter()
            .filter_map(|module| module.symbol(&name).map(|symbol| (module, symbol)))
            .filter(|(_, symbol)| matches!(symbol.kind.as_str(), "const" | "static"))
            .collect();
        if candidates.is_empty() {
            continue;
        }
        for meaning in crate::seed::parse_lexicon_text(source).meanings {
            if !meaning.roles.iter().any(|role| {
                role.split(['_', '-'])
                    .any(|part| role_words.contains(&singular(&part.to_lowercase())))
            }) {
                continue;
            }
            for surface in meaning.words() {
                if surface.contains('…') {
                    continue;
                }
                let parts: Vec<_> = tokens_of(surface)
                    .into_iter()
                    .map(|word| word.to_lowercase())
                    .collect();
                if parts.is_empty() || !tokens.windows(parts.len()).any(|window| window == parts) {
                    continue;
                }
                if parts.len() > best_length {
                    best_length = parts.len();
                    best.clear();
                }
                if parts.len() == best_length {
                    for (module, symbol) in &candidates {
                        let candidate = target(module, symbol);
                        if !best.contains(&candidate) {
                            best.push(candidate);
                        }
                    }
                }
            }
        }
    }
    if best.len() == 1 { best.pop() } else { None }
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
