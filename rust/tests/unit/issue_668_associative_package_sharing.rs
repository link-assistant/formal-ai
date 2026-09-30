//! Issue #668: fresh-instance import, permission review, and loud rejection.
use formal_ai::associative_package::PackageStore;
use formal_ai::associative_packages::{
    AvailableHandler, PermissionReview, SharedPackage, SharingError, import_package,
};
use formal_ai::memory::MemoryStore;
use std::collections::{BTreeMap, BTreeSet};

const EXAMPLE: &str = include_str!("../../../examples/packages/greeting.lino");
fn catalog(agent: bool) -> BTreeMap<String, AvailableHandler> {
    BTreeMap::from([(
        String::from("greeting-response"),
        AvailableHandler {
            kind: String::from("response"),
            capability: String::from("greeting"),
            agent_tagged: agent,
        },
    )])
}
fn review(text: &str) -> PermissionReview {
    PermissionReview {
        artifact: text.to_owned(),
        acknowledged: true,
        approved_agent_capabilities: BTreeSet::new(),
    }
}
#[test]
fn associative_package_sharing_round_trip_preserves_links_and_permission_gates() {
    let source = SharedPackage::parse(EXAMPLE).unwrap();
    let artifact = source.export();
    let mut registry = PackageStore::default();
    let mut memory = MemoryStore::new();
    let imported = import_package(
        &artifact,
        &mut registry,
        &mut memory,
        &catalog(false),
        &review(&artifact),
    )
    .unwrap();
    assert_eq!(imported, source);
    assert_eq!(
        registry.packages()[0].link_records(),
        source.package.link_records()
    );
    assert!(
        registry.packages()[0]
            .grants_capability("greeting")
            .is_some()
    );
    assert!(registry.packages()[0].grants_capability("shell").is_none());
    assert_eq!(memory.events()[0].kind.as_deref(), Some("package_imported"));
    assert_eq!(SharedPackage::parse(&imported.export()).unwrap(), imported);
}
#[test]
fn associative_package_sharing_rejects_missing_handler_without_mutation() {
    let mut registry = PackageStore::default();
    let mut memory = MemoryStore::new();
    assert!(matches!(
        import_package(
            EXAMPLE,
            &mut registry,
            &mut memory,
            &BTreeMap::new(),
            &review(EXAMPLE)
        ),
        Err(SharingError::UnavailableHandler(_))
    ));
    assert!(registry.packages().is_empty());
    assert!(memory.events().is_empty());
}
#[test]
fn associative_package_sharing_requires_explicit_agent_approval() {
    let mut registry = PackageStore::default();
    let mut memory = MemoryStore::new();
    let mut consent = review(EXAMPLE);
    assert!(matches!(
        import_package(
            EXAMPLE,
            &mut registry,
            &mut memory,
            &catalog(true),
            &consent
        ),
        Err(SharingError::AgentApprovalRequired(_))
    ));
    consent
        .approved_agent_capabilities
        .insert(String::from("greeting"));
    import_package(
        EXAMPLE,
        &mut registry,
        &mut memory,
        &catalog(true),
        &consent,
    )
    .unwrap();
    consent.artifact.push(' ');
    assert_eq!(
        import_package(
            EXAMPLE,
            &mut registry,
            &mut memory,
            &catalog(true),
            &consent
        ),
        Err(SharingError::ReviewRequired)
    );
}

#[test]
fn associative_package_sharing_agent_handler_cannot_omit_its_permission_to_bypass_consent() {
    let mut package = SharedPackage::parse(EXAMPLE).unwrap();
    package.package.permissions.clear();
    let artifact = package.export();
    let mut registry = PackageStore::default();
    let mut memory = MemoryStore::new();
    assert!(matches!(
        import_package(
            &artifact,
            &mut registry,
            &mut memory,
            &catalog(true),
            &review(&artifact)
        ),
        Err(SharingError::AgentApprovalRequired(_))
    ));
    assert!(registry.packages().is_empty());
}
