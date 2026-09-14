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
    /// License the retrieved bytes carry.
    pub license_name: String,
    /// Canonical URL of that license.
    pub license_url: String,
    /// Where captures of this source are cached.
    pub cache_path: String,
    /// Registry note explaining what the source is used for.
    pub note: String,
}

impl SourceRecord {
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
        let mut url = self.api.clone();
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
                license_name: entry.find_child_value("license_name").to_owned(),
                license_url: entry.find_child_value("license_url").to_owned(),
                cache_path: entry.find_child_value("cache_path").to_owned(),
                note: entry.find_child_value("note").to_owned(),
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
