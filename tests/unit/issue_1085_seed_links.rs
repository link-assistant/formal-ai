//! Issue #1085 (D1.2): the seed and meta documents are one links network, and
//! routing reads it through link queries.

use formal_ai::cue_lexicon::{CUE_LEXICON_LINO, CUE_LEXICON_PATH, cue_sets, cue_sets_from};
use formal_ai::seed::{
    HANDLER_PRECEDENCE_LINO, HANDLER_PRECEDENCE_PATH, INTENT_ROUTING_LINO, INTENT_ROUTING_PATH,
    handler_precedence, handler_precedence_from, intent_routing, intent_routing_from, seed_files,
};
use formal_ai::seed_links::{SeedLinkNetwork, meta_documents, network};

#[test]
fn every_seed_and_meta_document_is_loaded_into_the_network_once() {
    let loaded = network();
    assert_eq!(
        loaded.document_count(),
        seed_files().len() + meta_documents().len(),
        "one document node per bundled seed file plus the routing meta documents"
    );
    for (path, _) in seed_files() {
        assert!(loaded.document(path).is_some(), "{path} must be projected");
    }
    assert!(loaded.document(CUE_LEXICON_PATH).is_some());
    assert!(
        loaded.links().len() > 50_000,
        "the seed bundle projects to tens of thousands of links, got {}",
        loaded.links().len()
    );
}

#[test]
fn handler_precedence_read_through_link_queries_equals_the_document() {
    let from_links = handler_precedence();
    assert_eq!(from_links, handler_precedence_from(HANDLER_PRECEDENCE_LINO));
    assert!(from_links.len() > 40);
    // The same rows through the generic pattern `(root $child)`.
    let loaded = network();
    let root = loaded
        .top_level(HANDLER_PRECEDENCE_PATH)
        .into_iter()
        .next()
        .expect("the precedence document has a root");
    let matched = loaded.query(&SeedLinkNetwork::children_pattern(&root.index));
    let names: Vec<&str> = matched
        .iter()
        .filter(|link| !link.index.ends_with('='))
        .map(|link| link.to.as_str())
        .collect();
    assert_eq!(names, from_links);
}

#[test]
fn cue_sets_and_intent_routing_read_the_same_records_as_their_former_parsers() {
    assert_eq!(cue_sets(), cue_sets_from(CUE_LEXICON_LINO).as_slice());
    assert!(cue_sets().iter().any(|set| set.name == "write_script"));
    let routing = intent_routing();
    assert_eq!(routing, intent_routing_from(INTENT_ROUTING_LINO));
    assert!(routing.intents.iter().any(|route| route.slug == "greeting"));
    assert!(network().document(INTENT_ROUTING_PATH).is_some());
}

#[test]
fn a_node_with_a_value_and_children_projects_to_node_value_and_child_links() {
    let document = "root\n  entry first\n    slug alpha\n    cue one\n    cue two\n  entry\n";
    let small = SeedLinkNetwork::from_documents(&[("fixture.lino", document)]);
    let root = small.top_level("fixture.lino");
    assert_eq!(root.len(), 1);
    let entries = small.nodes_under(&root[0].index);
    assert_eq!(entries.len(), 2);
    assert_eq!(small.value_of(&entries[0].index), Some("first"));
    assert_eq!(small.value_of(&entries[1].index), None);
    assert_eq!(small.field(&entries[0].index, "slug"), Some("alpha"));
    assert_eq!(
        small.field_values(&entries[0].index, "cue"),
        vec!["one".to_owned(), "two".to_owned()]
    );
    assert_eq!(
        small.links().len(),
        1 + 1 + 2 + 1 + 2 + 2 + 2,
        "document, root, two entries, one value, slug node+value, two cue nodes+values"
    );
}

#[cfg(feature = "doublets-native")]
#[test]
fn the_network_rebuilds_a_native_link_cli_store_without_accumulating() {
    use formal_ai::link_store::LinkCliLinkStore;
    let small =
        SeedLinkNetwork::from_documents(&[(HANDLER_PRECEDENCE_PATH, HANDLER_PRECEDENCE_LINO)]);
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-seed-links-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default()
    ));
    std::fs::create_dir_all(&directory).expect("temp directory");
    let database = formal_ai::seed_links::seed_link_database_path(&directory.join("memory.lino"));
    let first =
        LinkCliLinkStore::rebuild_with_doublets(&database, small.links()).expect("first rebuild");
    let count = first.native_link_count();
    assert!(
        count >= small.links().len(),
        "{count} native links for {} doublets",
        small.links().len()
    );
    drop(first);
    let second =
        LinkCliLinkStore::rebuild_with_doublets(&database, small.links()).expect("second rebuild");
    assert_eq!(
        second.native_link_count(),
        count,
        "a rebuild starts from an empty store"
    );
    drop(second);
    std::fs::remove_dir_all(&directory).ok();
}
