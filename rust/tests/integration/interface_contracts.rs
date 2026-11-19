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
