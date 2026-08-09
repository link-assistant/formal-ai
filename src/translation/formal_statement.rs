//! Seed-defined projections between natural statements and formal syntax.
//!
//! Natural and formal surfaces are concrete syntaxes of one semantic triple.
//! Entity/relation identities come from the Wikidata meaning lexicon; templates,
//! aliases, word order, and canonical predicate surfaces come from
//! `data/seed/formal-language-projections.lino`. No language pair is encoded.

use std::fmt;
use std::sync::OnceLock;

use crate::seed;

const PROJECTIONS: &str = include_str!("../../data/seed/formal-language-projections.lino");
const SUBJECT_SLOT: &str = "{subject}";
const PREDICATE_SLOT: &str = "{predicate}";
const OBJECT_SLOT: &str = "{object}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticStatement {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl SemanticStatement {
    #[must_use]
    pub fn meaning(&self) -> String {
        format!(
            "statement:{}({},{})",
            self.predicate, self.subject, self.object
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementTranslation {
    pub surface: String,
    pub meaning: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementTranslationError {
    UnsupportedProjection(String),
    UnrecognizedStatement(String),
}

impl fmt::Display for StatementTranslationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProjection(language) => {
                write!(formatter, "unsupported statement projection: {language}")
            }
            Self::UnrecognizedStatement(surface) => {
                write!(formatter, "unrecognized semantic statement: {surface}")
            }
        }
    }
}

impl std::error::Error for StatementTranslationError {}

#[derive(Debug)]
struct ProjectionCatalog {
    formal: Vec<FormalProjection>,
    natural: Vec<NaturalProjection>,
}

#[derive(Debug)]
struct FormalProjection {
    slug: String,
    aliases: Vec<String>,
    statement: String,
}

#[derive(Debug)]
struct NaturalProjection {
    language: String,
    statement: String,
    relations: Vec<RelationSurface>,
}

#[derive(Debug)]
struct RelationSurface {
    id: String,
    predicate: String,
}

fn catalog() -> &'static ProjectionCatalog {
    static CATALOG: OnceLock<ProjectionCatalog> = OnceLock::new();
    CATALOG.get_or_init(|| parse_catalog(PROJECTIONS))
}

fn parse_catalog(raw: &str) -> ProjectionCatalog {
    let tree = seed::parser::parse_lino(raw);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "formal_language_projections");
    let mut formal = Vec::new();
    let mut natural = Vec::new();
    for node in root.into_iter().flat_map(|node| &node.children) {
        match node.name.as_str() {
            "formal_language" => formal.push(parse_formal_projection(node)),
            "natural_language" => natural.push(parse_natural_projection(node)),
            _ => {}
        }
    }
    ProjectionCatalog { formal, natural }
}

fn parse_formal_projection(node: &seed::parser::LinoNode) -> FormalProjection {
    FormalProjection {
        slug: node.id.clone(),
        aliases: node
            .children
            .iter()
            .filter(|child| child.name == "alias")
            .map(|child| child.id.to_lowercase())
            .collect(),
        statement: node.find_child_value("statement").to_owned(),
    }
}

fn parse_natural_projection(node: &seed::parser::LinoNode) -> NaturalProjection {
    NaturalProjection {
        language: node.id.clone(),
        statement: node.find_child_value("statement").to_owned(),
        relations: node
            .children
            .iter()
            .filter(|child| child.name == "relation")
            .map(|relation| RelationSurface {
                id: relation.id.clone(),
                predicate: relation.find_child_value("predicate").to_owned(),
            })
            .collect(),
    }
}

/// Formal language slugs supplied entirely by the projection seed.
#[must_use]
pub fn formal_language_targets() -> Vec<&'static str> {
    catalog()
        .formal
        .iter()
        .map(|projection| projection.slug.as_str())
        .collect()
}

/// The formal projection mentioned anywhere in a normalized request.
#[must_use]
pub fn formal_language_in_prompt(normalized: &str) -> Option<&'static str> {
    catalog().formal.iter().find_map(|projection| {
        projection
            .aliases
            .iter()
            .any(|alias| surface_present(normalized, alias))
            .then_some(projection.slug.as_str())
    })
}

/// Translate one complete semantic statement through the seeded meta layer.
pub fn translate_statement(
    surface: &str,
    source: &str,
    target: &str,
) -> Result<StatementTranslation, StatementTranslationError> {
    let statement = if catalog().formal.iter().any(|item| item.slug == source) {
        parse_formal_statement(surface, source)
    } else if catalog().natural.iter().any(|item| item.language == source) {
        parse_natural_statement(surface, source)
    } else {
        return Err(StatementTranslationError::UnsupportedProjection(
            source.to_owned(),
        ));
    }?;
    let target_surface = if catalog().formal.iter().any(|item| item.slug == target) {
        render_formal_statement(&statement, target)
    } else if catalog().natural.iter().any(|item| item.language == target) {
        render_natural_statement(&statement, target)
    } else {
        return Err(StatementTranslationError::UnsupportedProjection(
            target.to_owned(),
        ));
    }?;
    Ok(StatementTranslation {
        surface: target_surface,
        meaning: statement.meaning(),
    })
}

fn parse_natural_statement(
    surface: &str,
    language: &str,
) -> Result<SemanticStatement, StatementTranslationError> {
    let projection = catalog()
        .natural
        .iter()
        .find(|item| item.language == language)
        .ok_or_else(|| StatementTranslationError::UnsupportedProjection(language.to_owned()))?;
    let lexicon = seed::lexicon();
    for relation in &projection.relations {
        if meaning_with_role(lexicon, &relation.id, seed::ROLE_BINARY_RELATION_PROPERTY).is_none() {
            continue;
        }
        let predicate = relation.predicate.as_str();
        for subject in lexicon.meanings_with_role(seed::ROLE_WIKIDATA_ENTITY_ANCHOR) {
            for subject_surface in word_surfaces(subject, language) {
                for object in lexicon.meanings_with_role(seed::ROLE_WIKIDATA_ENTITY_ANCHOR) {
                    for object_surface in word_surfaces(object, language) {
                        let candidate = render_template(
                            &projection.statement,
                            subject_surface,
                            predicate,
                            object_surface,
                        );
                        if normalized_statement(&candidate) == normalized_statement(surface) {
                            return Ok(SemanticStatement {
                                subject: subject.wikidata.clone(),
                                predicate: relation.id.clone(),
                                object: object.wikidata.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
    Err(StatementTranslationError::UnrecognizedStatement(
        surface.to_owned(),
    ))
}

fn parse_formal_statement(
    surface: &str,
    language: &str,
) -> Result<SemanticStatement, StatementTranslationError> {
    let projection = catalog()
        .formal
        .iter()
        .find(|item| item.slug == language)
        .ok_or_else(|| StatementTranslationError::UnsupportedProjection(language.to_owned()))?;
    let values = match_template(&projection.statement, surface)
        .ok_or_else(|| StatementTranslationError::UnrecognizedStatement(surface.to_owned()))?;
    let value = |name: &str| {
        values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    };
    let statement = SemanticStatement {
        subject: value("subject").unwrap_or_default(),
        predicate: value("predicate").unwrap_or_default(),
        object: value("object").unwrap_or_default(),
    };
    let lexicon = seed::lexicon();
    let valid_subject = meaning_with_role(
        lexicon,
        &statement.subject,
        seed::ROLE_WIKIDATA_ENTITY_ANCHOR,
    )
    .is_some();
    let valid_predicate = meaning_with_role(
        lexicon,
        &statement.predicate,
        seed::ROLE_BINARY_RELATION_PROPERTY,
    )
    .is_some();
    let valid_object = meaning_with_role(
        lexicon,
        &statement.object,
        seed::ROLE_WIKIDATA_ENTITY_ANCHOR,
    )
    .is_some();
    if valid_subject && valid_predicate && valid_object {
        Ok(statement)
    } else {
        Err(StatementTranslationError::UnrecognizedStatement(
            surface.to_owned(),
        ))
    }
}

fn render_formal_statement(
    statement: &SemanticStatement,
    language: &str,
) -> Result<String, StatementTranslationError> {
    let projection = catalog()
        .formal
        .iter()
        .find(|item| item.slug == language)
        .ok_or_else(|| StatementTranslationError::UnsupportedProjection(language.to_owned()))?;
    Ok(render_template(
        &projection.statement,
        &statement.subject,
        &statement.predicate,
        &statement.object,
    ))
}

fn render_natural_statement(
    statement: &SemanticStatement,
    language: &str,
) -> Result<String, StatementTranslationError> {
    let projection = catalog()
        .natural
        .iter()
        .find(|item| item.language == language)
        .ok_or_else(|| StatementTranslationError::UnsupportedProjection(language.to_owned()))?;
    let lexicon = seed::lexicon();
    let subject = meaning_with_role(
        lexicon,
        &statement.subject,
        seed::ROLE_WIKIDATA_ENTITY_ANCHOR,
    )
    .and_then(|meaning| meaning.word_in(language));
    let object = meaning_with_role(
        lexicon,
        &statement.object,
        seed::ROLE_WIKIDATA_ENTITY_ANCHOR,
    )
    .and_then(|meaning| meaning.word_in(language));
    let predicate = projection
        .relations
        .iter()
        .find(|relation| relation.id == statement.predicate)
        .map(|relation| relation.predicate.as_str())
        .or_else(|| {
            meaning_with_role(
                lexicon,
                &statement.predicate,
                seed::ROLE_BINARY_RELATION_PROPERTY,
            )
            .and_then(|meaning| meaning.word_in(language))
        });
    match (subject, predicate, object) {
        (Some(subject), Some(predicate), Some(object)) => Ok(render_template(
            &projection.statement,
            subject,
            predicate,
            object,
        )),
        _ => Err(StatementTranslationError::UnrecognizedStatement(
            statement.meaning(),
        )),
    }
}

fn meaning_with_role<'a>(
    lexicon: &'a seed::Lexicon,
    id: &str,
    role: &str,
) -> Option<&'a seed::Meaning> {
    lexicon
        .meanings
        .iter()
        .find(|meaning| meaning.wikidata == id && meaning.has_role(role))
}

fn word_surfaces<'a>(meaning: &'a seed::Meaning, language: &str) -> Vec<&'a str> {
    meaning
        .lexemes
        .iter()
        .filter(|lexeme| lexeme.language == language)
        .flat_map(|lexeme| lexeme.words.iter().map(|word| word.text.as_str()))
        .collect()
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
        let literal = &template_rest[..open];
        surface_rest = surface_rest.strip_prefix(literal)?;
        let close = template_rest[open + 1..].find('}')? + open + 1;
        let name = &template_rest[open + 1..close];
        template_rest = &template_rest[close + 1..];
        let next_open = template_rest.find('{').unwrap_or(template_rest.len());
        let next_literal = &template_rest[..next_open];
        let value_end = if next_literal.is_empty() {
            surface_rest.len()
        } else {
            surface_rest.find(next_literal)?
        };
        values.push((name.to_owned(), surface_rest[..value_end].trim().to_owned()));
        surface_rest = &surface_rest[value_end..];
    }
    surface_rest
        .strip_prefix(template_rest)
        .filter(|rest| rest.trim().is_empty())?;
    Some(values)
}

fn normalized_statement(value: &str) -> String {
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
            .is_none_or(|character| !character.is_alphanumeric());
        let right = haystack[end..]
            .chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric());
        left && right
    })
}
