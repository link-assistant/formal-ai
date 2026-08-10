//! Browser adapter for seed-defined natural/formal statement projections.

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::seed_parser::{parse_lino, LinoNode};
use crate::{decode_uri_component, push_json_string};

const SUBJECT_SLOT: &str = "{subject}";
const PREDICATE_SLOT: &str = "{predicate}";
const OBJECT_SLOT: &str = "{object}";

#[derive(Clone)]
struct Meaning {
    id: String,
    roles: Vec<String>,
    lexemes: Vec<(String, Vec<String>)>,
}

struct FormalProjection {
    slug: String,
    aliases: Vec<String>,
    statement: String,
}

struct NaturalProjection {
    language: String,
    statement: String,
    relations: Vec<(String, String)>,
}

struct Catalog {
    formal: Vec<FormalProjection>,
    natural: Vec<NaturalProjection>,
}

struct Statement {
    subject: String,
    predicate: String,
    object: String,
}

/// Translate six URI-encoded lines: surface, normalized request, natural source,
/// natural target, projection seed, and the Wikidata meaning seed.
pub(crate) fn answer(payload: &str) -> String {
    let decoded = payload
        .lines()
        .map(decode_uri_component)
        .collect::<Vec<_>>();
    if decoded.len() != 6 {
        return String::new();
    }
    let catalog = parse_catalog(&decoded[4]);
    let meanings = parse_meanings(&decoded[5]);
    let Some(formal) = catalog.formal.iter().find(|projection| {
        projection
            .aliases
            .iter()
            .any(|alias| surface_present(&decoded[1], alias))
    }) else {
        return String::new();
    };
    let (statement, source, target) = if decoded[3].is_empty() {
        let Some(natural) = catalog
            .natural
            .iter()
            .find(|projection| projection.language == decoded[2])
        else {
            return String::new();
        };
        let Some(statement) = parse_natural(&decoded[0], natural, &meanings) else {
            return String::new();
        };
        (statement, decoded[2].as_str(), formal.slug.as_str())
    } else {
        let Some(statement) = parse_formal(&decoded[0], formal, &meanings) else {
            return String::new();
        };
        (statement, formal.slug.as_str(), decoded[3].as_str())
    };
    let surface = if target == formal.slug {
        render_template(
            &formal.statement,
            &statement.subject,
            &statement.predicate,
            &statement.object,
        )
    } else {
        let Some(natural) = catalog
            .natural
            .iter()
            .find(|projection| projection.language == target)
        else {
            return String::new();
        };
        let Some(surface) = render_natural(&statement, natural, &meanings) else {
            return String::new();
        };
        surface
    };
    serialize_answer(source, target, &surface, &statement)
}

fn parse_catalog(raw: &str) -> Catalog {
    let tree = parse_lino(raw);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "formal_language_projections");
    let mut formal = Vec::new();
    let mut natural = Vec::new();
    for node in root.into_iter().flat_map(|node| &node.children) {
        if node.name == "formal_language" {
            formal.push(FormalProjection {
                slug: node.id.clone(),
                aliases: children(node, "alias"),
                statement: node.find_child_value("statement").to_owned(),
            });
        } else if node.name == "natural_language" {
            natural.push(NaturalProjection {
                language: node.id.clone(),
                statement: node.find_child_value("statement").to_owned(),
                relations: node
                    .children
                    .iter()
                    .filter(|child| child.name == "relation")
                    .map(|relation| {
                        (
                            relation.id.clone(),
                            relation.find_child_value("predicate").to_owned(),
                        )
                    })
                    .collect(),
            });
        }
    }
    Catalog { formal, natural }
}

fn parse_meanings(raw: &str) -> Vec<Meaning> {
    let tree = parse_lino(raw);
    let Some(root) = tree.children.iter().find(|node| node.name == "meanings") else {
        return Vec::new();
    };
    root.children
        .iter()
        .map(|node| Meaning {
            id: node.find_child_value("grounded-in").to_owned(),
            roles: children(node, "role"),
            lexemes: node
                .children
                .iter()
                .filter(|child| child.name == "lexeme")
                .map(|lexeme| {
                    (
                        lexeme.id.clone(),
                        lexeme
                            .children
                            .iter()
                            .filter(|child| child.name == "surface")
                            .map(|surface| surface.find_child_value("text").to_owned())
                            .collect(),
                    )
                })
                .collect(),
        })
        .collect()
}

fn children(node: &LinoNode, name: &str) -> Vec<String> {
    node.children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.clone())
        .collect()
}

fn parse_natural(
    surface: &str,
    projection: &NaturalProjection,
    meanings: &[Meaning],
) -> Option<Statement> {
    for (predicate_id, predicate) in &projection.relations {
        if meaning_with_role(meanings, predicate_id, "binary_relation_property").is_none() {
            continue;
        }
        for subject in meanings
            .iter()
            .filter(|meaning| has_role(meaning, "wikidata_entity_anchor"))
        {
            for subject_surface in words(subject, &projection.language) {
                for object in meanings
                    .iter()
                    .filter(|meaning| has_role(meaning, "wikidata_entity_anchor"))
                {
                    for object_surface in words(object, &projection.language) {
                        let candidate = render_template(
                            &projection.statement,
                            subject_surface,
                            predicate,
                            object_surface,
                        );
                        if normalize_statement(&candidate) == normalize_statement(surface) {
                            return Some(Statement {
                                subject: subject.id.clone(),
                                predicate: predicate_id.clone(),
                                object: object.id.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
    None
}

fn parse_formal(
    surface: &str,
    projection: &FormalProjection,
    meanings: &[Meaning],
) -> Option<Statement> {
    let values = match_template(&projection.statement, surface)?;
    let value = |name: &str| {
        values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    };
    let statement = Statement {
        subject: value("subject"),
        predicate: value("predicate"),
        object: value("object"),
    };
    let valid = |id: &str, role: &str| {
        meanings
            .iter()
            .any(|meaning| meaning.id == id && has_role(meaning, role))
    };
    (valid(&statement.subject, "wikidata_entity_anchor")
        && valid(&statement.predicate, "binary_relation_property")
        && valid(&statement.object, "wikidata_entity_anchor"))
    .then_some(statement)
}

fn render_natural(
    statement: &Statement,
    projection: &NaturalProjection,
    meanings: &[Meaning],
) -> Option<String> {
    let subject = first_word(
        meanings,
        &statement.subject,
        &projection.language,
        "wikidata_entity_anchor",
    )?;
    let object = first_word(
        meanings,
        &statement.object,
        &projection.language,
        "wikidata_entity_anchor",
    )?;
    let predicate = projection
        .relations
        .iter()
        .find(|(id, _)| id == &statement.predicate)
        .map(|(_, surface)| surface.as_str())
        .or_else(|| {
            first_word(
                meanings,
                &statement.predicate,
                &projection.language,
                "binary_relation_property",
            )
        })?;
    Some(render_template(
        &projection.statement,
        subject,
        predicate,
        object,
    ))
}

fn has_role(meaning: &Meaning, role: &str) -> bool {
    meaning.roles.iter().any(|candidate| candidate == role)
}

fn words<'a>(meaning: &'a Meaning, language: &str) -> Vec<&'a str> {
    meaning
        .lexemes
        .iter()
        .filter(|(candidate, _)| candidate == language)
        .flat_map(|(_, words)| words.iter().map(String::as_str))
        .collect()
}

fn meaning_with_role<'a>(meanings: &'a [Meaning], id: &str, role: &str) -> Option<&'a Meaning> {
    meanings
        .iter()
        .find(|meaning| meaning.id == id && has_role(meaning, role))
}

fn first_word<'a>(
    meanings: &'a [Meaning],
    id: &str,
    language: &str,
    role: &str,
) -> Option<&'a str> {
    meaning_with_role(meanings, id, role)
        .and_then(|meaning| words(meaning, language).into_iter().next())
}

fn render_template(template: &str, subject: &str, predicate: &str, object: &str) -> String {
    template
        .replace(SUBJECT_SLOT, subject)
        .replace(PREDICATE_SLOT, predicate)
        .replace(OBJECT_SLOT, object)
}

fn match_template(template: &str, surface: &str) -> Option<Vec<(String, String)>> {
    let mut template_rest = template;
    let mut surface_rest = surface.trim();
    let mut values = Vec::new();
    while let Some(open) = template_rest.find('{') {
        surface_rest = surface_rest.strip_prefix(&template_rest[..open])?;
        let close = template_rest[open + 1..].find('}')? + open + 1;
        let name = &template_rest[open + 1..close];
        template_rest = &template_rest[close + 1..];
        let next_open = template_rest.find('{').unwrap_or(template_rest.len());
        let literal = &template_rest[..next_open];
        let end = if literal.is_empty() {
            surface_rest.len()
        } else {
            surface_rest.find(literal)?
        };
        values.push((name.to_owned(), surface_rest[..end].trim().to_owned()));
        surface_rest = &surface_rest[end..];
    }
    surface_rest
        .strip_prefix(template_rest)
        .filter(|rest| rest.trim().is_empty())?;
    Some(values)
}

fn normalize_statement(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(['.', '。', '!', '?'])
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn surface_present(haystack: &str, needle: &str) -> bool {
    haystack.match_indices(needle).any(|(start, _)| {
        let end = start + needle.len();
        let left = haystack[..start]
            .chars()
            .next_back()
            .map_or(true, |character| !character.is_alphanumeric());
        let right = haystack[end..]
            .chars()
            .next()
            .map_or(true, |character| !character.is_alphanumeric());
        left && right
    })
}

fn serialize_answer(source: &str, target: &str, content: &str, statement: &Statement) -> String {
    let meaning = format!(
        "statement:{}({},{})",
        statement.predicate, statement.subject, statement.object
    );
    let mut output = String::from("{\"intent\":");
    push_json_string(&mut output, &format!("translate_{source}_to_{target}"));
    output.push_str(",\"content\":");
    push_json_string(&mut output, content);
    output.push_str(",\"confidence\":1.0,\"evidence\":[\"handler:translation\",");
    for (index, evidence) in [
        format!("language_from:{source}"),
        format!("language_to:{target}"),
        format!("meaning:{meaning}"),
    ]
    .iter()
    .enumerate()
    {
        if index > 0 {
            output.push(',');
        }
        push_json_string(&mut output, evidence);
    }
    output.push_str("]}");
    output
}
