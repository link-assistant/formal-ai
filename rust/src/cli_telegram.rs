//! The `formal-ai telegram` runner, lifted out of `src/main.rs`.
//!
//! `main.rs` is the command *surface*: the clap declarations and the one match
//! that routes them. Each command's body lives in its own `cli_*` module, and
//! this one was the last body still inlined beside the surface. Moving it keeps
//! `main.rs` under the file-size limit the whole tree is held to, which is the
//! same reason `cli_improve`, `cli_memory` and the rest exist.

use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use formal_ai::{TelegramPollingConfig, run_telegram_polling, run_telegram_webhook_server};

use crate::{TelegramMode, TelegramRunArgs};

/// Run the Telegram client in whichever mode the command line selected.
pub fn run_telegram(args: TelegramRunArgs) -> Result<(), Box<dyn Error>> {
    match args.mode {
        TelegramMode::Polling => {
            let token = args.token.ok_or_else(|| {
                String::from(
                    "Telegram polling mode requires a bot token. \
                     Pass --token or set TELEGRAM_BOT_TOKEN.",
                )
            })?;
            let mut config = TelegramPollingConfig::new(token);
            config.api_base = args.api_base;
            config.timeout_seconds = args.timeout;
            config.limit = args.limit.clamp(1, 100);
            config.allowed_updates = parse_allowed_updates(&args.allowed_updates);
            run_telegram_polling(&config, None, Arc::new(AtomicBool::new(false)))?;
        }
        TelegramMode::Webhook => {
            run_telegram_webhook_server(&format!(
                "{host}:{port}",
                host = args.host,
                port = args.port
            ))?;
        }
    }
    Ok(())
}

/// Split a comma-separated `--allowed-updates` list, dropping empty entries.
fn parse_allowed_updates(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
