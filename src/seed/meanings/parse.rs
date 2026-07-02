//! Seed-file parser for the meaning lexicon (issue #386).
//!
//! Reading the embedded `data/seed/meanings*.lino` sources into the in-memory
//! [`Lexicon`](super::Lexicon) is a self-contained pipeline: it turns Links
//! Notation nodes into [`Meaning`](super::Meaning) records, their
//! [`Lexeme`](super::Lexeme) blocks, [`WordForm`](super::WordForm)s, and the
//! [`SemanticFacet`](super::SemanticFacet)s each form exposes. Splitting it out
//! of [`super`] keeps the loader focused on querying and keeps both files clear
//! of the seed file-size guard (mirroring [`super::super::roles`]).

use super::{Lexeme, Lexicon, Meaning, SemanticFacet, WordForm};
use crate::seed::parser::{decode_codepoints, parse_lino, LinoNode};

pub(super) fn parse_lexicon(text: &str) -> Lexicon {
    let root = parse_lino(text);
    // The lexicon is split across several files (program, units, …), each
    // wrapping its records under a top-level `meanings` node. When the files
    // are concatenated the document therefore holds one-or-more `meanings`
    // containers; collect the records from every one. If none is present the
    // records sit at the document root (kept for robustness).
    let mut meanings = Vec::new();
    let containers: Vec<&LinoNode> = root
        .children
        .iter()
        .filter(|c| c.name == "meanings")
        .collect();
    let sources: Vec<&LinoNode> = if containers.is_empty() {
        vec![&root]
    } else {
        containers
    };
    for container in sources {
        for node in container
            .children
            .iter()
            .filter(|c| c.name == "meaning" || c.name != "meanings")
        {
            meanings.push(parse_meaning(node));
        }
    }
    Lexicon { meanings }
}

fn parse_meaning(node: &LinoNode) -> Meaning {
    let slug = meaning_slug(node);
    let mut defined_by = Vec::new();
    let mut roles = Vec::new();
    let semantic_facets = parse_semantic_facets(node);
    let mut lexemes = Vec::new();
    let mut wikidata = String::new();
    if node.name != "meaning" && !node.id.is_empty() {
        defined_by.extend(definition_targets(&node.id));
    }
    for child in &node.children {
        match child.name.as_str() {
            "defined_by" | "defined-by" => defined_by.extend(definition_targets(&child.id)),
            "grounded-in" | "wikidata" => wikidata.clone_from(&child.id),
            "role" => roles.push(child.id.clone()),
            "lexeme" => {
                let words = child
                    .children
                    .iter()
                    .filter(|w| w.name == "word" || w.name == "surface")
                    .map(|w| parse_word_form(&slug, w))
                    .collect();
                lexemes.push(Lexeme {
                    language: lexeme_language(child),
                    words,
                });
            }
            "surface" => {
                let language = child.find_child_value("language").to_string();
                lexemes.push(Lexeme {
                    language,
                    words: vec![parse_word_form(&slug, child)],
                });
            }
            _ => {}
        }
    }
    Meaning {
        gloss: generated_meaning_description(&slug, &defined_by, node.find_child_value("gloss")),
        slug,
        wiktionary: node.find_child_value("wiktionary").to_string(),
        wikidata,
        defined_by,
        roles,
        semantic_facets,
        lexemes,
    }
}

fn definition_targets(raw: &str) -> impl Iterator<Item = String> + '_ {
    raw.split(|character: char| {
        character.is_whitespace() || matches!(character, '(' | ')' | '[' | ']' | ',')
    })
    .filter(|target| !target.is_empty())
    .map(canonical_definition_target)
}

fn canonical_definition_target(target: &str) -> String {
    match target {
        "reference_action" => String::from("reference-action"),
        "link_action" => String::from("link-action"),
        "any_of_reference" => String::from("any-of-reference"),
        "any_of_link" => String::from("any-of-link"),
        "repeatable_from_zero" => String::from("repeatable-from-zero"),
        "zero_or_more" => String::from("zero-or-more"),
        "point_at" => String::from("point-at"),
        "or_else" => String::from("or-else"),
        "is_identity" => String::from("is-identity"),
        "is_a_kind_of" => String::from("is-a-kind-of"),
        "held_by" => String::from("held-by"),
        "together_with" => String::from("together-with"),
        "self_equation" => String::from("self-equation"),
        "one_symbol_one_meaning" => String::from("one-symbol-one-meaning"),
        "sense_split" => String::from("sense-split"),
        "bank_river" => String::from("bank-river"),
        "bank_money" => String::from("bank-money"),
        other => other.to_string(),
    }
}

fn parse_word_form(parent_meaning: &str, node: &LinoNode) -> WordForm {
    let mut semantic_facets = parse_semantic_facets(node);
    // The seed nesting itself asserts that this literal surface denotes the
    // parent meaning. Expose that as data so consumers do not have to read an
    // authored free-text field to understand the word form.
    ensure_semantic_facet_target(&mut semantic_facets, "notation", "word_surface");
    ensure_semantic_facet_target(&mut semantic_facets, "denotation", parent_meaning);

    WordForm {
        text: surface_text(node),
        description: generated_word_description(parent_meaning, node),
        action: node.find_child_value("action").to_string(),
        semantic_facets,
    }
}

fn ensure_semantic_facet_target(facets: &mut Vec<SemanticFacet>, kind: &str, target: &str) {
    if let Some(facet) = facets.iter_mut().find(|facet| facet.kind == kind) {
        if !facet.meanings.iter().any(|meaning| meaning == target) {
            facet.meanings.push(target.to_string());
        }
        return;
    }

    facets.push(SemanticFacet {
        kind: kind.to_string(),
        meanings: vec![target.to_string()],
    });
}

/// The closed facet vocabulary. A semantic facet is written either as the
/// native `subject predicate` line (`notation word_surface`) or, for backward
/// compatibility, as a `facet <kind>` wrapper. The two forms are equivalent;
/// `scripts/migrate-empty-facet-fields.rs` rewrites the wrapper into the line
/// form so the seed never carries an empty `word_surface:` colon redefinition.
const FACET_KINDS: &[&str] = &[
    "notation",
    "annotation",
    "denotation",
    "connotation",
    "part_of_speech",
    // Issue #538: a surface may pin the grammatical number it lexicalises
    // (`grammatical_number singular` / `grammatical_number plural`), so a word
    // form records whether it is the singular or plural way to express its
    // meaning rather than leaving the seed to guess from the spelling.
    "grammatical_number",
    "self-equation",
];

fn parse_semantic_facets(node: &LinoNode) -> Vec<SemanticFacet> {
    let mut facets: Vec<SemanticFacet> = Vec::new();
    for child in &node.children {
        if child.name == "facet" {
            // Legacy wrapper: `facet <kind>` with nested target children.
            let targets = child.children.iter().filter_map(semantic_facet_target);
            merge_facet_targets(&mut facets, &child.id, targets);
        } else if FACET_KINDS.contains(&child.name.as_str()) && !child.id.is_empty() {
            // Native subject-predicate line: `<kind> <target>`.
            merge_facet_targets(&mut facets, &child.name, std::iter::once(child.id.clone()));
        }
    }
    facets
}

/// Append `targets` under the `kind` facet, creating it if absent and skipping
/// duplicates so the wrapper and line forms collapse to one facet.
fn merge_facet_targets(
    facets: &mut Vec<SemanticFacet>,
    kind: &str,
    targets: impl Iterator<Item = String>,
) {
    let position = facets.iter().position(|facet| facet.kind == kind);
    let index = position.unwrap_or_else(|| {
        facets.push(SemanticFacet {
            kind: kind.to_string(),
            meanings: Vec::new(),
        });
        facets.len() - 1
    });
    for target in targets {
        if !facets[index].meanings.contains(&target) {
            facets[index].meanings.push(target);
        }
    }
}

fn meaning_slug(node: &LinoNode) -> String {
    if node.name == "meaning" {
        node.id.clone()
    } else {
        node.name.clone()
    }
}

fn lexeme_language(node: &LinoNode) -> String {
    let explicit = node.find_child_value("language");
    if explicit.is_empty() {
        node.id.clone()
    } else {
        explicit.to_string()
    }
}

fn surface_text(node: &LinoNode) -> String {
    let text = node.find_child_value("text");
    if !text.is_empty() {
        return text.to_string();
    }
    // Backward compatibility with the historical `codepoints <ints>` encoding.
    let codepoints = node.find_child_value("codepoints");
    if codepoints.is_empty() {
        node.id.clone()
    } else {
        decode_codepoints(codepoints)
    }
}

fn generated_meaning_description(slug: &str, defined_by: &[String], stored: &str) -> String {
    if !stored.is_empty() {
        return stored.to_string();
    }
    if defined_by.is_empty() {
        slug.to_string()
    } else {
        format!("{} defined by {}", slug, defined_by.join(" "))
    }
}

fn generated_word_description(parent_meaning: &str, node: &LinoNode) -> String {
    let stored = node.find_child_value("description");
    if !stored.is_empty() {
        return stored.to_string();
    }
    let surface = surface_text(node);
    if surface.is_empty() {
        parent_meaning.to_string()
    } else {
        format!("{surface} denotes {parent_meaning}")
    }
}

fn semantic_facet_target(node: &LinoNode) -> Option<String> {
    match node.name.as_str() {
        "meaning" | "target" | "facet-target" => Some(node.id.clone()),
        _ if !node.id.is_empty() => Some(node.id.clone()),
        _ if !node.name.is_empty() => Some(node.name.clone()),
        _ => None,
    }
}
