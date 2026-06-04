use super::*;

#[test]
fn loads_hive_mind_project() {
    let registry = projects_registry();
    let hive = registry
        .by_alias("Hive Mind")
        .expect("hive-mind project must be present");
    assert_eq!(hive.repo_slug(), "link-assistant/hive-mind");
    assert_eq!(hive.url, "https://github.com/link-assistant/hive-mind");
    assert!(!hive.statements.is_empty());
    let purpose = hive
        .statements
        .iter()
        .find(|s| s.kind == "purpose")
        .expect("hive-mind needs a purpose statement");
    assert!(purpose.text.contains("AI"));
}

#[test]
fn alias_lookup_is_case_insensitive() {
    let registry = projects_registry();
    let by_lower = registry.by_alias("hive mind").map(|p| p.slug.clone());
    let by_upper = registry.by_alias("Hive Mind").map(|p| p.slug.clone());
    let by_compact = registry.by_alias("hivemind").map(|p| p.slug.clone());
    assert_eq!(
        by_lower,
        Some("project_link_assistant_hive_mind".to_owned())
    );
    assert_eq!(by_lower, by_upper);
    assert_eq!(by_lower, by_compact);
}

#[test]
fn alias_lookup_handles_hyphen_variants() {
    let registry = projects_registry();
    let with_hyphen = registry.by_alias("hive-mind").map(|p| p.slug.clone());
    let with_underscore = registry.by_alias("hive_mind").map(|p| p.slug.clone());
    assert!(with_hyphen.is_some());
    assert_eq!(with_hyphen, with_underscore);
}

#[test]
fn by_org_returns_only_matching_org() {
    let registry = projects_registry();
    let assistant = registry.by_org("link-assistant");
    let foundation = registry.by_org("link-foundation");
    assert!(!assistant.is_empty());
    assert!(!foundation.is_empty());
    assert!(assistant.iter().all(|p| p.org == "link-assistant"));
    assert!(foundation.iter().all(|p| p.org == "link-foundation"));
}

#[test]
fn localized_russian_overrides_statements_when_present() {
    let registry = projects_registry();
    let hive = registry.by_alias("hive mind").expect("hive-mind present");
    let ru_statements = hive.statements_for("ru");
    assert!(!ru_statements.is_empty());
    assert!(ru_statements
        .iter()
        .any(|s| s.text.contains("ИИ") || s.text.contains("Hive Mind")));
}

#[test]
fn unknown_language_falls_back_to_default() {
    let registry = projects_registry();
    let hive = registry.by_alias("hive mind").expect("hive-mind present");
    // hindi / chinese aren't defined for this entry: default applies
    let hi_statements = hive.statements_for("hi");
    assert_eq!(hi_statements.len(), hive.statements.len());
}

#[test]
fn every_project_carries_url_and_purpose() {
    let registry = projects_registry();
    assert!(
        registry.projects.len() >= 10,
        "expected curated registry of at least 10 projects (got {})",
        registry.projects.len()
    );
    for project in &registry.projects {
        assert!(
            project.url.starts_with("https://github.com/"),
            "{} must point to a GitHub URL",
            project.slug
        );
        assert!(
            project
                .statements
                .iter()
                .any(|s| s.kind == "purpose" && !s.text.is_empty()),
            "{} missing purpose statement",
            project.slug
        );
    }
}
