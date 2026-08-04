//! Issue #903: the argv `formal-ai with <tool> …` builds for the native CLI.
//!
//! Kept apart from the configuration and server tests so each file stays
//! reviewable; the shims and capture helpers are shared with them.

use std::os::unix::fs::PermissionsExt as _;
use std::process::{Command, Stdio};

use super::with_formal_ai::{
    captured_args_without_model_catalog, path_with_fake_clis, run_with_capture,
    run_with_capture_stdin, tmpdir, write_fake_cli,
};

/// Issue #903 defects 2, 3 and 4: a caller flag the wrapper also defines
/// (`--verbose`) reaches the client, a piped prompt is rendered in the client's
/// own vocabulary instead of leaving a value-less mode flag behind, and no
/// client is handed another client's prompt spelling (`codex exec` has no `-p`).
#[test]
fn with_formal_ai_renders_the_same_argv_from_an_argument_and_a_piped_prompt() {
    let cases = [
        (
            "codex",
            vec![
                "exec",
                "--skip-git-repo-check",
                "--sandbox",
                "read-only",
                "--verbose",
                "hi",
            ],
        ),
        (
            "opencode",
            vec!["run", "-m", "formalai/formal-ai", "--verbose", "hi"],
        ),
        (
            "agent",
            vec![
                "--no-summarize-session",
                "--compaction-model",
                "same",
                "--model",
                "formalai/formal-ai",
                "--verbose",
                "-p",
                "hi",
            ],
        ),
        ("gemini", vec!["-m", "formal-ai", "--verbose", "-p", "hi"]),
        (
            "claude",
            vec!["--model", "formal-ai", "--verbose", "--print", "hi"],
        ),
        (
            "qwen",
            vec!["--model", "formal-ai", "--verbose", "-p", "hi"],
        ),
        (
            "grok",
            vec!["--model", "formal-ai", "--verbose", "--prompt", "hi"],
        ),
        (
            "aider",
            vec![
                "--no-auto-commits",
                "--model",
                "openai/formal-ai",
                "--verbose",
                "--message",
                "hi",
            ],
        ),
    ];
    for (tool, expected) in cases {
        let dir = tmpdir();
        let home = dir.join("home");
        let bin_dir = dir.join("bin");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin_dir).expect("bin");
        write_fake_cli(&bin_dir, tool);

        // Shape A: the prompt is an argument, spelled the way a caller who knows
        // one client would spell it.
        let argument_capture = dir.join("argument.txt");
        let argument = run_with_capture(
            &home,
            &bin_dir,
            &argument_capture,
            &["with", tool, "--verbose", "-p", "hi"],
        );
        assert!(
            argument.status.success(),
            "{tool}: {}",
            String::from_utf8_lossy(&argument.stderr)
        );
        let captured = std::fs::read_to_string(&argument_capture).expect("argument capture");
        assert_eq!(
            captured_args_without_model_catalog(&captured),
            expected,
            "{tool} argument-prompt capture:\n{captured}"
        );

        // Shape B: the same prompt piped in, with stdin deliberately not a TTY.
        let piped_capture = dir.join("piped.txt");
        let piped = run_with_capture_stdin(
            &home,
            &bin_dir,
            &piped_capture,
            &["with", tool, "--verbose"],
            Some("hi\n"),
        );
        assert!(
            piped.status.success(),
            "{tool}: {}",
            String::from_utf8_lossy(&piped.stderr)
        );
        let captured = std::fs::read_to_string(&piped_capture).expect("piped capture");
        assert_eq!(
            captured_args_without_model_catalog(&captured),
            expected,
            "{tool} piped-prompt capture:\n{captured}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// The prompt is the request wherever the caller put it: the client matrix
/// sends it ahead of the client's own trailing flags (`<prompt> --file
/// alpha.txt`), and a client whose headless flag takes the prompt as its value
/// must still receive it.
#[test]
fn with_formal_ai_reads_a_prompt_that_precedes_trailing_client_flags() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "aider");
    let capture = dir.join("capture.txt");

    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &[
            "with",
            "--non-interactive",
            "aider",
            "read alpha.txt",
            "--file",
            "alpha.txt",
        ],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert_eq!(
        captured_args_without_model_catalog(&captured),
        [
            "--no-auto-commits",
            "--model",
            "openai/formal-ai",
            "--file",
            "alpha.txt",
            "--message",
            "read alpha.txt",
        ],
        "capture:\n{captured}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A client whose prompt is positional keeps the caller's tokens in the order
/// they were written: `t3code serve` is a subcommand, not a request, and the
/// launch leg of the client matrix fails outright when it is moved.
#[test]
fn with_formal_ai_keeps_a_positional_subcommand_in_place() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "t3");
    let capture = dir.join("capture.txt");

    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &[
            "with",
            "t3code",
            "serve",
            "--no-browser",
            "--host",
            "127.0.0.1",
            "--port",
            "9010",
        ],
    );

    assert!(
        output.status.success(),
        "t3code: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert_eq!(
        captured_args_without_model_catalog(&captured),
        [
            "serve",
            "--host",
            "127.0.0.1",
            "--port",
            "9010",
            "--no-browser",
        ],
        "capture:\n{captured}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The two shapes the E2E harnesses use: a caller who writes the `--`
/// delimiter themselves, and repeated unknown flags whose values must not
/// swallow the prompt that follows them.
#[test]
fn with_formal_ai_forwards_caller_delimited_and_repeated_unknown_flags() {
    let cases = [
        (
            "claude",
            vec![
                "--non-interactive",
                "claude",
                "--",
                "--mcp-config",
                "/tmp/formal-ai-mcp.json",
                "--strict-mcp-config",
                "--",
                "research the part",
            ],
            vec![
                "--model",
                "formal-ai",
                "--mcp-config",
                "/tmp/formal-ai-mcp.json",
                "--strict-mcp-config",
                "--print",
                "research the part",
            ],
        ),
        (
            "codex",
            vec![
                "--non-interactive",
                "codex",
                "--",
                "--json",
                "-c",
                "mcp_servers.demo.command=\"node\"",
                "-c",
                "mcp_servers.demo.args=[\"server.mjs\"]",
                "research the part",
            ],
            vec![
                "exec",
                "--skip-git-repo-check",
                "--sandbox",
                "read-only",
                "--json",
                "-c",
                "mcp_servers.demo.command=\"node\"",
                "-c",
                "mcp_servers.demo.args=[\"server.mjs\"]",
                "research the part",
            ],
        ),
    ];
    for (tool, arguments, expected) in cases {
        let dir = tmpdir();
        let home = dir.join("home");
        let bin_dir = dir.join("bin");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin_dir).expect("bin");
        write_fake_cli(&bin_dir, tool);
        let capture = dir.join("capture.txt");

        let mut argv = vec!["with"];
        argv.extend(arguments);
        let output = run_with_capture(&home, &bin_dir, &capture, &argv);

        assert!(
            output.status.success(),
            "{tool}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let captured = std::fs::read_to_string(&capture).expect("capture");
        assert_eq!(
            captured_args_without_model_catalog(&captured),
            expected,
            "{tool} capture:\n{captured}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Issue #903 defect 1: the provider prefix belongs only to a bare model alias.
#[test]
fn with_formal_ai_keeps_an_already_qualified_model_selector() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "agent");
    let capture = dir.join("capture.txt");

    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &["with", "--model", "formalai/formal-ai", "agent", "hi"],
    );

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert_eq!(
        captured_args_without_model_catalog(&captured),
        [
            "--no-summarize-session",
            "--compaction-model",
            "same",
            "--model",
            "formalai/formal-ai",
            "-p",
            "hi",
        ],
        "capture:\n{captured}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Issue #903 defect 5: the completion ladder re-renders the caller's own
/// option set with only the prompt substituted, so the retry runs with the
/// permission posture and toolset the caller configured — and the wrapper's own
/// overlay is not duplicated by a caller who already passed it.
#[test]
fn with_formal_ai_retries_with_the_caller_passthrough_flags_intact() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    let workspace = dir.join("workspace");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    std::fs::create_dir_all(&workspace).expect("workspace");
    let log = dir.join("attempts.log");
    let claude = bin_dir.join("claude");
    std::fs::write(
        &claude,
        format!(
            r#"#!/bin/sh
{{ printf 'ATTEMPT'; for arg in "$@"; do printf '[%s]' "$arg"; done; }} >> "{log}"
printf '{{"type":"result","subtype":"success","result":"done"}}\n'
"#,
            log = log.display()
        ),
    )
    .expect("write recording claude");
    let mut permissions = std::fs::metadata(&claude).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&claude, permissions).expect("chmod recording claude");

    let output = Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .args([
            "with",
            "--no-start-server",
            "claude",
            "--dangerously-skip-permissions",
            "--output-format",
            "stream-json",
            "--mcp-config",
            "/tmp/formal-ai-mcp.json",
            "--disallowedTools",
            "Bash",
            "Edit",
            "-p",
            "Create a file named hello.txt with the text hi",
        ])
        .current_dir(&workspace)
        .env("HOME", &home)
        .env("PATH", path_with_fake_clis(&bin_dir))
        .stdin(Stdio::null())
        .output()
        .expect("run formal-ai with");

    // The client never writes the file, so the completion ladder engages; the
    // wrapper reports the unmet contract, which is not what this test asserts.
    let _ = output.status;
    // Recovery prompts span several lines, so attempts are separated by a
    // marker the shim writes rather than by newlines.
    let attempts = std::fs::read_to_string(&log).expect("attempt log");
    let attempts: Vec<&str> = attempts.split("ATTEMPT").skip(1).collect();
    assert!(
        attempts.len() > 1,
        "the completion ladder did not retry:\n{}",
        attempts.join("\n---\n")
    );
    for attempt in &attempts {
        for flag in [
            "[--dangerously-skip-permissions]",
            "[--mcp-config][/tmp/formal-ai-mcp.json]",
            "[--disallowedTools][Bash][Edit]",
            "[--print]",
        ] {
            assert!(attempt.contains(flag), "attempt lost {flag}:\n{attempt}");
        }
        assert_eq!(
            attempt.matches("[--output-format][stream-json]").count(),
            1,
            "the caller's --output-format was duplicated:\n{attempt}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}
