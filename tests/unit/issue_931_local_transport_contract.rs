//! Self-hosted protocol-contract regressions for issue #931.

const CANONICAL: &str = include_str!("../../data/meta/local-transport-protocol.lino");
const AUTHORED: &str = include_str!(
    "../../docs/case-studies/issue-931/self-hosting-authorship/local-transport-protocol.lino"
);
const DECOMPOSITION: &str =
    include_str!("../../docs/case-studies/issue-931/self-hosting-authorship/decomposition.lino");

#[test]
fn issue_931_formal_ai_authored_protocol_is_preserved_byte_for_byte() {
    assert_eq!(CANONICAL.as_bytes(), AUTHORED.as_bytes());
    for invariant in [
        "default_host \"127.0.0.1\"",
        "router \"handle_api_request_with_headers\"",
        "server_flag \"--ws\"",
        "server_flag \"--webrtc\"",
        "ice_policy \"host_only\"",
        "central_relay \"false\"",
        "new_storage_engine \"false\"",
    ] {
        assert!(CANONICAL.contains(invariant), "missing {invariant}");
    }
}

#[test]
fn issue_931_decomposition_assigns_one_of_five_leaves_to_formal_ai() {
    assert!(DECOMPOSITION.contains("leaf_count \"5\""));
    assert!(DECOMPOSITION.contains("formal_ai_authored_leaf_count \"1\""));
    assert!(DECOMPOSITION.contains("formal_ai_authored_percent \"20\""));
    assert_eq!(DECOMPOSITION.matches("owner \"formal_ai_agent_cli\"").count(), 1);
    assert_eq!(DECOMPOSITION.matches("record_type \"smallest_leaf\"").count(), 5);
}
