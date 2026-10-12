// Computed trust features, document-conversion sources and prompt-supplied
// pages (issue #1163 R7, R10, R13). Included from `web_formalize.rs`, so it
// shares that module's imports and seed texts.

/// The trust-feature detectors the seed declares beside the weights
/// (`data/seed/source-trust-weights.lino`): the open-license markers and the
/// Wikidata property naming a subject's official website.
#[derive(Debug, Clone, Default)]
struct TrustFeatureRules {
    license_markers: Vec<String>,
    official_site_property: String,
}

impl TrustFeatureRules {
    fn load() -> Self {
        let tree = parse_lino(TRUST_TEXT);
        let mut rules = Self::default();
        for record in seed_records(&tree) {
            if record.name != "feature" {
                continue;
            }
            for child in &record.children {
                match child.name.as_str() {
                    "license_marker" if !child.id.is_empty() => {
                        rules.license_markers.push(child.id.to_ascii_lowercase());
                    }
                    "claim_property" if !child.id.is_empty() => {
                        rules.official_site_property.clone_from(&child.id);
                    }
                    _ => {}
                }
            }
        }
        rules
    }
}

/// The official websites a Wikidata entity payload states.
///
/// The payload is `Special:EntityData` JSON, read through the seed's
/// `claim_property` (P856), so the official-site feature is resolved from data
/// rather than asserted by a caller (issue #1163 R7).
#[must_use]
pub fn official_websites(entity_json: &[u8]) -> Vec<String> {
    let property = TrustFeatureRules::load().official_site_property;
    let Ok(value) = serde_json::from_slice::<Value>(entity_json) else {
        return Vec::new();
    };
    let Some(entities) = value.get("entities").and_then(Value::as_object) else {
        return Vec::new();
    };
    entities
        .values()
        .filter_map(|entity| entity.get("claims")?.get(property.as_str())?.as_array())
        .flatten()
        .filter_map(|claim| {
            claim
                .get("mainsnak")?
                .get("datavalue")?
                .get("value")?
                .as_str()
        })
        .map(str::to_owned)
        .collect()
}

/// The primacy grade of the registered source whose endpoint shares the
/// page's domain, if one does.
fn registry_primacy(domain: &str) -> Option<String> {
    if domain.is_empty() {
        return None;
    }
    crate::seed::source_registry()
        .into_iter()
        .filter(|record| !record.api.is_empty() && url_domain(&record.api) == domain)
        .find_map(|record| {
            record
                .primacy
                .steps
                .first()
                .map(|step| step.kind.slug().to_owned())
        })
}

/// The statement texts two pages can agree on: paragraphs and code blocks.
fn agreement_texts(blocks: &[(LinkId, PageBlock)]) -> std::collections::BTreeSet<String> {
    blocks
        .iter()
        .filter_map(|(_, block)| match block {
            PageBlock::Paragraph { text, .. } | PageBlock::CodeBlock { text, .. } => {
                Some(text.clone())
            }
            _ => None,
        })
        .filter(|text| !text.trim().is_empty())
        .collect()
}

/// How many stored pages from other domains state at least one of the
/// page's paragraphs or code blocks verbatim.
fn agreeing_pages(url: &str, bytes: &[u8], store: &FormalizedPageStore) -> u32 {
    let own = agreement_texts(&formalize_page_with_context(bytes, None, Some(url)).1);
    let domain = url_domain(url);
    let count = store
        .pages
        .values()
        .filter(|page| url_domain(&page.url) != domain)
        .filter(|page| {
            page.paragraphs
                .iter()
                .map(|paragraph| &paragraph.text)
                .chain(page.code_blocks.iter().map(|block| &block.text))
                .any(|text| own.contains(text))
        })
        .count();
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Compute a page's trust features from what is known about it.
///
/// Issue #1163 R7: HTTPS from the URL, the primacy of a registered source on
/// the same domain, an open license from the seed's markers in the bytes,
/// agreement from the other pages in `store`, and the official site from the
/// `official_sites` a Wikidata entity states (see [`official_websites`]).
#[must_use]
pub fn trust_features_for(
    url: &str,
    bytes: &[u8],
    store: &FormalizedPageStore,
    official_sites: &[String],
) -> TrustFeatures {
    let rules = TrustFeatureRules::load();
    let domain = url_domain(url);
    let lower = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    TrustFeatures {
        official_site: !domain.is_empty()
            && official_sites.iter().any(|site| url_domain(site) == domain),
        https: url.starts_with(HTTPS_SCHEME),
        primacy: registry_primacy(&domain),
        open_license: rules
            .license_markers
            .iter()
            .any(|marker| lower.contains(marker.as_str())),
        agreement_pages: agreeing_pages(url, bytes, store),
    }
}

/// The seed's `document_source` row a document format names.
///
/// Returned as `(name, mime hint)`: the format is formalized under that hint as
/// a document-conversion source (issue #1163 R13). The row's name or one of its
/// `alias` children must equal the format, ASCII case-insensitively; `None` for
/// a format the seed does not list.
#[must_use]
pub fn document_source(format: &str) -> Option<(String, String)> {
    let wanted = format.trim().to_ascii_lowercase();
    if wanted.is_empty() {
        return None;
    }
    let tree = parse_lino(RULES_TEXT);
    seed_records(&tree)
        .iter()
        .filter(|record| record.name == "document_source")
        .find(|record| {
            record.id.to_ascii_lowercase() == wanted
                || record
                    .children
                    .iter()
                    .any(|child| child.name == "alias" && child.id.to_ascii_lowercase() == wanted)
        })
        .map(|record| {
            (
                record.id.clone(),
                record.find_child_value("hint").to_owned(),
            )
        })
        .filter(|(_, hint)| !hint.is_empty())
}

/// Split a prompt that asks a page query over a page it supplies.
///
/// Issue #1163 R10: the first line is the query, with the seed's
/// `supplied_page_separator` trimmed from its end, and the rest is the page.
/// `None` when either half is empty.
#[must_use]
pub fn split_supplied_page(prompt: &str) -> Option<(String, String)> {
    let tree = parse_lino(RULES_TEXT);
    let separator = seed_records(&tree)
        .iter()
        .find(|record| record.name == "supplied_page_separator")
        .map(|record| record.id.clone())
        .unwrap_or_default();
    let (first, rest) = prompt.trim().split_once('\n')?;
    let mut query = first.trim();
    if !separator.is_empty() {
        query = query.trim_end_matches(separator.as_str()).trim_end();
    }
    let page = rest.trim();
    (!query.is_empty() && !page.is_empty()).then(|| (query.to_owned(), page.to_owned()))
}
