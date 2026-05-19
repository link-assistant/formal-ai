//! Shared symbolic core for the multi-engine `web_search` planner.
//!
//! Every reasoning surface — CLI, server, browser worker, and the Rust→WASM
//! port mounted in `src/web/wasm-worker/` — consumes the same provider list,
//! the same Reciprocal Rank Fusion constant, and the same per-category
//! concurrency cap. This module owns the contract.
//!
//! Issue #133 wants the browser to call into the Rust core for these
//! primitives so the offline event log and the live answer agree on every
//! `web_search:*` event. The module is `no_std` + `alloc` compatible so it
//! can be `#[path = ...]`-included by the WASM crate without dragging in
//! the standard library.

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Reciprocal Rank Fusion constant used to merge per-provider rankings.
///
/// `k = 60` is the value Cormack, Clarke, and Buettcher use in the 2009 TREC
/// submissions where RRF was introduced. The formula is
/// `score(d) = Σ 1 / (k + rank_i(d))` summed across the providers that
/// returned `d`.
///
/// Source: <https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf>
pub const WEB_SEARCH_RRF_K: u32 = 60;

/// Maximum number of providers fired concurrently inside a single category.
///
/// Modern browsers cap per-origin sockets at six, so five keeps a slot free
/// for the rest of the page. The CLI and the connectivity dashboard mirror
/// the same cap so test traces stay deterministic across surfaces.
pub const WEB_SEARCH_CONCURRENCY_PER_CATEGORY: u32 = 5;

/// Top-N results consumed from each provider before fusion. `10` matches the
/// "top-10 per provider" wording in the issue description.
pub const WEB_SEARCH_PROVIDER_LIMIT: u32 = 10;

/// Coarse category buckets used to schedule providers in parallel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderCategory {
    Search,
    Knowledge,
    Papers,
    Code,
}

impl ProviderCategory {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::Knowledge => "knowledge",
            Self::Papers => "papers",
            Self::Code => "code",
        }
    }
}

/// Static description of a single provider.
#[derive(Debug, Clone, Copy)]
pub struct ProviderSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub category: ProviderCategory,
    /// `true` when the provider exposes a CORS-readable JSON endpoint that
    /// the browser worker can call directly. `false` providers still appear
    /// in the connectivity dashboard but are skipped during automated fusion.
    pub cors_readable: bool,
    /// `true` for the canonical default in each category (e.g. `DuckDuckGo`
    /// for search). The default is always the first plan entry for its bucket.
    pub default_for_category: bool,
}

/// Master provider list.
///
/// Order is significant: defaults come first inside each category so the
/// JS side can preserve the issue's "`DuckDuckGo` first" guarantee without
/// re-sorting. CORS-readable providers participate in fusion; non-CORS
/// providers participate in the connectivity dashboard
/// (`src/web/tests/connectivity.js`) and are recorded as evidence even
/// when the live worker cannot call them.
pub const WEB_SEARCH_PROVIDER_REGISTRY: &[ProviderSpec] = &[
    ProviderSpec {
        id: "duckduckgo",
        label: "DuckDuckGo Instant Answer",
        category: ProviderCategory::Search,
        cors_readable: true,
        default_for_category: true,
    },
    ProviderSpec {
        id: "google",
        label: "Google Search",
        category: ProviderCategory::Search,
        cors_readable: false,
        default_for_category: false,
    },
    ProviderSpec {
        id: "bing",
        label: "Bing Search",
        category: ProviderCategory::Search,
        cors_readable: false,
        default_for_category: false,
    },
    ProviderSpec {
        id: "brave",
        label: "Brave Search",
        category: ProviderCategory::Search,
        cors_readable: false,
        default_for_category: false,
    },
    ProviderSpec {
        id: "yahoo",
        label: "Yahoo Search",
        category: ProviderCategory::Search,
        cors_readable: false,
        default_for_category: false,
    },
    ProviderSpec {
        id: "yandex",
        label: "Yandex Search",
        category: ProviderCategory::Search,
        cors_readable: false,
        default_for_category: false,
    },
    ProviderSpec {
        id: "ecosia",
        label: "Ecosia",
        category: ProviderCategory::Search,
        cors_readable: false,
        default_for_category: false,
    },
    ProviderSpec {
        id: "mojeek",
        label: "Mojeek",
        category: ProviderCategory::Search,
        cors_readable: false,
        default_for_category: false,
    },
    ProviderSpec {
        id: "startpage",
        label: "Startpage",
        category: ProviderCategory::Search,
        cors_readable: false,
        default_for_category: false,
    },
    ProviderSpec {
        id: "wikipedia",
        label: "Wikipedia REST",
        category: ProviderCategory::Knowledge,
        cors_readable: true,
        default_for_category: true,
    },
    ProviderSpec {
        id: "wikidata",
        label: "Wikidata entities",
        category: ProviderCategory::Knowledge,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "wiktionary",
        label: "Wiktionary opensearch",
        category: ProviderCategory::Knowledge,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "internet-archive",
        label: "Internet Archive (archive.org)",
        category: ProviderCategory::Knowledge,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "dbpedia",
        label: "DBpedia Lookup",
        category: ProviderCategory::Knowledge,
        cors_readable: false,
        default_for_category: false,
    },
    ProviderSpec {
        id: "openlibrary",
        label: "Open Library",
        category: ProviderCategory::Knowledge,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "openalex",
        label: "OpenAlex works",
        category: ProviderCategory::Knowledge,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "crossref",
        label: "Crossref works",
        category: ProviderCategory::Knowledge,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "semantic-scholar",
        label: "Semantic Scholar",
        category: ProviderCategory::Knowledge,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "arxiv",
        label: "arXiv atom export",
        category: ProviderCategory::Papers,
        cors_readable: true,
        default_for_category: true,
    },
    ProviderSpec {
        id: "europepmc",
        label: "Europe PMC",
        category: ProviderCategory::Papers,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "doaj",
        label: "DOAJ articles",
        category: ProviderCategory::Papers,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "github",
        label: "GitHub repositories",
        category: ProviderCategory::Code,
        cors_readable: true,
        default_for_category: true,
    },
    ProviderSpec {
        id: "gitlab",
        label: "GitLab projects",
        category: ProviderCategory::Code,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "codeberg",
        label: "Codeberg",
        category: ProviderCategory::Code,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "gitee",
        label: "Gitee",
        category: ProviderCategory::Code,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "bitbucket",
        label: "Bitbucket Cloud",
        category: ProviderCategory::Code,
        cors_readable: true,
        default_for_category: false,
    },
    ProviderSpec {
        id: "gitflic",
        label: "GitFlic",
        category: ProviderCategory::Code,
        cors_readable: false,
        default_for_category: false,
    },
];

/// Provider ids that participate in live RRF fusion in the browser worker.
/// These are the CORS-readable subset of [`WEB_SEARCH_PROVIDER_REGISTRY`].
///
/// Issue #180 expands the default plan to also include Internet Archive and
/// Wiktionary, in the priority order requested in the issue body
/// (`DuckDuckGo` → Internet Archive → Wikipedia → Wikidata → Wiktionary).
/// The remaining providers in the registry still feed the connectivity
/// dashboard and the case study.
pub const WEB_SEARCH_PROVIDERS: &[&str] = &[
    "duckduckgo",
    "internet-archive",
    "wikipedia",
    "wikidata",
    "wiktionary",
];

/// Default plan id list returned to JS. JS uses this to seed the planner
/// even when the live `fetch()` is offline.
#[must_use]
pub fn default_search_plan_ids() -> Vec<String> {
    WEB_SEARCH_PROVIDERS
        .iter()
        .map(|id| (*id).to_string())
        .collect()
}

/// Build the `web_search:*` evidence prefix for a given query/language.
///
/// The browser worker appends per-provider rank lines after these prefixes;
/// the offline solver records the same prefixes through [`EventLog`].
#[must_use]
pub fn build_request_evidence(query: &str, language: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("web_search:request:{query}"));
    if !language.is_empty() {
        lines.push(format!("web_search:language:{language}"));
    }
    for provider in WEB_SEARCH_PROVIDERS {
        lines.push(format!("web_search:provider:{provider}"));
    }
    lines.push(format!("web_search:combined:rrf:k={WEB_SEARCH_RRF_K}"));
    lines
}

/// Per-provider ranked entry consumed by [`reciprocal_rank_fusion`].
///
/// `provider_id` is the registry id (e.g. `"duckduckgo"`), `rank` is the
/// 1-based position the provider returned for the URL, and `title`/`excerpt`
/// preserve the human-readable metadata for the first appearance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRanking {
    pub provider_id: String,
    pub rank: u32,
    pub url: String,
    pub title: String,
    pub excerpt: String,
}

/// Single fused row produced by [`reciprocal_rank_fusion`].
///
/// `score` is the RRF sum; `providers` lists the contributing
/// `provider#rank` pairs in insertion order so the JS side can render
/// "via duckduckgo#3, wikipedia#1" inline.
#[derive(Debug, Clone, PartialEq)]
pub struct FusedEntry {
    pub url: String,
    pub title: String,
    pub excerpt: String,
    pub score: f64,
    pub providers: Vec<(String, u32)>,
}

/// Compute the reciprocal-rank-fusion ranking for a flat list of provider
/// rows. Equivalent to the JS implementation in `formal_ai_worker.js` and
/// the offline trace in `web_requests.rs`.
#[must_use]
pub fn reciprocal_rank_fusion(entries: &[ProviderRanking], k: u32) -> Vec<FusedEntry> {
    let mut fused: Vec<FusedEntry> = Vec::new();
    for entry in entries {
        if entry.url.is_empty() {
            continue;
        }
        let denom = f64::from(k) + f64::from(entry.rank);
        if denom <= 0.0 {
            continue;
        }
        let score = 1.0 / denom;
        if let Some(existing) = fused.iter_mut().find(|item| item.url == entry.url) {
            existing.score += score;
            existing
                .providers
                .push((entry.provider_id.clone(), entry.rank));
            if existing.title.is_empty() && !entry.title.is_empty() {
                existing.title.clone_from(&entry.title);
            }
            if existing.excerpt.is_empty() && !entry.excerpt.is_empty() {
                existing.excerpt.clone_from(&entry.excerpt);
            }
        } else {
            let providers: Vec<(String, u32)> = vec![(entry.provider_id.clone(), entry.rank)];
            fused.push(FusedEntry {
                url: entry.url.clone(),
                title: if entry.title.is_empty() {
                    entry.url.clone()
                } else {
                    entry.title.clone()
                },
                excerpt: entry.excerpt.clone(),
                score,
                providers,
            });
        }
    }
    fused.sort_by(|left, right| match right.score.partial_cmp(&left.score) {
        Some(Ordering::Equal) | None => right.providers.len().cmp(&left.providers.len()),
        Some(order) => order,
    });
    fused
}

/// Parse the line-delimited byte buffer the WASM worker exchanges with JS.
///
/// Each line is `provider_id\trank\turl\ttitle\texcerpt`. Empty lines are
/// skipped; malformed lines are dropped silently so the JS fallback can
/// recover even when one provider returns junk.
#[must_use]
pub fn parse_rrf_input(input: &str) -> Vec<ProviderRanking> {
    let mut entries: Vec<ProviderRanking> = Vec::new();
    for line in input.split('\n') {
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let Some(provider_id) = parts.next() else {
            continue;
        };
        let Some(rank_text) = parts.next() else {
            continue;
        };
        let Some(url) = parts.next() else {
            continue;
        };
        let title = parts.next().unwrap_or("");
        let excerpt = parts.next().unwrap_or("");
        let Ok(rank) = rank_text.parse::<u32>() else {
            continue;
        };
        if url.is_empty() {
            continue;
        }
        entries.push(ProviderRanking {
            provider_id: provider_id.to_string(),
            rank,
            url: url.to_string(),
            title: title.to_string(),
            excerpt: excerpt.to_string(),
        });
    }
    entries
}

/// Serialize the fused result list back to the JS side. The format mirrors
/// `parse_rrf_input` for symmetry: one row per line, tab-separated, with
/// the `providers` field encoded as `id#rank` joined by `+`.
#[must_use]
pub fn serialize_rrf_output(fused: &[FusedEntry]) -> String {
    let mut buffer = String::new();
    for entry in fused {
        if !buffer.is_empty() {
            buffer.push('\n');
        }
        buffer.push_str(&entry.url);
        buffer.push('\t');
        buffer.push_str(&entry.title);
        buffer.push('\t');
        buffer.push_str(&entry.excerpt);
        buffer.push('\t');
        buffer.push_str(&format_score(entry.score));
        buffer.push('\t');
        for (index, (provider_id, rank)) in entry.providers.iter().enumerate() {
            if index > 0 {
                buffer.push('+');
            }
            buffer.push_str(provider_id);
            buffer.push('#');
            buffer.push_str(&rank.to_string());
        }
    }
    buffer
}

/// Format an RRF score with six decimal places — enough precision to keep
/// the JS side's tie-break behaviour identical without depending on `std`'s
/// formatter quirks. The `no_std` WASM target does not link the `round`
/// intrinsic, so we do the round-half-away-from-zero conversion by hand.
fn format_score(score: f64) -> String {
    let scaled_f = score * 1_000_000.0;
    let scaled = if scaled_f >= 0.0 {
        (scaled_f + 0.5) as i64
    } else {
        (scaled_f - 0.5) as i64
    };
    let whole = scaled / 1_000_000;
    let fraction = (scaled % 1_000_000).abs();
    let mut text = String::new();
    if scaled < 0 && whole == 0 {
        text.push('-');
    }
    text.push_str(&whole.to_string());
    text.push('.');
    let fraction_str = fraction.to_string();
    for _ in 0..(6 - fraction_str.len()) {
        text.push('0');
    }
    text.push_str(&fraction_str);
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_k_is_sixty() {
        assert_eq!(WEB_SEARCH_RRF_K, 60);
    }

    #[test]
    fn default_plan_lists_duckduckgo_first() {
        let plan = default_search_plan_ids();
        assert_eq!(plan.first().map(String::as_str), Some("duckduckgo"));
        assert!(plan.contains(&"wikipedia".to_string()));
        assert!(plan.contains(&"wikidata".to_string()));
        assert!(plan.contains(&"wiktionary".to_string()));
        assert!(plan.contains(&"internet-archive".to_string()));
    }

    /// Issue #180 specifies the strict priority order the JS worker uses
    /// when rendering and deduping fused results. Pin it here so the WASM
    /// evidence prefix stays in lockstep with the JS rendering.
    #[test]
    fn default_plan_preserves_issue_180_priority_order() {
        let plan = default_search_plan_ids();
        assert_eq!(
            plan,
            vec![
                "duckduckgo".to_string(),
                "internet-archive".to_string(),
                "wikipedia".to_string(),
                "wikidata".to_string(),
                "wiktionary".to_string(),
            ]
        );
    }

    #[test]
    fn registry_includes_all_four_categories() {
        let mut search = 0;
        let mut knowledge = 0;
        let mut papers = 0;
        let mut code = 0;
        for spec in WEB_SEARCH_PROVIDER_REGISTRY {
            match spec.category {
                ProviderCategory::Search => search += 1,
                ProviderCategory::Knowledge => knowledge += 1,
                ProviderCategory::Papers => papers += 1,
                ProviderCategory::Code => code += 1,
            }
        }
        assert!(search >= 7, "expected ≥7 search providers, found {search}");
        assert!(
            knowledge >= 6,
            "expected ≥6 knowledge providers, found {knowledge}"
        );
        assert!(papers >= 3, "expected ≥3 papers providers, found {papers}");
        assert!(code >= 5, "expected ≥5 code providers, found {code}");
    }

    #[test]
    fn build_request_evidence_includes_combined_ranking_line() {
        let lines = build_request_evidence("formal-ai", "en");
        assert_eq!(
            lines.first().map(String::as_str),
            Some("web_search:request:formal-ai")
        );
        assert!(lines.contains(&"web_search:language:en".to_string()));
        assert!(lines.contains(&"web_search:provider:duckduckgo".to_string()));
        assert_eq!(
            lines.last().map(String::as_str),
            Some("web_search:combined:rrf:k=60")
        );
    }

    #[test]
    fn reciprocal_rank_fusion_combines_shared_urls() {
        let entries = [
            ProviderRanking {
                provider_id: "duckduckgo".to_string(),
                rank: 1,
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
                excerpt: "DDG".to_string(),
            },
            ProviderRanking {
                provider_id: "wikipedia".to_string(),
                rank: 2,
                url: "https://example.com".to_string(),
                title: "Example".to_string(),
                excerpt: "Wiki".to_string(),
            },
            ProviderRanking {
                provider_id: "wikidata".to_string(),
                rank: 1,
                url: "https://other.example".to_string(),
                title: "Other".to_string(),
                excerpt: String::new(),
            },
        ];
        let fused = reciprocal_rank_fusion(&entries, WEB_SEARCH_RRF_K);
        assert_eq!(fused.len(), 2);
        assert_eq!(fused[0].url, "https://example.com");
        assert_eq!(fused[0].providers.len(), 2);
    }

    #[test]
    fn rrf_input_round_trips_through_parser_and_serializer() {
        let input = "duckduckgo\t1\thttps://example.com\tExample\tDDG\n\
                     wikipedia\t2\thttps://example.com\tExample\tWiki";
        let entries = parse_rrf_input(input);
        assert_eq!(entries.len(), 2);
        let fused = reciprocal_rank_fusion(&entries, WEB_SEARCH_RRF_K);
        let output = serialize_rrf_output(&fused);
        assert!(output.contains("duckduckgo#1+wikipedia#2"));
        assert!(output.starts_with("https://example.com"));
    }

    #[test]
    fn format_score_pads_fraction_to_six_digits() {
        assert_eq!(format_score(0.000_032_786_885_245_901_64), "0.000033");
        assert_eq!(format_score(1.5), "1.500000");
    }

    /// Issue #133 explicitly enumerates the providers each category must
    /// cover. Pin the ids here so a future refactor cannot quietly drop one.
    #[test]
    fn registry_pins_issue_133_explicit_providers() {
        let ids: Vec<&str> = WEB_SEARCH_PROVIDER_REGISTRY
            .iter()
            .map(|spec| spec.id)
            .collect();
        for required in [
            // Search providers called out in the issue body.
            "duckduckgo",
            "google",
            "bing",
            "brave",
            "yahoo",
            "yandex",
            "ecosia",
            "mojeek",
            "startpage",
            // Knowledge providers the issue asks for beyond Wikipedia/Wikidata.
            "wikipedia",
            "wikidata",
            "wiktionary",
            "internet-archive",
            // Open-access paper providers (no paywall, as the issue requires).
            "arxiv",
            "europepmc",
            "doaj",
            // Code hosts including Chinese and Russian ones from the issue.
            "github",
            "gitlab",
            "codeberg",
            "gitee",
            "bitbucket",
            "gitflic",
        ] {
            assert!(
                ids.contains(&required),
                "registry must list `{required}` (issue #133)"
            );
        }
    }

    #[test]
    fn cors_readable_defaults_are_consistent_with_default_plan() {
        let plan = default_search_plan_ids();
        for id in &plan {
            let spec = WEB_SEARCH_PROVIDER_REGISTRY
                .iter()
                .find(|spec| spec.id == id.as_str())
                .unwrap_or_else(|| panic!("plan provider `{id}` missing from registry"));
            assert!(
                spec.cors_readable,
                "default-plan provider `{id}` must be CORS-readable"
            );
        }
    }

    /// Issue #180: the evidence prefix must list providers in the same order
    /// as the JS worker so the WASM-derived prefix matches what `tryWebSearch`
    /// would emit when it falls back to its inline list. Without this the
    /// browser would render the providers in a different order than the
    /// canonical Rust core.
    #[test]
    fn build_request_evidence_lists_providers_in_priority_order() {
        let lines = build_request_evidence("query", "en");
        let provider_lines: Vec<&str> = lines
            .iter()
            .filter(|line| line.starts_with("web_search:provider:"))
            .map(String::as_str)
            .collect();
        assert_eq!(
            provider_lines,
            vec![
                "web_search:provider:duckduckgo",
                "web_search:provider:internet-archive",
                "web_search:provider:wikipedia",
                "web_search:provider:wikidata",
                "web_search:provider:wiktionary",
            ]
        );
    }

    /// Issue #180: when a language is empty the evidence prefix must still
    /// produce well-formed lines and must not emit `web_search:language:` with
    /// a trailing empty value.
    #[test]
    fn build_request_evidence_skips_empty_language_line() {
        let lines = build_request_evidence("query", "");
        assert!(!lines
            .iter()
            .any(|line| line == "web_search:language:" || line == "web_search:language: "));
    }

    /// Issue #180: Internet Archive is listed in the default plan and tagged
    /// as CORS-readable so the browser can hit it without a proxy.
    #[test]
    fn internet_archive_is_cors_readable_in_registry() {
        let spec = WEB_SEARCH_PROVIDER_REGISTRY
            .iter()
            .find(|spec| spec.id == "internet-archive")
            .expect("internet-archive must be in registry");
        assert!(
            spec.cors_readable,
            "internet-archive must stay CORS-readable so the demo browser can call it directly"
        );
        assert!(matches!(spec.category, ProviderCategory::Knowledge));
    }

    /// Issue #180: rendering depends on a stable RRF-tied score. Pin the
    /// formula `1 / (k + rank)` to k=60 so a regression in either k or the
    /// score function trips the test instead of silently shifting the rank
    /// order in the rendered list.
    #[test]
    fn rrf_score_matches_cormack_clarke_buettcher_formula() {
        let entries = [ProviderRanking {
            provider_id: "duckduckgo".to_string(),
            rank: 1,
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            excerpt: String::new(),
        }];
        let fused = reciprocal_rank_fusion(&entries, WEB_SEARCH_RRF_K);
        assert_eq!(fused.len(), 1);
        let expected = 1.0_f64 / (WEB_SEARCH_RRF_K as f64 + 1.0);
        assert!(
            (fused[0].score - expected).abs() < 1e-9,
            "expected score {expected}, got {}",
            fused[0].score
        );
    }

    /// Issue #180: every provider in the default plan must declare a label
    /// so the diagnostics panel can render a human-readable row instead of
    /// the raw id.
    #[test]
    fn default_plan_providers_carry_human_labels() {
        for id in &*default_search_plan_ids() {
            let spec = WEB_SEARCH_PROVIDER_REGISTRY
                .iter()
                .find(|spec| spec.id == id.as_str())
                .unwrap_or_else(|| panic!("plan id `{id}` missing from registry"));
            assert!(
                !spec.label.is_empty(),
                "plan provider `{id}` must have a non-empty label"
            );
        }
    }

    /// Issue #180: registry must include every provider in the default plan
    /// and the plan must only reference registered providers. Tightens the
    /// invariant from the cors-readable test so a typo can't slip through.
    #[test]
    fn default_plan_is_a_subset_of_registry_ids() {
        let registry_ids: Vec<&str> = WEB_SEARCH_PROVIDER_REGISTRY
            .iter()
            .map(|spec| spec.id)
            .collect();
        for id in &*default_search_plan_ids() {
            assert!(
                registry_ids.contains(&id.as_str()),
                "default-plan id `{id}` not present in registry"
            );
        }
    }
}
