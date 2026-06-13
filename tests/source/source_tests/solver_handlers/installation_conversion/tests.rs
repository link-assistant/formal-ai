//! Unit tests for the structural command recognizer and verb/object intent
//! inference introduced in issue #433. These exercise the private helpers
//! directly so the reasoning (not just the end-to-end routing) is pinned down.

use super::*;

#[test]
fn recognizes_commands_for_tools_absent_from_any_whitelist() {
    // None of these leading tools existed in the retired `PREFIXES` table; the
    // recognizer accepts them on executable-shape alone.
    for command in [
        "bun install",
        "deno task start",
        "uv pip install -r requirements.txt",
        "just build",
        "task test",
        "zig build",
        "pdm install",
        "poetry install",
        "nix build",
        "meson compile build",
    ] {
        assert!(
            looks_like_command(command, Provenance::CodeSpan),
            "expected `{command}` to be recognized as a command"
        );
    }
}

#[test]
fn pipelines_and_paths_are_commands_regardless_of_provenance() {
    for command in [
        "curl -fsSL https://example.com/install.sh | sh",
        "irm https://get.example.com | iex",
        "./configure && make",
        "./webui.sh",
    ] {
        assert!(
            looks_like_command(command, Provenance::CodeSpan),
            "expected `{command}` to be recognized as a command"
        );
    }
}

#[test]
fn prose_lines_are_rejected() {
    for prose in [
        "Clone the repository",
        "Enter the project directory",
        "Install dependencies",
        "Build the project.",
        "Run the verification command",
        "First clone the repository and then install the dependencies",
        "clone the repository manually",
        "Выполните установку",
        "运行安装脚本",
        "## Installation",
        "Then build everything from source",
    ] {
        assert!(
            !looks_like_command(prose, Provenance::BareLine),
            "expected prose `{prose}` to be rejected"
        );
    }
}

#[test]
fn bare_line_embedding_a_code_span_is_prose() {
    // The inline collector already lifted the real command out of the span, so
    // the surrounding sentence is noise.
    assert!(!looks_like_command(
        "Run `npm install` to set up",
        Provenance::BareLine
    ));
    // The same command text on its own (code provenance) is recognized.
    assert!(looks_like_command("npm install", Provenance::CodeSpan));
}

#[test]
fn bare_lines_need_more_than_a_lone_word() {
    // A single bare word might be stray prose; demand an argument or path.
    assert!(!looks_like_command("make", Provenance::BareLine));
    assert!(looks_like_command("make", Provenance::CodeSpan));
    assert!(looks_like_command("npm install", Provenance::BareLine));
    assert!(looks_like_command("./webui.sh", Provenance::BareLine));
}

#[test]
fn executable_head_separates_tools_from_words() {
    assert!(is_executable_head("npm"));
    assert!(is_executable_head("yt-dlp"));
    assert!(is_executable_head("docker-compose"));
    assert!(is_executable_head("./webui.sh"));
    assert!(is_executable_head("python3"));
    assert!(!is_executable_head("Clone"));
    assert!(!is_executable_head("Install"));
    assert!(!is_executable_head("##"));
    assert!(!is_executable_head("Выполните"));
    assert!(!is_executable_head("运行"));
}

#[test]
fn describe_command_infers_action_from_verb_not_tool() {
    assert_eq!(
        describe_command("git clone https://example.com/x.git"),
        "Clone the repository"
    );
    assert_eq!(
        describe_command("cd project"),
        "Enter the project directory"
    );
    // Tools the old table never enumerated, but the verb is recognizable.
    assert_eq!(describe_command("bun install"), "Install dependencies");
    assert_eq!(describe_command("pdm install"), "Install dependencies");
    assert_eq!(
        describe_command("just test"),
        "Run the verification command"
    );
    assert_eq!(describe_command("zig build"), "Build the project");
    assert_eq!(
        describe_command("python -m pytest"),
        "Run the verification command"
    );
    assert_eq!(
        describe_command("python -m pip install -r requirements.txt"),
        "Install dependencies"
    );
    // A generic launcher verb defers to the concrete object.
    assert_eq!(describe_command("npm run build"), "Build the project");
    assert_eq!(
        describe_command("flutter doctor"),
        "Run the verification command"
    );
    assert_eq!(
        describe_command("ollama --version"),
        "Verify the installation"
    );
    assert_eq!(describe_command("ollama serve"), "Start the application");
}

#[test]
fn describe_command_synthesizes_for_unknown_verbs() {
    // Unknown verb, but still structurally derived rather than a flat constant.
    assert_eq!(
        describe_command("frobnicate widgets"),
        "Run the frobnicate widgets step"
    );
    assert_eq!(describe_command("daemonize"), "Run daemonize");
}
