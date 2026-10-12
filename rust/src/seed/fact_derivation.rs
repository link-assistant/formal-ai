//! Fact records derived from committed Wikidata captures (issue #1172 R9).
//!
//! A record is a subject entity of `data/seed/fact-captures.lino` whose first
//! claim's property grounds a `fact_relation` meaning that
//! `data/seed/fact-realization.lino` realizes, and whose value item is captured
//! too. The value is that claim's item; labels and sources come from the two
//! captures; the sentence is the relation's template with each slot filled by
//! the entity's preferred surface in the form the slot names (the English
//! article, a Russian case). The records have the shape of the written ones,
//! so every consumer of [`super::facts`] reads both alike. Mirrored by
//! `js/worker/formal_ai_worker_fact_derivation.js` (browser worker) and
//! `js/agentic/crate/fact_derivation.mjs` (agentic port).

use std::sync::OnceLock;

use super::embedded::{FACT_CAPTURES_LINO, FACT_REALIZATION_LINO};
use super::facts::{FactRecord, LocalizedFact};
use super::parser::{LinoNode, parse_lino, split_pipe_list};

const SUBJECT_SLOT: &str = "subject";
const VALUE_SLOT: &str = "value";
const DEFINITE_FORM: &str = "definite";
const LANGUAGE_SLOT: &str = "{language}";
const TITLE_SLOT: &str = "{title}";

/// One captured claim: a property and the item it states.
#[derive(Debug, Clone)]
struct Claim {
    property: String,
    value: String,
}

/// One captured entity of the projection.
#[derive(Debug, Clone, Default)]
struct Entity {
    qid: String,
    labels: Vec<(String, String)>,
    sitelinks: Vec<(String, String)>,
    claims: Vec<Claim>,
}

impl Entity {
    fn label(&self, key: &str) -> Option<&str> {
        lookup(&self.labels, key).map(String::as_str)
    }
}

#[derive(Debug, Clone, Default)]
struct Article {
    definite: String,
    heads: Vec<String>,
    possessive: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct CaseRule {
    endings: Vec<(String, String)>,
    after_consonant: String,
}

#[derive(Debug, Clone, Default)]
struct Morphology {
    initial_mark: String,
    acronyms: bool,
    consonants: String,
    cases: Vec<(String, CaseRule)>,
}

#[derive(Debug, Clone, Default)]
struct Relation {
    slug: String,
    intent: String,
    category: String,
    source_kind: String,
    source_page: String,
    order: Vec<String>,
    keywords: Vec<String>,
    sentences: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
struct SourceRule {
    site: String,
    url: String,
    title_space: String,
}

#[derive(Debug, Clone, Default)]
struct Rules {
    languages: Vec<String>,
    variants: Vec<(String, Vec<String>)>,
    absorbed_mark: String,
    source: Option<SourceRule>,
    relations: Vec<Relation>,
    preferred: Vec<(String, Vec<(String, String)>)>,
    surfaces: Vec<(String, Vec<String>)>,
    articles: Vec<(String, Article)>,
    morphology: Vec<(String, Morphology)>,
}

fn lookup<'a, T>(pairs: &'a [(String, T)], key: &str) -> Option<&'a T> {
    pairs
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value)
}

fn children<'a>(node: &'a LinoNode, name: &'a str) -> impl Iterator<Item = &'a LinoNode> {
    node.children.iter().filter(move |child| child.name == name)
}

fn child_list(node: &LinoNode, name: &str) -> Vec<String> {
    split_pipe_list(node.find_child_value(name))
}

/// The top-level record `name` of a seed document.
fn seed_root(text: &str, name: &str) -> Option<LinoNode> {
    let tree = parse_lino(text);
    if tree.name == name {
        return Some(tree);
    }
    tree.children.into_iter().find(|child| child.name == name)
}

/// `language (a b)` records: the leading word, then the list after it.
fn keyed_lists(root: &LinoNode, name: &str) -> Vec<(String, Vec<String>)> {
    children(root, name)
        .filter_map(|node| {
            let (key, rest) = node.id.split_once(' ')?;
            Some((key.to_owned(), split_pipe_list(rest)))
        })
        .collect()
}

fn captured_entities(root: &LinoNode) -> Vec<Entity> {
    children(root, "entity")
        .map(|node| {
            let mut entity = Entity {
                qid: node.id.clone(),
                ..Entity::default()
            };
            for child in &node.children {
                let items = split_pipe_list(&child.id);
                match (child.name.as_str(), items.as_slice()) {
                    ("label", [language, text]) => {
                        entity.labels.push((language.clone(), text.clone()));
                    }
                    ("sitelink", [site, title]) => {
                        entity.sitelinks.push((site.clone(), title.clone()));
                    }
                    ("claim", [property, value, _rank]) => entity.claims.push(Claim {
                        property: property.clone(),
                        value: value.clone(),
                    }),
                    _ => {}
                }
            }
            entity
        })
        .collect()
}

fn case_rule(node: &LinoNode) -> CaseRule {
    let mut endings: Vec<(String, String)> = children(node, "ending")
        .filter_map(|ending| match split_pipe_list(&ending.id).as_slice() {
            [from, to] => Some((from.clone(), to.clone())),
            _ => None,
        })
        .collect();
    endings.sort_by_key(|(from, _)| std::cmp::Reverse(from.chars().count()));
    CaseRule {
        endings,
        after_consonant: node.find_child_value("after_consonant").to_owned(),
    }
}

fn relation(node: &LinoNode) -> Relation {
    Relation {
        slug: node.id.clone(),
        intent: node.find_child_value("intent").to_owned(),
        category: node.find_child_value("category").to_owned(),
        source_kind: node.find_child_value("source_kind").to_owned(),
        source_page: node.find_child_value("source_page").to_owned(),
        order: child_list(node, "wikidata_order"),
        keywords: child_list(node, "question_keywords"),
        sentences: children(node, "sentence")
            .map(|sentence| {
                (
                    sentence.id.clone(),
                    sentence.find_child_value("text").to_owned(),
                )
            })
            .collect(),
    }
}

fn realization_rules(root: &LinoNode) -> Rules {
    Rules {
        languages: child_list(root, "languages"),
        variants: keyed_lists(root, "label_variant"),
        absorbed_mark: root.find_child_value("absorbed_mark").to_owned(),
        source: children(root, "source").next().map(|node| SourceRule {
            site: node.find_child_value("site").to_owned(),
            url: node.find_child_value("url").to_owned(),
            title_space: node.find_child_value("title_space").to_owned(),
        }),
        relations: children(root, "relation").map(relation).collect(),
        preferred: children(root, "preferred_label")
            .map(|node| {
                let labels = node
                    .children
                    .iter()
                    .map(|entry| {
                        (
                            entry.name.clone(),
                            entry.find_child_value("text").to_owned(),
                        )
                    })
                    .collect();
                (node.id.clone(), labels)
            })
            .collect(),
        surfaces: keyed_lists(root, "prompt_surface"),
        articles: children(root, "article")
            .map(|node| {
                let article = Article {
                    definite: node.find_child_value("definite").to_owned(),
                    heads: child_list(node, "definite_head"),
                    possessive: child_list(node, "possessive"),
                };
                (node.id.clone(), article)
            })
            .collect(),
        morphology: children(root, "morphology")
            .map(|node| {
                let morphology = Morphology {
                    initial_mark: node.find_child_value("initial_mark").to_owned(),
                    acronyms: node.find_child_value("indeclinable_acronym") == "true",
                    consonants: node.find_child_value("consonants").to_owned(),
                    cases: children(node, "case")
                        .map(|case| (case.id.clone(), case_rule(case)))
                        .collect(),
                };
                (node.id.clone(), morphology)
            })
            .collect(),
    }
}

impl Rules {
    /// The entity's preferred label, else its label in the first captured
    /// variant the language lists, else its label.
    fn surface(&self, entity: &Entity, language: &str) -> String {
        if let Some(text) = lookup(&self.preferred, &entity.qid)
            .and_then(|labels| lookup(labels, language))
            .filter(|text| !text.is_empty())
        {
            return text.clone();
        }
        let own = [language.to_owned()];
        let variants = lookup(&self.variants, language).map_or(own.as_slice(), Vec::as_slice);
        variants
            .iter()
            .find_map(|variant| entity.label(variant))
            .or_else(|| entity.label(language))
            .unwrap_or_default()
            .to_owned()
    }

    fn definite(&self, text: &str, language: &str) -> String {
        let Some(article) = lookup(&self.articles, language).filter(|a| !a.definite.is_empty())
        else {
            return text.to_owned();
        };
        let lower = text.to_lowercase();
        let last = lower.rsplit(' ').next().unwrap_or_default();
        if text.is_empty()
            || lower.starts_with(&format!("{} ", article.definite))
            || !article.heads.iter().any(|head| head == last)
        {
            return text.to_owned();
        }
        format!("{} {text}", article.definite)
    }

    fn inflect(&self, text: &str, language: &str, case: &str) -> String {
        let Some(morphology) = lookup(&self.morphology, language) else {
            return text.to_owned();
        };
        let Some(rule) = lookup(&morphology.cases, case) else {
            return text.to_owned();
        };
        let mut words: Vec<String> = text.split(' ').map(str::to_owned).collect();
        let is_initial = |word: &str| {
            !morphology.initial_mark.is_empty() && word.ends_with(&morphology.initial_mark)
        };
        let Some(index) = words
            .iter()
            .position(|word| !(is_initial(word) || (morphology.acronyms && is_acronym(word))))
        else {
            return text.to_owned();
        };
        let word = words[index].clone();
        let lower = word.to_lowercase();
        let inflected = rule
            .endings
            .iter()
            .find(|(from, _)| lower.ends_with(from.as_str()))
            .map(|(from, to)| {
                let keep = word.chars().count().saturating_sub(from.chars().count());
                format!("{}{to}", word.chars().take(keep).collect::<String>())
            })
            .or_else(|| {
                let last = lower.chars().last()?;
                (!rule.after_consonant.is_empty() && morphology.consonants.contains(last))
                    .then(|| format!("{word}{}", rule.after_consonant))
            });
        let Some(inflected) = inflected else {
            return text.to_owned();
        };
        words[index] = inflected;
        words.join(" ")
    }

    fn form(&self, text: &str, language: &str, form: &str) -> String {
        match form {
            "" => text.to_owned(),
            DEFINITE_FORM => self.definite(text, language),
            case => self.inflect(text, language, case),
        }
    }

    /// Fill a sentence template; a slot ending in the absorbed mark takes
    /// the place of the same mark right after it.
    fn realize(&self, template: &str, language: &str, subject: &Entity, value: &Entity) -> String {
        let mut out = String::new();
        let mut rest = template;
        while let Some(open) = rest.find('{') {
            let Some(close) = rest[open..].find('}').map(|offset| open + offset) else {
                break;
            };
            let slot = &rest[open + 1..close];
            let (role, form) = slot.split_once(':').unwrap_or((slot, ""));
            let entity = match role {
                SUBJECT_SLOT => subject,
                VALUE_SLOT => value,
                _ => {
                    out.push_str(&rest[..=close]);
                    rest = &rest[close + 1..];
                    continue;
                }
            };
            out.push_str(&rest[..open]);
            let filled = self.form(&self.surface(entity, language), language, form);
            out.push_str(&filled);
            rest = &rest[close + 1..];
            let mark = self.absorbed_mark.as_str();
            if !mark.is_empty() && filled.ends_with(mark) {
                rest = rest.strip_prefix(mark).unwrap_or(rest);
            }
        }
        out.push_str(rest);
        out
    }

    /// The page of `entity` in the language's edition, from its sitelink.
    fn source_url(&self, entity: &Entity, language: &str) -> String {
        let Some(source) = &self.source else {
            return String::new();
        };
        let site = source.site.replace(LANGUAGE_SLOT, language);
        lookup(&entity.sitelinks, &site).map_or_else(String::new, |title| {
            source
                .url
                .replace(LANGUAGE_SLOT, language)
                .replace(TITLE_SLOT, &title.replace(' ', &source.title_space))
        })
    }

    /// The label-index surfaces of a subject, lowercased and deduplicated.
    fn label_surfaces(&self, entity: &Entity) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut add = |text: &str| {
            let lower = text.to_lowercase();
            if !lower.is_empty() && !out.contains(&lower) {
                out.push(lower);
            }
        };
        let mut keys = self.languages.clone();
        for language in &self.languages {
            for variant in lookup(&self.variants, language).into_iter().flatten() {
                if !keys.contains(variant) {
                    keys.push(variant.clone());
                }
            }
        }
        let preferred = lookup(&self.preferred, &entity.qid);
        for key in &keys {
            let own = entity.label(key);
            let favoured = preferred
                .and_then(|labels| lookup(labels, key))
                .map(String::as_str);
            for label in [own, favoured]
                .into_iter()
                .flatten()
                .filter(|l| !l.is_empty())
            {
                add(label);
                if let Some(article) =
                    lookup(&self.articles, key).filter(|a| !a.definite.is_empty())
                {
                    let lead = format!("{} ", article.definite);
                    if label.to_lowercase().starts_with(&lead) {
                        add(label.get(lead.len()..).unwrap_or_default());
                    }
                    add(&self.definite(label, key));
                    if !label.contains(' ') {
                        for ending in &article.possessive {
                            add(&format!("{label}{ending}"));
                        }
                    }
                }
                if let Some(morphology) = lookup(&self.morphology, key) {
                    for (case, _) in &morphology.cases {
                        add(&self.inflect(label, key, case));
                    }
                }
            }
        }
        for surface in lookup(&self.surfaces, &entity.qid).into_iter().flatten() {
            add(surface);
        }
        out
    }

    fn record(&self, relation: &Relation, subject: &Entity, value: &Entity) -> FactRecord {
        let page = if relation.source_page == SUBJECT_SLOT {
            subject
        } else {
            value
        };
        let localized: Vec<LocalizedFact> = self
            .languages
            .iter()
            .filter_map(|language| {
                let template = lookup(&relation.sentences, language)?;
                Some(LocalizedFact {
                    language: language.clone(),
                    subject_label: self.definite(&self.surface(subject, language), language),
                    value_label: self.surface(value, language),
                    summary: self.realize(template, language, subject, value),
                    source: self.source_url(page, language),
                    source_kind: relation.source_kind.clone(),
                })
            })
            .collect();
        let lead = localized.first().cloned().unwrap_or_default();
        FactRecord {
            slug: format!("fact_{}_{}", relation.slug, subject.qid.to_lowercase()),
            intent: if relation.intent.is_empty() {
                String::from("fact_lookup")
            } else {
                relation.intent.clone()
            },
            category: relation.category.clone(),
            wikidata: relation
                .order
                .iter()
                .map(|role| {
                    if role == SUBJECT_SLOT {
                        subject.qid.clone()
                    } else {
                        value.qid.clone()
                    }
                })
                .collect(),
            relation: relation.slug.clone(),
            subject_qid: subject.qid.clone(),
            value_qid: value.qid.clone(),
            subject_label: lead.subject_label,
            value_label: lead.value_label,
            subject_aliases: self.label_surfaces(subject),
            question_keywords: relation.keywords.iter().map(|k| k.to_lowercase()).collect(),
            summary: lead.summary,
            source: lead.source,
            source_kind: relation.source_kind.clone(),
            release_timeline: String::new(),
            localized,
        }
    }
}

/// An all-capital abbreviation of two or more letters.
fn is_acronym(word: &str) -> bool {
    let letters: Vec<char> = word
        .chars()
        .filter(|character| character.is_lowercase() || character.is_uppercase())
        .collect();
    letters.len() > 1 && letters.iter().all(|character| character.is_uppercase())
}

fn build_derived_facts() -> Vec<FactRecord> {
    let (Some(captures), Some(realization)) = (
        seed_root(FACT_CAPTURES_LINO, "fact_captures"),
        seed_root(FACT_REALIZATION_LINO, "fact_realization"),
    ) else {
        return Vec::new();
    };
    let rules = realization_rules(&realization);
    let entities = captured_entities(&captures);
    let lexicon = super::lexicon();
    let relation_of = |property: &str| {
        rules.relations.iter().find(|relation| {
            lexicon
                .meaning(&relation.slug)
                .is_some_and(|meaning| meaning.wikidata == property)
        })
    };
    entities
        .iter()
        .filter_map(|subject| {
            let claim = subject.claims.first()?;
            let relation = relation_of(&claim.property)?;
            let value = entities.iter().find(|entity| entity.qid == claim.value)?;
            Some(rules.record(relation, subject, value))
        })
        .collect()
}

/// The fact records the committed captures and the realization rules derive,
/// in capture order. Parsed once per process.
#[must_use]
pub fn derived_facts() -> &'static [FactRecord] {
    static RECORDS: OnceLock<Vec<FactRecord>> = OnceLock::new();
    RECORDS.get_or_init(build_derived_facts).as_slice()
}
