//! Host observation of the pinned grammar network; no copied lexical parser.
use super::{source_from_links, text_field};
use meta_language::{LinkId, LinkNetwork, LinkType, ParseConfiguration};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type Rules = [(String, String, String)];
struct Projection<'a> {
    network: &'a LinkNetwork,
    children: BTreeMap<LinkId, Vec<LinkId>>,
    roles: BTreeMap<LinkId, String>,
    rules: &'a Rules,
    language: &'a str,
}

impl Projection<'_> {
    fn node(
        &self,
        identifier: LinkId,
        visited: &mut BTreeSet<LinkId>,
        depth: usize,
    ) -> Result<Value, String> {
        if depth > 512 {
            return Err("SourceModelDepthExceeded".to_owned());
        }
        let link = self
            .network
            .link(identifier)
            .ok_or_else(|| "MissingSourceNode".to_owned())?;
        let metadata = link.metadata();
        if metadata.flags().has_error() || metadata.flags().is_missing() {
            return Err("UnrepresentedSourceSyntax".to_owned());
        }
        let descendants = self
            .children
            .get(&identifier)
            .map_or(&[][..], Vec::as_slice);
        let is_token = metadata.link_type() == Some(LinkType::Token);
        let form = if is_token {
            if metadata.flags().is_extra() {
                "whitespace"
            } else {
                "hidden_text"
            }
        } else {
            metadata
                .term()
                .ok_or_else(|| "MissingSourceForm".to_owned())?
        };
        let concept = self
            .rules
            .iter()
            .find(|(language, kind, _)| language == self.language && kind == form)
            .map_or("syntax", |(_, _, concept)| concept.as_str());
        let mut result = vec![json!(concept), text_field("form", form)];
        if let Some(role) = self.roles.get(&identifier) {
            result.push(text_field("role", role));
        }
        if is_token {
            if !visited.insert(identifier) {
                return Err("AmbiguousSourceLeaf".to_owned());
            }
            result.push(text_field(
                "text",
                metadata
                    .term()
                    .ok_or_else(|| "MissingSourceText".to_owned())?,
            ));
        } else if descendants.len() == 1
            && self
                .network
                .link(descendants[0])
                .is_some_and(|child| child.metadata().link_type() == Some(LinkType::Token))
        {
            let child = self
                .network
                .link(descendants[0])
                .ok_or_else(|| "MissingSourceLeaf".to_owned())?;
            if !visited.insert(child.id()) {
                return Err("AmbiguousSourceLeaf".to_owned());
            }
            result.push(text_field(
                "text",
                child
                    .metadata()
                    .term()
                    .ok_or_else(|| "MissingSourceText".to_owned())?,
            ));
        } else {
            for child in descendants {
                result.push(self.node(*child, visited, depth + 1)?);
            }
        }
        Ok(Value::Array(result))
    }
}

pub(super) fn convert(source: &str, language: &str, rules: &Rules) -> Result<Value, String> {
    let network = LinkNetwork::parse(source, language, ParseConfiguration::default());
    if network.reconstruct_text() != source || !network.verify_full_match(None).is_clean() {
        return Err("IncompleteSourcePartition".to_owned());
    }
    let mut projection = Projection {
        network: &network,
        children: BTreeMap::new(),
        roles: BTreeMap::new(),
        rules,
        language,
    };
    for link in network.links() {
        let references = link.references();
        match link.metadata().link_type() {
            Some(LinkType::Syntax | LinkType::Token) => {
                if let Some(parent) = references.first() {
                    projection
                        .children
                        .entry(*parent)
                        .or_default()
                        .push(link.id());
                }
            }
            Some(LinkType::Field) if references.len() == 3 => {
                let role = network
                    .link(references[1])
                    .and_then(|label| label.metadata().term())
                    .ok_or_else(|| "MissingSourceRole".to_owned())?;
                if projection
                    .roles
                    .insert(references[2], role.to_owned())
                    .is_some()
                {
                    return Err("AmbiguousSourceRole".to_owned());
                }
            }
            _ => {}
        }
    }
    for children in projection.children.values_mut() {
        children.sort_by_key(|identifier| {
            network
                .link(*identifier)
                .and_then(|link| link.metadata().span())
                .map(|span| (span.byte_range().start(), span.byte_range().end()))
        });
    }
    let roots = network
        .links()
        .filter(|link| link.metadata().link_type() == Some(LinkType::Document))
        .filter_map(|link| projection.children.get(&link.id()))
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err("OneSourceTreeRequired".to_owned());
    }
    let root_form = rules
        .iter()
        .find(|(rule_language, _, concept)| rule_language == language && concept == "module")
        .map(|(_, kind, _)| kind.as_str())
        .ok_or_else(|| "SourceRootRuleRequired".to_owned())?;
    if roots[0]
        .iter()
        .filter(|identifier| {
            network.link(**identifier).is_some_and(|link| {
                link.metadata().link_type() == Some(LinkType::Syntax)
                    && link.metadata().term() == Some(root_form)
            })
        })
        .count()
        != 1
    {
        return Err("UnrepresentedSourceSyntax".to_owned());
    }
    let mut visited = BTreeSet::new();
    let mut result = vec![json!("source"), text_field("language", language)];
    for identifier in roots[0] {
        result.push(projection.node(*identifier, &mut visited, 0)?);
    }
    let token_count = network
        .links()
        .filter(|link| link.metadata().link_type() == Some(LinkType::Token))
        .count();
    if visited.len() != token_count {
        return Err("UnattachedSourceText".to_owned());
    }
    let document = Value::Array(result);
    if source_from_links(&document)? != source {
        return Err("SourceReconstructionMismatch".to_owned());
    }
    Ok(document)
}
