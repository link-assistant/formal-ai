//! Keep report commands attached to clients that omit a provider session ID.

use std::env;
use std::fs;
use std::io::{self, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{CallerArgs, RenderContext};

pub(super) struct AiderHistory {
    scoped: PathBuf,
    default: PathBuf,
    start: u64,
}

impl AiderHistory {
    pub(super) fn preserve(&self) -> io::Result<()> {
        let mut source = fs::File::open(&self.scoped)?;
        if source.metadata()?.len() == self.start {
            return Ok(());
        }
        source.seek(SeekFrom::Start(self.start))?;
        let mut destination = crate::dialog_log::open_private_log(&self.default)?;
        destination.lock()?;
        io::copy(&mut source, &mut destination)?;
        Ok(())
    }
}

pub(super) fn configure(
    command: &mut Command,
    client: &str,
    caller: &CallerArgs,
    context: &RenderContext,
    scoped_home: Option<&Path>,
) -> io::Result<Option<AiderHistory>> {
    let variable = match client {
        "aider" => "OPENAI_API_BASE",
        // Grok Build uses its TOML provider and declares its own session ID.
        // The earlier Grok CLI uses this environment variable.
        "grok" => "GROK_BASE_URL",
        _ => return Ok(None),
    };
    let endpoint = context.endpoint_base_url.trim_end_matches('/');
    command.env(
        variable,
        format!("{endpoint}/dialog/{}", context.report_dialog_id),
    );
    if client != "aider" {
        return Ok(None);
    }
    let cwd = env::current_dir()?;
    let explicit = caller
        .option_value("--chat-history-file")
        .map(PathBuf::from)
        .or_else(|| env::var_os("AIDER_CHAT_HISTORY_FILE").map(PathBuf::from));
    if let Some(explicit) = explicit {
        set_aider_history(command, &cwd.join(explicit));
        return Ok(None);
    }
    let project = if caller.has_option("--no-git") {
        &cwd
    } else {
        cwd.ancestors()
            .find(|path| path.join(".git").exists())
            .unwrap_or(&cwd)
    };
    let root = scoped_home
        .ok_or(io::ErrorKind::NotFound)?
        .join(".aider/chats");
    fs::create_dir_all(&root)?;
    let scoped = root.join(format!("{}.md", context.report_dialog_id));
    let default = project.join(".aider.chat.history.md");
    let mut transcript = crate::dialog_log::open_private_log(&scoped)?;
    if caller.has_option("--restore-chat-history") {
        match fs::File::open(&default) {
            Ok(mut history) => {
                io::copy(&mut history, &mut transcript)?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    let start = set_aider_history(command, &scoped);
    Ok(Some(AiderHistory {
        scoped,
        default,
        start,
    }))
}

fn set_aider_history(command: &mut Command, history: &Path) -> u64 {
    let start = fs::metadata(history).map_or(0, |metadata| metadata.len());
    command
        .env("AIDER_CHAT_HISTORY_FILE", history)
        .env("FORMAL_AI_REPORT_AIDER_HISTORY", history)
        .env("FORMAL_AI_REPORT_AIDER_HISTORY_START", start.to_string());
    start
}
