//! Replay complete CLI, terminal, memory and handler-family contracts.
//! Keep ordinary module lookup so nested integration fixtures resolve unchanged.

pub mod formal_ai_cli;
#[path = "../unit/seed/issue_699_handler_migration.rs"]
pub mod handler_migration;
pub mod http_server;
pub mod issue_1138_family_behaviour;
pub mod issue_819_tui_isolation;
pub mod pty;
#[path = "../unit/formal_ai/seed_and_memory.rs"]
pub mod seed_and_memory;
#[path = "../support/structured_stderr.rs"]
pub mod structured_stderr;
pub mod with_formal_ai;
pub mod with_formal_ai_argv;

// Keep this input finite: a nested scheme must be opaque without following the
// recursive origin behavior fixed by RUSTSEC-2026-0592.
#[test]
fn url_origins_keep_nested_blob_schemes_opaque() {
    for input in ["https://example.com/path", "blob:https://example.com/id"] {
        let parsed = url::Url::parse(input).unwrap();
        assert_eq!(parsed.origin().ascii_serialization(), "https://example.com");
    }
    for inner in ["blob:https://example.com/id", "ws://example.com/id"] {
        let parsed = url::Url::parse(&format!("blob:{inner}")).unwrap();
        assert_eq!(parsed.origin().ascii_serialization(), "null", "{inner}");
    }
    let nested = format!("{}https://example.com/id", "blob:".repeat(128));
    assert_eq!(
        url::Url::parse(&nested)
            .unwrap()
            .origin()
            .ascii_serialization(),
        "null"
    );
}
