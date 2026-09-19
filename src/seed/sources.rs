//! The runtime source registry parsed from `data/seed/sources-registry.lino`.
//!
//! Before issue #991 the registry existed only as data: the browser worker
//! hardcoded a parallel `EXTERNAL_TRUSTED_SERVICES` table and the Rust side had
//! no loader at all, so "enabled relevant services from `sources-registry.lino`
//! can contribute" could not be true on the native path. This module makes the
//! seed file the single source of truth for both runtimes: which services
//! exist, which settings key opts each one out, what its API template is, and
//! under which license its bytes may be quoted.
//!
//! Issue #1073 (requirement 4) changed where one field comes from. The registry
//! used to declare `source_tier` per source and this loader read it back
//! verbatim, defaulting silently to `independent_corroboration` for anything
//! that declared nothing — trust asserted, and asserted by omission at that.
//! Now every entry declares its `primacy` chain (how far it stands from the
//! primary record, and the source's own policy page establishing each hop) and
//! [`SourceRecord::tier`] is *derived* from that chain by
//! [`PrimacyChain::derive_tier`]. The declared `source_tier` survives only as
//! [`SourceRecord::asserted_tier`], an assertion the derivation is checked
//! against rather than an input to it.

use std::fmt::Write as _;

use super::embedded::SOURCES_REGISTRY_LINO;
use super::parser::parse_lino;
use crate::needs::NeedKind;
use crate::reasoning_standard::episode::tier_from_slug;
use crate::reasoning_standard::trust::{PrimacyChain, chain_from_node};
use crate::relative_meta_logic::SourceTier;

/// The `service_group` marking a live, opt-out-able external service.
pub const EXTERNAL_TRUSTED_GROUP: &str = "external_trusted";

/// The part a registered source may play in procedural synthesis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum HowToRole {
    /// Structured, ordered steps: consulted first.
    Primary,
    /// Corroborating procedure: consulted after the primary sources.
    Secondary,
    /// Not a procedural source; never consulted for "how to X".
    #[default]
    None,
}

impl HowToRole {
    /// Stable slug used in the registry and in evidence events.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::None => "none",
        }
    }

    fn from_seed(value: &str) -> Self {
        match value {
            "primary" => Self::Primary,
            "secondary" => Self::Secondary,
            _ => Self::None,
        }
    }

    /// Whether a source in this role may contribute steps.
    #[must_use]
    pub const fn contributes(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// One declared retrieval source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRecord {
    /// Registry id (`wikihow`, `stackexchange`, …).
    pub id: String,
    /// Human label shown in traces and rendered guides.
    pub name: String,
    /// What the source is (`how_to_guide`, `qa_network`, `software_docs`, …).
    pub kind: String,
    /// `external_trusted` for live, settings-gated services; empty otherwise.
    pub service_group: String,
    /// Settings toggle that opts the service out (`externalServiceWikihow`, …).
    pub settings_key: String,
    /// Whether the service participates when settings say nothing about it.
    pub default_enabled: bool,
    /// Role this source plays in "how to X" synthesis: `primary` (structured
    /// ordered steps), `secondary` (corroborating procedure), or `none`.
    pub how_to_role: HowToRole,
    /// The #709 relative-meta-logic tier the source's bytes carry.
    ///
    /// Issue #1073, requirement 4: this is *derived* from [`Self::primacy`], not
    /// read from the registry. A source whose registry entry declares no primacy
    /// chain therefore derives [`SourceTier::Unoriginal`] — trust by omission is
    /// exactly the assumption the requirement forbids.
    pub tier: SourceTier,
    /// The hops that separate this source from the primary record, each naming
    /// its upstream and the primary document (the site's own policy or charter)
    /// that establishes the hop.
    pub primacy: PrimacyChain,
    /// The tier the registry *asserts*, when it asserts one. Kept only so the
    /// assertion can be checked against the derivation; nothing reads it to
    /// decide how much a source is worth.
    pub asserted_tier: Option<SourceTier>,
    /// API template with `{placeholder}` slots.
    pub api: String,
    /// The same source's own endpoint for every language `api` does not serve.
    ///
    /// Issue #1138, plan 01 L10. A project can publish its material through two
    /// surfaces: Wiktionary's content reaches English through the Free
    /// Dictionary API and every other language through that language edition's
    /// `MediaWiki` `extracts` API. Declaring the second endpoint on the same
    /// record — rather than as a second registry row — is what keeps one
    /// *service* occupying one of the `max_services` slots the bounds allow,
    /// which is what `max_services` counts.
    ///
    /// Empty when the source has one endpoint. When it is set, `api` serves the
    /// language `api_language` leads with and `language_api` serves the rest.
    pub language_api: String,
    /// License the retrieved bytes carry.
    pub license_name: String,
    /// Canonical URL of that license.
    pub license_url: String,
    /// Where captures of this source are cached.
    pub cache_path: String,
    /// Registry note explaining what the source is used for.
    pub note: String,
    /// The need kinds this source declares it may answer, in declared order.
    ///
    /// Issue #1138 B1, plan 01 L3: the second selection axis, and it is data.
    /// Before it, `how_to_role` was the only axis a selector had, so the four
    /// lexical sources — wikidata, wiktionary, wordnet, wikipedia — could not be
    /// reached by any walk: a dictionary has no procedural role, and the only
    /// live selector in the tree filtered on exactly that. A source now says
    /// which questions it answers.
    pub need_kinds: Vec<NeedKind>,
    /// Which extractor reads this source's bytes (`wiktionary_entry_v1`, …).
    ///
    /// Registry data rather than a host-name `match` in Rust, so adding a source
    /// is a seed edit plus one extractor function.
    pub extractor: String,
    /// Language codes the endpoint actually serves, in declared order.
    ///
    /// Empty means the template carries no `{language}` slot. A lookup in a
    /// language a source does not serve is reported `unbound_template`, never
    /// silently answered in English.
    pub api_language: Vec<String>,
    /// Position in `data/seed/sources-registry.lino`; the declared order *is*
    /// the consultation order within one tier, so ordering is data.
    pub registry_index: usize,
}

impl SourceRecord {
    /// The need kinds this source declares it may answer.
    #[must_use]
    pub fn need_kinds(&self) -> &[NeedKind] {
        &self.need_kinds
    }

    /// Whether this source declares it may answer `kind`.
    ///
    /// [`NeedKind::None`] is never answered: a source that declares no kind is
    /// not a candidate for anything, which is what makes the axis data rather
    /// than a default.
    #[must_use]
    pub fn answers(&self, kind: NeedKind) -> bool {
        kind != NeedKind::None && self.need_kinds.contains(&kind)
    }

    /// Position in `data/seed/sources-registry.lino`.
    #[must_use]
    pub const fn registry_index(&self) -> usize {
        self.registry_index
    }

    /// Whether the endpoint serves `language`.
    ///
    /// A template with no `{language}` slot serves whatever it serves and is not
    /// language-bound, so it answers `true`; a template that names languages
    /// answers only for the ones it names.
    #[must_use]
    pub fn serves_language(&self, language: &str) -> bool {
        self.api_language.is_empty()
            || self
                .api_language
                .iter()
                .any(|declared| declared == language)
    }

    /// Whether this record is a live, settings-gated external service.
    #[must_use]
    pub fn is_external_trusted(&self) -> bool {
        self.service_group == EXTERNAL_TRUSTED_GROUP
    }

    /// Fill the API template's `{name}` slots with percent-encoded values.
    ///
    /// A placeholder without a supplied value stays literal so a caller can see
    /// exactly which slot it failed to bind rather than silently requesting a
    /// malformed URL.
    #[must_use]
    pub fn api_url(&self, parameters: &[(&str, &str)]) -> String {
        Self::bind_template(&self.api, parameters)
    }

    /// The endpoint that serves `language`, template unbound.
    ///
    /// The primary `api` for the language `api_language` leads with, and
    /// [`SourceRecord::language_api`] for every other declared language when the
    /// source publishes a second endpoint. One service, two published surfaces
    /// (issue #1138, plan 01 L10).
    #[must_use]
    pub fn api_template_for(&self, language: &str) -> &str {
        if self.language_api.is_empty() || language.is_empty() {
            return &self.api;
        }
        let primary = self.api_language.first().map_or("", String::as_str);
        if language == primary {
            &self.api
        } else {
            &self.language_api
        }
    }

    /// Fill the slots of the endpoint that serves `language`.
    #[must_use]
    pub fn api_url_in(&self, language: &str, parameters: &[(&str, &str)]) -> String {
        Self::bind_template(self.api_template_for(language), parameters)
    }

    fn bind_template(template: &str, parameters: &[(&str, &str)]) -> String {
        let mut url = template.to_owned();
        for (name, value) in parameters {
            url = url.replace(&format!("{{{name}}}"), &percent_encode(value));
        }
        url
    }

    /// Host of the API endpoint, used as the source label in evidence.
    #[must_use]
    pub fn host(&self) -> &str {
        self.api
            .split_once("://")
            .map_or(self.api.as_str(), |(_, rest)| {
                rest.split('/').next().unwrap_or(rest)
            })
    }
}

/// The members of a Links Notation inline list, written `(a b c)` or bare.
///
/// The parser hands the value back with its parentheses, because the notation
/// does not distinguish a one-element list from a scalar; a reader that wants
/// members asks for members.
fn seed_list(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split_whitespace()
        .map(|member| member.trim_matches('"').to_owned())
        .filter(|member| !member.is_empty())
        .collect()
}

/// Every declared source, in registry order.
#[must_use]
pub fn source_registry() -> Vec<SourceRecord> {
    let tree = parse_lino(SOURCES_REGISTRY_LINO);
    let mut records = Vec::new();
    for root in tree
        .children
        .iter()
        .filter(|node| node.name == "sources_registry")
    {
        for entry in root.children.iter().filter(|node| node.name == "source") {
            let primacy = chain_from_node(entry);
            let registry_index = records.len();
            records.push(SourceRecord {
                id: entry.id.clone(),
                name: entry.find_child_value("name").to_owned(),
                kind: entry.find_child_value("kind").to_owned(),
                service_group: entry.find_child_value("service_group").to_owned(),
                settings_key: entry.find_child_value("settings_key").to_owned(),
                default_enabled: entry.find_child_value("default_enabled") != "false",
                how_to_role: HowToRole::from_seed(entry.find_child_value("how_to_role")),
                tier: primacy.derive_tier(),
                primacy,
                asserted_tier: tier_from_slug(entry.find_child_value("source_tier")),
                api: entry.find_child_value("api").to_owned(),
                language_api: entry.find_child_value("language_api").to_owned(),
                license_name: entry.find_child_value("license_name").to_owned(),
                license_url: entry.find_child_value("license_url").to_owned(),
                cache_path: entry.find_child_value("cache_path").to_owned(),
                note: entry.find_child_value("note").to_owned(),
                need_kinds: seed_list(entry.find_child_value("need_kinds"))
                    .into_iter()
                    .map(|slug| NeedKind::from_seed(&slug))
                    .filter(|kind| *kind != NeedKind::None)
                    .collect(),
                extractor: entry.find_child_value("extractor").to_owned(),
                api_language: seed_list(entry.find_child_value("api_language")),
                registry_index,
            });
        }
    }
    records
}

/// Only the live, settings-gated services, in registry order.
#[must_use]
pub fn external_trusted_sources() -> Vec<SourceRecord> {
    source_registry()
        .into_iter()
        .filter(SourceRecord::is_external_trusted)
        .collect()
}

/// Every source that declares `kind`, in consultation order: derived tier
/// descending, then registry order. No Rust code contains a source list.
///
/// The dictionary to lexicon to encyclopedia to technical progression issue
/// #1138 B1 asks for is not a new Rust enum: `wikidata`, `wiktionary`,
/// `wordnet` and `wikipedia` already sit at the head of
/// `data/seed/sources-registry.lino`, ahead of `wikihow`, `stackexchange` and
/// `github`, and declaring `need_kinds` on them yields exactly that order from
/// the file as written.
#[must_use]
pub fn sources_for_need_kind(kind: NeedKind) -> Vec<SourceRecord> {
    let mut selected: Vec<SourceRecord> = source_registry()
        .into_iter()
        .filter(|record| record.answers(kind))
        .collect();
    selected.sort_by_key(|record| {
        (
            u8::MAX - record.tier.weight_percent(),
            record.registry_index,
        )
    });
    selected
}

/// Look one source up by registry id.
#[must_use]
pub fn source_record(id: &str) -> Option<SourceRecord> {
    source_registry().into_iter().find(|record| record.id == id)
}

/// Every distinct settings key that can opt a service out, in registry order.
#[must_use]
pub fn external_service_settings_keys() -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for record in external_trusted_sources() {
        if !record.settings_key.is_empty() && !keys.contains(&record.settings_key) {
            keys.push(record.settings_key);
        }
    }
    keys
}

/// Percent-encode a query/path parameter with the unreserved set from RFC 3986.
#[must_use]
pub fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}
