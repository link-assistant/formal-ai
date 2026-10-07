//! Self-hosted protocol-contract regressions for issue #931.

const CANONICAL: &str = include_str!("../../../data/meta/local-transport-protocol.lino");
const AUTHORED: &str = include_str!(
    "../../../docs/case-studies/issue-931/self-hosting-authorship/local-transport-protocol.lino"
);
const DECOMPOSITION: &str =
    include_str!("../../../docs/case-studies/issue-931/self-hosting-authorship/decomposition.lino");
const CASE_STUDY: &str = include_str!("../../../docs/case-studies/issue-931/README.md");
const USER_GUIDE: &str = include_str!("../../../docs/local-transports.md");
const REQUIREMENTS: &str =
    include_str!("../../../docs/requirements/issue-0931-local-transports.md");

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
    assert_eq!(
        DECOMPOSITION
            .matches("owner \"formal_ai_agent_cli\"")
            .count(),
        1
    );
    assert_eq!(
        DECOMPOSITION
            .matches("record_type \"smallest_leaf\"")
            .count(),
        5
    );
    assert_eq!(DECOMPOSITION.matches("status \"complete\"").count(), 5);
}

#[test]
fn issue_931_documents_every_requirement_and_manual_protocol_check() {
    for requirement in 1..=12 {
        let id = format!("R931-{requirement}");
        assert!(CASE_STUDY.contains(&id), "case study is missing {id}");
        assert!(
            REQUIREMENTS.contains(&id),
            "requirement shard is missing {id}"
        );
    }
    for contract in [
        "formal-ai serve --ws",
        "formal-ai serve --webrtc",
        "formal-ai connect",
        "websocat",
        "four-byte big-endian length",
        "16 MiB",
    ] {
        assert!(
            USER_GUIDE.contains(contract),
            "user guide is missing {contract}"
        );
    }
}

/// R931-11: the transports shipped as a minor release with their changelog
/// entry (0.343.0, PR #1008), wherever the changelog keeps that release.
#[test]
fn issue_931_shipped_as_a_minor_release_with_its_changelog_entry() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut parts = vec![root.join("CHANGELOG.md")];
    if let Ok(entries) = std::fs::read_dir(root.join("docs/changelog")) {
        parts.extend(entries.filter_map(Result::ok).map(|entry| entry.path()));
    }
    let entry =
        "Add localhost-default WebSocket and host-only WebRTC data-channel server and client modes";
    let release = parts
        .iter()
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .find_map(|text| {
            let at = text.find(entry)?;
            let heading = text[..at].rfind("\n## [")?;
            Some(text[heading + 1..at].to_owned())
        })
        .expect("the changelog keeps the issue #931 entry");
    assert!(release.starts_with("## [0.343.0]"), "{release}");
    assert!(release.contains("### Added"), "{release}");
}
