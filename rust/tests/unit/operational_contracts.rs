//! Replay the existing shell documentation and download retry contracts without
//! compiling the monolithic unit target.

#[path = "docs_requirements/agent_cli_shell.rs"]
mod agent_cli_shell;
#[path = "ci-cd/network_download_retry.rs"]
mod network_download_retry;
// Issue #1639: the minimal-core census pin ran only in the main-only full
// shards, so a new handler source reached `main` before the pin moved.
#[path = "minimal_core_and_seed_metadata.rs"]
mod minimal_core_and_seed_metadata;
