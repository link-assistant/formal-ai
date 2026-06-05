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

/// Issue #242: dictionary pages such as Cambridge Dictionary are useful
/// evidence sources, but they do not expose unauthenticated CORS-readable
/// JSON endpoints. Keep them in the registry for diagnostics/proxy checks
/// without adding them to the live browser default plan.
#[test]
fn dictionary_sources_are_non_cors_knowledge_providers() {
    let plan = default_search_plan_ids();
    for id in [
        "cambridge-dictionary",
        "merriam-webster",
        "dictionary-com",
        "collins-dictionary",
    ] {
        let spec = WEB_SEARCH_PROVIDER_REGISTRY
            .iter()
            .find(|spec| spec.id == id)
            .unwrap_or_else(|| panic!("dictionary provider `{id}` missing from registry"));
        assert!(matches!(spec.category, ProviderCategory::Knowledge));
        assert!(
            !spec.cors_readable,
            "dictionary page provider `{id}` must stay proxy/diagnostics-only"
        );
        assert!(
            !spec.default_for_category,
            "dictionary page provider `{id}` must not replace the live CORS default"
        );
        assert!(
            !plan.contains(&id.to_string()),
            "dictionary page provider `{id}` must not enter the default CORS plan"
        );
    }
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
    let expected = 1.0_f64 / (f64::from(WEB_SEARCH_RRF_K) + 1.0);
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
