use super::{
    default_native_link_store, memory_event_to_link_record, selected_link_store_backend,
    validate_memory_links_notation, LinkStore, LinkStoreBackend, LinkStoreError,
};
use crate::memory::{export_links_notation, MemoryEvent, MemoryStore};

#[test]
fn memory_events_reduce_to_type_subtype_value_doublets() {
    let record = memory_event_to_link_record(&MemoryEvent::user("hello"), 0);
    assert_eq!(record.record_type, "MemoryEvent");
    assert!(record
        .links
        .iter()
        .any(|link| link.from == "Type" && link.to == "MemoryEvent"));
    assert!(record
        .links
        .iter()
        .any(|link| link.from == "SubType" && link.to == "user"));
    assert!(record
        .links
        .iter()
        .any(|link| link.from == "field:content" && link.to == "value:hello"));
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
#[test]
fn native_default_build_selects_doublets_rs_backend() {
    assert_eq!(selected_link_store_backend(), LinkStoreBackend::DoubletsRs);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "doublets-native")))]
#[test]
fn native_without_default_features_falls_back_to_lino_projection() {
    assert_eq!(
        selected_link_store_backend(),
        LinkStoreBackend::LinoProjection
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn default_native_link_store_matches_selected_backend() {
    let mut store = default_native_link_store().expect("default link store");
    assert_eq!(store.backend(), selected_link_store_backend());
    let id = store
        .append_memory_event(MemoryEvent::user("hello from default store"))
        .expect("append");
    assert!(id.starts_with("memory_event_"));
    assert_eq!(store.records().len(), 1);
    assert!(store.export_memory_links_notation().contains("demo_memory"));
}

#[test]
fn memory_store_trait_assigns_stable_ids_and_exports_lino() {
    let mut store = MemoryStore::new();
    let id =
        LinkStore::append_memory_event(&mut store, MemoryEvent::user("hello")).expect("append");
    assert!(id.starts_with("memory_event_"));
    assert_eq!(store.backend(), LinkStoreBackend::LinoProjection);
    assert!(store.export_memory_links_notation().contains("demo_memory"));
    assert_eq!(store.records().len(), 1);
}

#[test]
fn strict_import_rejects_ill_formed_links_notation_without_mutation() {
    let mut store = MemoryStore::new();
    let err = store
        .try_import_links_notation("demo_memory\n  event \"unterminated\n")
        .expect_err("malformed import must fail");
    assert!(matches!(err, LinkStoreError::IllFormedLinksNotation(_)));
    assert!(store.is_empty());
}

#[test]
fn strict_import_accepts_legacy_memory_documents() {
    let text = export_links_notation(&[MemoryEvent {
        id: String::from("event_1"),
        role: Some(String::from("user")),
        content: Some(String::from("Hi")),
        ..MemoryEvent::default()
    }]);
    validate_memory_links_notation(&text).expect("valid memory document");
    let mut store = MemoryStore::new();
    let inserted = store.try_import_links_notation(&text).expect("import");
    assert_eq!(inserted, 1);
    assert_eq!(store.events()[0].id, "event_1");
}

#[cfg(feature = "doublets-native")]
#[test]
fn doublets_default_imports_full_lino_bundle_and_exports_deterministically() {
    use crate::memory::{export_full_memory, BundleInfo};

    let events = vec![
        MemoryEvent {
            id: String::from("legacy_user_1"),
            role: Some(String::from("user")),
            content: Some(String::from("Hi from an existing bundle")),
            sent_at: Some(String::from("2026-05-26T00:00:00.000Z")),
            ..MemoryEvent::default()
        },
        MemoryEvent {
            id: String::from("legacy_assistant_1"),
            role: Some(String::from("assistant")),
            intent: Some(String::from("greeting")),
            content: Some(String::from("Hi, how may I help you?")),
            sent_at: Some(String::from("2026-05-26T00:00:01.000Z")),
            evidence: vec![String::from("intent:greeting")],
            ..MemoryEvent::default()
        },
    ];
    let seed = [(
        "data/seed/agent-info.lino",
        "agent_info\n  version \"0.1.0\"\n",
    )];
    let bundle = export_full_memory(
        &seed,
        &events,
        &[],
        &BundleInfo {
            exported_at: Some(String::from("2026-05-26T00:00:02.000Z")),
            version: Some(String::from("0.1.0")),
            ..BundleInfo::default()
        },
    );

    let mut store = default_native_link_store().expect("default native store");
    assert_eq!(store.backend(), LinkStoreBackend::DoubletsRs);

    let imported = store
        .import_memory_links_notation(&bundle)
        .expect("import existing bundle");

    assert_eq!(imported, events.len());
    assert_eq!(store.records().len(), events.len());
    assert_eq!(store.records()[0].source_id, "legacy_user_1");
    assert_eq!(store.records()[1].source_id, "legacy_assistant_1");
    assert!(
        store.native_link_count() > store.records()[0].links.len(),
        "imported bundle should be mirrored into raw native doublets"
    );
    assert_eq!(
        store.export_memory_links_notation(),
        export_links_notation(&events)
    );
}

#[cfg(feature = "doublets-native")]
#[test]
fn doublets_default_rejects_malformed_import_without_mutation() {
    let mut store = default_native_link_store().expect("default native store");
    store
        .append_memory_event(MemoryEvent::user("kept"))
        .expect("append");
    let before_export = store.export_memory_links_notation();
    let before_records = store.records();

    let err = store
        .import_memory_links_notation("demo_memory\n  event \"unterminated\n")
        .expect_err("malformed import must fail");

    assert!(matches!(err, LinkStoreError::IllFormedLinksNotation(_)));
    assert_eq!(store.records(), before_records);
    assert_eq!(store.export_memory_links_notation(), before_export);
}

#[cfg(feature = "doublets-native")]
#[test]
fn doublets_native_backend_mirrors_memory_events() {
    use super::DoubletsLinkStore;

    let mut store = DoubletsLinkStore::new().expect("native doublets store");
    let id = store
        .append_memory_event(MemoryEvent::assistant("hi back"))
        .expect("append");
    assert!(id.starts_with("memory_event_"));
    assert_eq!(store.backend(), LinkStoreBackend::DoubletsRs);
    assert_eq!(store.records().len(), 1);
    assert!(
        store.native_link_count() > store.records()[0].links.len(),
        "native store should contain point nodes plus projected doublets"
    );
}
