use super::super::{export_links_notation, MemoryEvent};
use super::{
    export_bundle, export_full_memory, extract_memory_from_bundle, import_full_memory,
    suggest_migrations, BundleInfo,
};
use std::collections::BTreeMap;

fn sample_events() -> Vec<MemoryEvent> {
    vec![
        MemoryEvent {
            id: String::from("1"),
            role: Some(String::from("user")),
            content: Some(String::from("Hi")),
            sent_at: Some(String::from("2026-05-15T12:00:00.000Z")),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("2"),
            role: Some(String::from("assistant")),
            intent: Some(String::from("greeting")),
            content: Some(String::from("Hi, how may I help you?")),
            sent_at: Some(String::from("2026-05-15T12:00:01.000Z")),
            ..MemoryEvent::default()
        },
    ]
}

#[test]
fn bundle_export_embeds_seed_and_memory() {
    let seed_files: Vec<(&str, &str)> = vec![("data/seed/example.lino", "example\n  key \"v\"")];
    let events = sample_events();
    let bundle = export_bundle(&seed_files, &events);
    assert!(bundle.starts_with("formal_ai_bundle\n"));
    assert!(bundle.contains("seed_files"));
    assert!(bundle.contains("data/seed/example.lino"));
    assert!(bundle.contains("demo_memory"));
    assert!(bundle.contains("Hi, how may I help you?"));
}

#[test]
fn extract_memory_from_bundle_recovers_events() {
    let seed_files: Vec<(&str, &str)> = vec![("data/seed/example.lino", "example\n  key \"v\"")];
    let events = sample_events();
    let bundle = export_bundle(&seed_files, &events);
    let recovered = extract_memory_from_bundle(&bundle).expect("recover");
    assert_eq!(recovered, events);
}

#[test]
fn full_memory_round_trip_preserves_seed_preferences_and_events() {
    let seed: Vec<(&str, &str)> = vec![
        (
            "data/seed/agent-info.lino",
            "agent_info\n  field \"version\"\n    value \"0.22.0\"\n",
        ),
        ("data/seed/example.lino", "example\n  key \"v\"\n"),
    ];
    let prefs: Vec<(&str, &str)> = vec![("demoMode", "off"), ("diagnosticsMode", "on")];
    let events = sample_events();
    let info = BundleInfo {
        version: Some(String::from("0.22.0")),
        url: Some(String::from("https://example.test/")),
        user_agent: Some(String::from("playwright/1.0")),
        worker_state: Some(String::from("wasm worker")),
        mode: Some(String::from("manual")),
        ..BundleInfo::default()
    };
    let bundle = export_full_memory(&seed, &events, &prefs, &info);
    assert!(bundle.starts_with("formal_ai_bundle\n"));
    assert!(bundle.contains("version \"0.22.0\""));
    assert!(bundle.contains("user_agent \"playwright/1.0\""));
    assert!(bundle.contains("seed_files"));
    assert!(bundle.contains("data/seed/agent-info.lino"));
    assert!(bundle.contains("preferences"));
    assert!(bundle.contains("demoMode \"off\""));
    assert!(bundle.contains("demo_memory"));
    let parsed = import_full_memory(&bundle);
    assert_eq!(parsed.events, events);
    assert_eq!(parsed.info.version.as_deref(), Some("0.22.0"));
    assert_eq!(parsed.info.mode.as_deref(), Some("manual"));
    assert_eq!(
        parsed.agent_info.get("version").map(String::as_str),
        Some("0.22.0")
    );
    let prefs_map: BTreeMap<String, String> = parsed.preferences.iter().cloned().collect();
    assert_eq!(prefs_map.get("demoMode").map(String::as_str), Some("off"));
    assert_eq!(
        prefs_map.get("diagnosticsMode").map(String::as_str),
        Some("on")
    );
    assert_eq!(parsed.seed_files.len(), 2);
}

#[test]
fn import_full_memory_accepts_legacy_demo_memory() {
    let text = export_links_notation(&sample_events());
    let parsed = import_full_memory(&text);
    assert_eq!(parsed.events, sample_events());
    assert!(parsed.seed_files.is_empty());
    assert!(parsed.preferences.is_empty());
    assert!(parsed.agent_info.is_empty());
}

#[test]
fn suggest_migrations_flags_seed_version_drift() {
    let seed: Vec<(&str, &str)> = vec![(
        "data/seed/agent-info.lino",
        "agent_info\n  field \"version\"\n    value \"0.21.0\"\n",
    )];
    let bundle = export_full_memory(&seed, &sample_events(), &[], &BundleInfo::default());
    let imported = import_full_memory(&bundle);
    let mut current = BTreeMap::new();
    current.insert(String::from("version"), String::from("0.22.0"));
    let suggestions = suggest_migrations(&imported, &current);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].contains("0.21.0"));
    assert!(suggestions[0].contains("0.22.0"));
}

#[test]
fn suggest_migrations_flags_legacy_only_import() {
    let imported = import_full_memory(&export_links_notation(&sample_events()));
    let mut current = BTreeMap::new();
    current.insert(String::from("version"), String::from("0.22.0"));
    let suggestions = suggest_migrations(&imported, &current);
    assert!(suggestions
        .iter()
        .any(|message| message.contains("legacy demo_memory")));
}

#[test]
fn suggest_migrations_is_quiet_when_versions_match() {
    let seed: Vec<(&str, &str)> = vec![(
        "data/seed/agent-info.lino",
        "agent_info\n  field \"version\"\n    value \"0.22.0\"\n",
    )];
    let bundle = export_full_memory(&seed, &sample_events(), &[], &BundleInfo::default());
    let imported = import_full_memory(&bundle);
    let mut current = BTreeMap::new();
    current.insert(String::from("version"), String::from("0.22.0"));
    assert!(suggest_migrations(&imported, &current).is_empty());
}
