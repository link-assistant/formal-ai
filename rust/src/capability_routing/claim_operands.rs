//! Claim operands (issue #1175 R3, the last five rows).
//!
//! `docs_method_explanation`, `kupi_slona`, `source_conflict`,
//! `github_repository_traffic` and `formalization_request` used to read
//! nothing but their cue. Each now reads the operand its answer is about: the
//! seeded documentation page whose project and method the prompt names, the
//! circular idiom as the whole utterance, the two alternatives a conflict
//! attributes to sources, the repository a traffic question names, and the
//! statement a formalization carries beyond its cue.
//!
//! A reader returns the operands it extracts, an empty list when there are
//! none. The claim row admits on that list being non-empty, and the rule
//! interpreter captures the same operands (`value <name> operand <kind>`), so
//! the row and the answer read one thing. Every surface is a seed role.
//!
//! Mirrored by `CLAIM_OPERANDS` in
//! `js/worker/formal_ai_worker_claim_operands.js`.

use std::collections::BTreeSet;

/// Marks that separate words besides whitespace (JS `CLAIM_OPERAND_WORD_MARKS`).
const WORD_MARKS: &str = ",.;:!?\"'«»()[]{}<>`—–-…，。；：！？、।“”‘’";
/// Marks that end a clause; a full stop ends one only before whitespace.
const CLAUSE_MARKS: &str = ",;，；。!?！？\n।";
/// The roles whose words carry no operand of their own.
const FRAME_ROLES: [&str; 3] = [
    "frame_filler_word",
    crate::seed::ROLE_REQUEST_FUNCTION_WORD,
    crate::seed::ROLE_STATEMENT_FUNCTION_WORD,
];

/// The operands of `kind` the prompt carries; `None` for a kind this module
/// does not read.
#[must_use]
pub fn claim_operands(kind: &str, prompt: &str) -> Option<Vec<String>> {
    Some(match kind {
        "documented_method" => documented_method(prompt),
        "idiom_utterance" => idiom_utterance(prompt).into_iter().collect(),
        "attributed_alternatives" => attributed_alternatives(prompt),
        "named_repository" => named_repository(prompt).into_iter().collect(),
        "formalization_statement" => crate::solver_handlers::formalization_statement(prompt)
            .into_iter()
            .collect(),
        _ => return None,
    })
}

/// Whether a surface is written in a script without spaces between words.
fn unspaced(surface: &str) -> bool {
    surface.chars().any(is_unspaced_script)
}

/// Whether `ch` belongs to a script written without spaces (kana, CJK, hangul).
const fn is_unspaced_script(ch: char) -> bool {
    matches!(
        ch,
        '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' | '\u{ac00}'..='\u{d7af}'
    )
}

/// The lowercased surfaces of a seed role.
fn role_surfaces(role: &str) -> Vec<String> {
    crate::seed::lexicon()
        .words_for_role(role)
        .iter()
        .map(|surface| surface.to_lowercase())
        .filter(|surface| !surface.is_empty())
        .collect()
}

/// Whether `ch` separates words; `None` is the edge of the text.
fn separates(ch: Option<char>) -> bool {
    ch.is_none_or(|ch| ch.is_whitespace() || WORD_MARKS.contains(ch))
}

/// The words of `text`: runs between whitespace and word marks.
fn words(text: &str) -> Vec<String> {
    text.split(|ch: char| separates(Some(ch)))
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether `surface` occurs in `lower`: as a substring for an unspaced
/// script, between separators otherwise.
fn names_surface(lower: &str, surface: &str) -> bool {
    if unspaced(surface) {
        return lower.contains(surface);
    }
    let chars: Vec<char> = lower.chars().collect();
    let wanted: Vec<char> = surface.chars().collect();
    (0..=chars.len().saturating_sub(wanted.len())).any(|start| {
        chars.get(start..start + wanted.len()) == Some(wanted.as_slice())
            && separates(
                start
                    .checked_sub(1)
                    .and_then(|before| chars.get(before).copied()),
            )
            && separates(chars.get(start + wanted.len()).copied())
    })
}

/// `text`, lowercased, with every surface removed, longest first.
#[must_use]
pub fn without_surfaces(text: &str, surfaces: &[String]) -> String {
    let mut ordered: Vec<String> = surfaces
        .iter()
        .map(|surface| surface.to_lowercase())
        .collect();
    ordered.sort_by_key(|surface| core::cmp::Reverse(surface.chars().count()));
    ordered
        .iter()
        .filter(|surface| !surface.is_empty())
        .fold(text.to_lowercase(), |rest, surface| {
            rest.replace(surface.as_str(), " ")
        })
}

/// Whether every word of `text` is a word of the frame roles (fillers and
/// function words); the surfaces of unspaced scripts are removed as
/// substrings first, longest first.
#[must_use]
pub fn only_frame_words(text: &str) -> bool {
    let mut covered = BTreeSet::new();
    let mut unspaced_surfaces = Vec::new();
    for surface in FRAME_ROLES.iter().copied().flat_map(role_surfaces) {
        if unspaced(&surface) {
            unspaced_surfaces.push(surface);
        } else {
            covered.extend(words(&surface));
        }
    }
    let rest = without_surfaces(text, &unspaced_surfaces);
    words(&rest).iter().all(|word| covered.contains(word))
}

/// The seeded documentation page whose project and method the prompt names,
/// as `[page, project, docs url]`.
///
/// The page is the `policy docs_method_explanation` block of
/// `data/seed/handler-rules.lino`; the prompt names its project and method as
/// identifiers, and its class (or the class alias, or a method noun).
fn documented_method(prompt: &str) -> Vec<String> {
    let policy = |key| crate::rule_interpreter::handler_policy("docs_method_explanation", key);
    let (Some(page), Some(docs_url)) = (policy("page"), policy("docs_url")) else {
        return Vec::new();
    };
    let alias = policy("class_alias").unwrap_or_default().to_lowercase();
    let segments: Vec<&str> = page.split('.').collect();
    let parts: Vec<String> = segments
        .iter()
        .map(|segment| segment.to_lowercase())
        .collect();
    let (Some(project), Some(method)) = (parts.first(), parts.last()) else {
        return Vec::new();
    };
    if parts.len() < 3 {
        return Vec::new();
    }
    let lower = prompt.to_lowercase();
    let identifiers: Vec<&str> = lower
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|word| !word.is_empty())
        .collect();
    let named = |word: &str| identifiers.contains(&word);
    let class_named = parts[1..parts.len() - 1]
        .iter()
        .any(|class| named(class.as_str()))
        || (!alias.is_empty() && named(alias.as_str()))
        || crate::seed::lexicon().mentions_role("code_method_noun", &lower);
    if named(project.as_str()) && named(method.as_str()) && class_named {
        vec![page.clone(), segments[0].to_owned(), docs_url]
    } else {
        Vec::new()
    }
}

/// The circular idiom when it is the whole utterance, framed only by frame
/// words ("Ну купи слона, пожалуйста"), never a phrase inside another request.
fn idiom_utterance(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    role_surfaces("circular_joke_phrase")
        .into_iter()
        .find(|surface| {
            lower.find(surface.as_str()).is_some_and(|at| {
                only_frame_words(&format!(
                    "{} {}",
                    &lower[..at],
                    &lower[at + surface.len()..]
                ))
            })
        })
}

/// The clauses of `text`, split at clause marks and seeded clause joiners,
/// trimmed of whitespace and word marks.
fn clauses(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let lower: Vec<char> = chars
        .iter()
        .map(|ch| ch.to_lowercase().next().unwrap_or(*ch))
        .collect();
    let mut joiners: Vec<Vec<char>> = role_surfaces("clause_joiner")
        .iter()
        .map(|surface| surface.chars().collect())
        .collect();
    joiners.sort_by_key(|joiner| core::cmp::Reverse(joiner.len()));
    let mut pieces = Vec::new();
    let (mut start, mut index) = (0, 0);
    while index < chars.len() {
        let ch = chars[index];
        if CLAUSE_MARKS.contains(ch) || (ch == '.' && separates(chars.get(index + 1).copied())) {
            pieces.push(chars[start..index].iter().collect::<String>());
            index += 1;
            start = index;
            continue;
        }
        let joiner = joiners.iter().find(|joiner| {
            lower.get(index..index + joiner.len()) == Some(joiner.as_slice())
                && (unspaced(&joiner.iter().collect::<String>())
                    || (separates(index.checked_sub(1).map(|before| chars[before]))
                        && separates(chars.get(index + joiner.len()).copied())))
        });
        if let Some(joiner) = joiner {
            pieces.push(chars[start..index].iter().collect::<String>());
            index += joiner.len();
            start = index;
        } else {
            index += 1;
        }
    }
    pieces.push(chars[start..].iter().collect::<String>());
    pieces
        .iter()
        .map(|piece| {
            piece
                .trim_matches(|ch: char| ch.is_whitespace() || WORD_MARKS.contains(ch))
                .to_owned()
        })
        .filter(|piece| !piece.is_empty())
        .collect()
}

/// The text after the first colon when it is not empty, else the prompt.
fn after_colon(prompt: &str) -> &str {
    prompt
        .split_once([':', '：'])
        .map(|(_, after)| after.trim())
        .filter(|after| !after.is_empty())
        .unwrap_or(prompt)
}

/// The first two clauses that attribute what they state to a source
/// ("Wikipedia says 1880", "по данным Британники в 1881").
fn attributed_alternatives(prompt: &str) -> Vec<String> {
    let markers = role_surfaces("source_attribution_marker");
    let attributed: Vec<String> = clauses(after_colon(prompt))
        .into_iter()
        .filter(|clause| {
            let lower = clause.to_lowercase();
            markers.iter().any(|marker| {
                names_surface(&lower, marker) && clause.chars().count() > marker.chars().count() + 1
            })
        })
        .collect();
    if attributed.len() >= 2 {
        attributed.into_iter().take(2).collect()
    } else {
        Vec::new()
    }
}

/// The repository a traffic question names: an owner/name slug whose
/// segments are not cue or function words, else the assistant's own
/// repository when a second-person possessive names it ("твое репо").
fn named_repository(prompt: &str) -> Option<String> {
    let excluded: BTreeSet<String> = FRAME_ROLES
        .iter()
        .copied()
        .chain([
            "github_repository_traffic_signal",
            "repository_reference",
            "github_repository_platform",
        ])
        .flat_map(role_surfaces)
        .flat_map(|surface| words(&surface))
        .collect();
    let slug = crate::solver_handlers::repository_slug_candidates(prompt)
        .into_iter()
        .find(|slug| {
            slug.split('/')
                .all(|segment| !excluded.contains(&segment.to_lowercase()))
        });
    if slug.is_some() {
        return slug;
    }
    let lower = prompt.to_lowercase();
    let own = crate::seed::agent_info()
        .get("repository")
        .map(|repository| repository.trim().to_owned())
        .filter(|repository| !repository.is_empty())?;
    role_surfaces("second_person_possessive")
        .iter()
        .any(|surface| names_surface(&lower, surface))
        .then_some(own)
}
