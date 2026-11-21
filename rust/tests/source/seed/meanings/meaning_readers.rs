use super::*;

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

pub(super) fn parse_meaning(node: &LinoNode) -> Meaning {
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
