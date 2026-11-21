//! Replay the existing shell documentation and download retry contracts without
//! compiling the monolithic unit target.

#[path = "docs_requirements/agent_cli_shell.rs"]
mod agent_cli_shell;
#[path = "ci-cd/network_download_retry.rs"]
mod network_download_retry;
