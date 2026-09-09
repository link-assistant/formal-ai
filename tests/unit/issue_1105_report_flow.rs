//! The five live root causes of the issue-reporting flow (issue #1105).
//!
//! One session (#1104) produced a filed issue that was wrong in five separate
//! ways at once. Each is pinned here against the mechanism that produced it,
//! not against the rendered wording, so a future change to the prose cannot
//! quietly reopen one of them.

use std::fs;
use std::path::PathBuf;

/// RC5 -- two conversations that open with the same words are not one.
///
/// With no `x-formal-ai-dialog-id` header -- which is every opencode session --
/// the id was a content hash of the first user message and nothing else. In the
/// reported store one collided dialog had absorbed 52 % of all exchanges, and
/// exporting it returned another conversation's turns.
mod dialog_identity {
    use formal_ai::dialog_log::{DIALOG_ID_HEADER, write_dialog_exchange};

    fn record(directory: &std::path::Path, headers: &[(&str, &str)], prompt: &str) -> String {
        let body = format!(r#"{{"messages":[{{"role":"user","content":"{prompt}"}}]}}"#);
        write_dialog_exchange(
            directory,
            "POST",
            "/v1/chat/completions",
            headers,
            &body,
            200,
            "application/json",
            r#"{"choices":[{"message":{"content":"ok"}}]}"#,
        )
        .expect("write the exchange")
        .file_stem()
        .expect("the log is named for its dialog")
        .to_string_lossy()
        .into_owned()
    }

    /// A declared session id is the caller's own answer and is used verbatim.
    #[test]
    fn a_declared_session_id_is_used_verbatim() {
        let directory = super::scratch("declared");
        let id = record(&directory, &[(DIALOG_ID_HEADER, "ses_declared_1")], "Hi");
        assert_eq!(id, "ses_declared_1");
    }

    /// The defect itself: two conversations opening with the same word, from a
    /// client that declares no id, must not become one conversation.
    #[test]
    fn two_undeclared_conversations_opening_alike_do_not_collide() {
        let first = super::scratch("collide-a");
        let second = super::scratch("collide-b");
        let one = record(&first, &[], "Hi");
        let two = record(&second, &[], "Hi");
        assert_ne!(
            one, two,
            "two separate conversations that both open with `Hi` were given one id, which is \
             how a single dialog absorbed 52 % of the reported store"
        );
    }

    /// The turns of one conversation still land in one log: a later request in
    /// the same dialog repeats its opening message, and that is what rejoins it.
    #[test]
    fn later_turns_of_one_conversation_rejoin_it() {
        let directory = super::scratch("rejoin");
        let body = r#"{"messages":[{"role":"user","content":"Hi"},{"role":"assistant","content":"Hello"},{"role":"user","content":"Execute it again."}]}"#;
        let first = record(&directory, &[], "Hi");
        let second = write_dialog_exchange(
            &directory,
            "POST",
            "/v1/chat/completions",
            &[],
            body,
            200,
            "application/json",
            r#"{"choices":[{"message":{"content":"ok"}}]}"#,
        )
        .expect("write the second turn")
        .file_stem()
        .expect("named log")
        .to_string_lossy()
        .into_owned();
        assert_eq!(
            first, second,
            "a second turn of the same conversation must extend the same log"
        );
    }
}

/// RC7 -- a report that names one destination while another failed.
///
/// `report_finished` read only the GitHub URL, so an export that failed beside
/// a filed issue was invisible; the session answered success.
mod report_outcome {
    /// The failure classifier the report path now shares with verification.
    #[test]
    fn a_non_zero_command_is_read_as_a_failure() {
        use formal_ai::agentic_coding::tool_result::{StepOutcome, step_outcome};
        assert_eq!(
            step_outcome("{\"exit_code\":1,\"stdout\":\"\",\"stderr\":\"gist: not found\"}"),
            StepOutcome::Failed,
            "a non-zero export must not be reported as a success"
        );
        assert_eq!(
            step_outcome("{\"exit_code\":0,\"stdout\":\"https://gist.github.com/x\"}"),
            StepOutcome::Succeeded
        );
    }
}

/// RC2 -- a 45 KB transcript pasted into the issue body.
///
/// The inline budget was 50 000 bytes, so a 45 KB capture stayed inline and
/// buried the report's own sections.
mod inline_budget {
    #[test]
    fn a_transcript_of_the_reported_size_no_longer_stays_inline() {
        let source = fs::read_to_string("src/cli_report.rs").expect("read cli_report.rs");
        let line = source
            .lines()
            .find(|line| line.contains("const DEFAULT_INLINE_BYTES"))
            .expect("the inline budget is declared");
        let budget: usize = line
            .rsplit_once('=')
            .and_then(|(_, value)| {
                value
                    .trim()
                    .trim_end_matches(';')
                    .replace('_', "")
                    .parse()
                    .ok()
            })
            .expect("the budget is a number");
        assert!(
            budget < 45_000,
            "the reported issue pasted 45 KB inline; a budget of {budget} would do it again"
        );
    }

    use std::fs;
}

/// RC4 -- `context learn --session latest` never resolved.
///
/// `export` resolved `latest` through `resolve_session`; `learn` passed the
/// word through as a literal conversation id, which never exists.
mod latest_resolution {
    #[test]
    fn learn_resolves_latest_like_every_other_session_subcommand() {
        let source = fs::read_to_string("src/cli_context.rs").expect("read cli_context.rs");
        let learn = source
            .split_once("ContextAction::Learn {")
            .expect("the learn arm exists")
            .1;
        let arm = &learn[..learn.find("ContextAction::").unwrap_or(learn.len())];
        assert!(
            arm.contains("resolve_session("),
            "learn must resolve its session argument, not use it verbatim:\n{arm}"
        );
    }

    use std::fs;
}

/// RC6 -- the export returned a different conversation and said nothing.
///
/// `latest` fell back to the newest dialog file on disk, which under any
/// concurrent session is somebody else's conversation.
mod export_target {
    #[test]
    fn a_guessed_session_is_announced_to_the_caller() {
        let source = fs::read_to_string("src/cli_context.rs").expect("read cli_context.rs");
        let block = source
            .split_once("latest_recorded_dialog(log_dir)")
            .expect("the recorded fallback exists")
            .1;
        let arm = &block[..block.find("\n}").unwrap_or(block.len())];
        assert!(
            arm.contains("eprintln!"),
            "resolving `latest` to the newest file on disk is a guess and must say so:\n{arm}"
        );
    }

    use std::fs;
}

/// A scratch dialog-log directory for one test.
fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "formal-ai-issue-1105-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("scratch directory");
    directory
}
