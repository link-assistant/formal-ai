//! The generated report script, executed end to end (#839).
//!
//! Every other test in this suite inspects the script as a string. This one
//! runs it: the planner is asked for the GitHub command, the command is handed
//! to `sh` with a stubbed `gh` on `PATH`, and the file that `gh issue create`
//! received is read back. That is the only way to prove the claim #838 broke —
//! that the issue Formal AI files carries the conversation it says it carries.
//!
//! Against the code that filed #838 each test below fails: the script asked for
//! a session id no harness had ever heard of, the body was a `tail -c 12000` of
//! a proxy trace with no sections at all, and a missing command or an empty
//! export still ended with an issue being filed.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::dialog_log::{write_dialog_exchange, DIALOG_ID_HEADER};
use formal_ai::issue_report::SECTIONS;
use formal_ai::server::{
    enable_http_agent_mode_for_current_process, handle_api_request_with_headers,
};
use serde_json::{json, Value};

/// The issue URL the stubbed `gh` prints, standing in for a filed issue.
const ISSUE_URL: &str = "https://github.com/link-assistant/formal-ai/issues/9001";

/// Shell the generated script is executed by, named absolutely because the
/// sandboxed `PATH` below holds only what the test installed.
const SHELL: &str = "/bin/sh";
/// Utilities the script and the `gh` stub call by name.
const SYSTEM_TOOLS: [&str; 3] = ["mktemp", "rm", "cp"];

/// The turn that opens the report flow, and which §4 never quotes as a subject.
const REPORT_REQUEST: &str = "Сообщи об ошибке";

/// The conversation being reported: four turns, two of them the user's.
const TURNS: [(&str, &str); 4] = [
    ("user", "Что такое фуфломицин?"),
    ("assistant", "A placebo sold as medicine."),
    ("user", "Так что это такое то?"),
    ("assistant", "A drug with no proven effect."),
];

/// Locate a program on the `PATH` this test process inherited.
fn which(program: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|directory| directory.join(program))
        .find(|candidate| candidate.is_file())
}

/// The interpreter that runs the harness extractor, resolved past any wrapper.
///
/// `python3` on a developer's `PATH` is often a shell shim; symlinking the shim
/// into an otherwise empty `PATH` would fail on the shim's own dependencies, so
/// the interpreter is asked where it really lives.
fn python_interpreter() -> Option<PathBuf> {
    let output = Command::new("python3")
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .ok()?;
    let path = PathBuf::from(String::from_utf8(output.stdout).ok()?.trim());
    path.is_file().then_some(path)
}

/// A workspace holding the fixture captures and the stubbed `PATH`.
struct Workspace {
    root: PathBuf,
}

impl Workspace {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "formal-ai-issue-839-script-{}-{label}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("bin")).expect("workspace");
        fs::create_dir_all(root.join("logs")).expect("log directory");
        let workspace = Self { root };
        workspace.install_program("formal-ai", Path::new(env!("CARGO_BIN_EXE_formal-ai")));
        for tool in SYSTEM_TOOLS {
            let located = which(tool).unwrap_or_else(|| panic!("{tool} is required by the script"));
            workspace.install_program(tool, &located);
        }
        let python = python_interpreter().expect("python3 is required by the harness extractor");
        workspace.install_program("python3", &python);
        workspace
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    /// Make one program reachable under the name the script calls.
    ///
    /// `PATH` holds nothing but this directory, so a program the test did not
    /// install is genuinely missing — which is how the preflight guard of §5 is
    /// exercised without depending on what the host happens to have.
    fn install_program(&self, name: &str, target: &Path) {
        std::os::unix::fs::symlink(target, self.path(&format!("bin/{name}")))
            .unwrap_or_else(|error| panic!("{name} on PATH: {error}"));
    }

    /// A `gh` that records its arguments, keeps the body it was handed and
    /// prints an issue URL, so the filed document can be read back.
    fn install_gh(&self) {
        let stub = "#!/bin/sh\n\
                    printf '%s\\n' \"$@\" >> \"$GH_ARGV\"\n\
                    while [ $# -gt 0 ]; do\n\
                    \x20 if [ \"$1\" = \"--body-file\" ]; then cp \"$2\" \"$GH_BODY\"; fi\n\
                    \x20 shift\n\
                    done\n\
                    printf '%s\\n' \"$GH_ISSUE_URL\"\n";
        let path = self.path("bin/gh");
        fs::write(&path, stub).expect("gh stub");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("gh stub is runnable");
    }

    /// Record the fixture conversation as the running server would.
    fn record_conversation(&self, session: &str) {
        for pair in TURNS.chunks(2) {
            let [(_, prompt), (_, reply)] = pair else {
                continue;
            };
            let request = json!({
                "model": "formal-ai",
                "messages": [{"role": "user", "content": prompt}],
            })
            .to_string();
            let response = json!({
                "choices": [{"message": {"role": "assistant", "content": reply}}],
            })
            .to_string();
            write_dialog_exchange(
                &self.path("logs"),
                "POST",
                "/v1/chat/completions",
                &[(DIALOG_ID_HEADER, session)],
                &request,
                200,
                "application/json",
                &response,
            )
            .expect("dialog fixture");
        }
    }

    /// Run one generated script with the fixture environment around it.
    fn run(&self, script: &str) -> Output {
        Command::new(SHELL)
            .arg("-c")
            .arg(script)
            .env("PATH", self.path("bin"))
            .env("FORMAL_AI_DIALOG_LOG_DIR", self.path("logs"))
            .env("XDG_DATA_HOME", self.path("data"))
            .env("FORMAL_AI_SILENT", "true")
            .env("GH_ARGV", self.path("gh-argv"))
            .env("GH_BODY", self.path("filed-body.md"))
            .env("GH_ISSUE_URL", ISSUE_URL)
            .env_remove("FORMAL_AI_DIALOG_ID")
            .output()
            .expect("run the generated script")
    }

    /// The body `gh issue create` was handed, or `None` when nothing was filed.
    fn filed_body(&self) -> Option<String> {
        fs::read_to_string(self.path("filed-body.md")).ok()
    }

    fn filed_arguments(&self) -> Option<String> {
        fs::read_to_string(self.path("gh-argv")).ok()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Ask the planner for the GitHub command of a session it was told about.
///
/// The session id travels in the request header, the same way `OpenCode` and
/// the desktop app send it, so the command below asks for the conversation the
/// user is actually in rather than a hash of their first sentence (§2.1).
fn github_command(session: &str) -> String {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "messages": report_messages(),
        "tools": opencode_tools(),
    })
    .to_string();
    let response = handle_api_request_with_headers(
        "POST",
        "/v1/chat/completions",
        &[(DIALOG_ID_HEADER, session)],
        &body,
    );
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).expect("JSON response");
    let call = &response["choices"][0]["message"]["tool_calls"][0]["function"];
    assert_eq!(call["name"], "run_shell_command", "{response}");
    let arguments: Value =
        serde_json::from_str(call["arguments"].as_str().expect("arguments")).expect("arguments");
    arguments["command"]
        .as_str()
        .expect("shell command")
        .to_owned()
}

/// A conversation whose report destinations and contents are already chosen.
fn report_messages() -> Vec<Value> {
    let mut messages: Vec<Value> = TURNS
        .iter()
        .map(|(role, text)| json!({"role": role, "content": text}))
        .collect();
    messages.push(json!({"role": "user", "content": REPORT_REQUEST}));
    messages.extend(answered_question(
        "targets",
        r#"{"report_target":["GitHub issue"]}"#,
    ));
    messages.extend(answered_question(
        "contents",
        r#"{"report_contents":"both_logs"}"#,
    ));
    messages
}

fn answered_question(id: &str, answer: &str) -> [Value; 2] {
    [
        json!({
            "role": "assistant",
            "tool_calls": [{
                "id": id,
                "type": "function",
                "function": {
                    "name": "request_user_input",
                    "arguments": "{\"questions\":[{\"multiple\":true}]}"
                }
            }]
        }),
        json!({
            "role": "tool",
            "tool_call_id": id,
            "name": "request_user_input",
            "content": answer,
        }),
    ]
}

/// The tool trio an `OpenCode` client advertises, as the reported session had.
fn opencode_tools() -> Value {
    json!([
        tool(
            "run_shell_command",
            &json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "description": {"type": "string"}
                },
                "required": ["command", "description"],
                "additionalProperties": false
            })
        ),
        tool(
            "request_user_input",
            &json!({
                "type": "object",
                "properties": {"questions": {"type": "array"}},
                "required": ["questions"]
            })
        ),
    ])
}

fn tool(name: &str, parameters: &Value) -> Value {
    json!({"type": "function", "function": {"name": name, "parameters": parameters}})
}

/// The whole journey: a real session id in, a complete report body out.
#[test]
fn the_generated_script_files_the_full_conversation_in_all_six_sections() {
    let session = "ses_06ac01b87ffeW5XnFmtYE8Amil";
    let workspace = Workspace::new("full");
    workspace.install_gh();
    workspace.record_conversation(session);

    let command = github_command(session);
    assert!(
        command.contains(&format!("--session '{session}'")),
        "the script must export the session the request declared:\n{command}"
    );

    let output = workspace.run(&command);
    assert!(
        output.status.success(),
        "the script failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(ISSUE_URL),
        "the script must print the URL its narration reports:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let body = workspace.filed_body().expect("gh received a body file");
    for section in SECTIONS {
        assert!(body.contains(section), "{section} is missing from:\n{body}");
    }
    // The conversation itself, not a summary of it and not a proxy frame.
    for (_, text) in TURNS {
        assert!(body.contains(text), "{text:?} is missing from:\n{body}");
    }
    assert!(
        body.contains(session),
        "the body must name the session it exported:\n{body}"
    );
    assert!(
        !contains_hashed_identifier(&body),
        "no hashed identifier may survive anywhere in the body:\n{body}"
    );

    // The title convention (§4) reaches GitHub with the body.
    let arguments = workspace.filed_arguments().expect("gh recorded its argv");
    assert!(
        arguments.contains("Formal AI: `Что такое фуфломицин?` + `Так что это такое то?`"),
        "{arguments}"
    );
}

/// Whether the text names a session the way #838 did: `dialog_` and 16 hex
/// digits, the shape `stable_id` produces from the first user message.
///
/// The metadata key `dialog_id` shares the prefix and is legitimate, so the
/// digits are what distinguishes an invented id from a recorded one.
fn contains_hashed_identifier(text: &str) -> bool {
    text.match_indices("dialog_").any(|(start, prefix)| {
        let tail = &text[start + prefix.len()..];
        tail.len() >= 16 && tail[..16].chars().all(|digit| digit.is_ascii_hexdigit())
    })
}

/// The exported context reaches the body whole, in conversation order.
#[test]
fn the_filed_body_round_trips_every_turn_of_a_multi_turn_conversation() {
    let session = "ses_multi_turn_round_trip";
    let workspace = Workspace::new("round-trip");
    workspace.install_gh();
    workspace.record_conversation(session);

    let output = workspace.run(&github_command(session));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body = workspace.filed_body().expect("gh received a body file");

    let mut position = 0;
    for (_, text) in TURNS {
        let found = body[position..]
            .find(text)
            .unwrap_or_else(|| panic!("{text:?} is out of order or missing in:\n{body}"));
        position += found + text.len();
    }
    // The attachment carries the export as Links Notation, not as base64 frames.
    assert!(body.contains("```lino"), "{body}");
    assert!(
        body.contains(&format!("conversation {session}")),
        "the attached context must be the conversation record:\n{body}"
    );
}

/// A missing program stops the script before anything is filed (§5, §7).
#[test]
fn a_missing_command_fails_loudly_and_files_nothing() {
    let session = "ses_missing_gh";
    let workspace = Workspace::new("missing-gh");
    // `gh` is deliberately not installed.
    workspace.record_conversation(session);

    let output = workspace.run(&github_command(session));
    assert!(
        !output.status.success(),
        "a script missing `gh` must not report success:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("gh"), "{diagnostic}");
    assert!(
        workspace.filed_body().is_none() && workspace.filed_arguments().is_none(),
        "nothing may be filed once a required command is missing"
    );
}

/// An empty export fails instead of filing an issue with an empty body (§5).
#[test]
fn an_unexportable_session_fails_before_gh_is_reached() {
    let session = "ses_never_recorded";
    let workspace = Workspace::new("empty");
    workspace.install_gh();
    // No conversation is recorded for this session, on either side.

    let output = workspace.run(&github_command(session));
    assert!(
        !output.status.success(),
        "an empty export must not end in a filed issue:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostic.contains(session),
        "the failure must name the session it could not export:\n{diagnostic}"
    );
    assert!(
        workspace.filed_arguments().is_none(),
        "`gh issue create` must never run after a failed export"
    );
}

/// Two sessions that opened with the same sentence stay separate (§2.1).
///
/// The identifier #838 asked for was an FNV hash of the first user message, so
/// two conversations starting with `Что такое фуфломицин?` produced the same
/// `--session` argument and the same export.
#[test]
fn two_conversations_sharing_a_first_message_file_different_contexts() {
    let first = "ses_first_reporter";
    let second = "ses_second_reporter";
    let workspace = Workspace::new("collision");
    workspace.install_gh();
    workspace.record_conversation(first);
    workspace.record_conversation(second);

    let first_command = github_command(first);
    let second_command = github_command(second);
    assert_ne!(
        first_command, second_command,
        "the same opening sentence must not produce the same export command"
    );
    assert!(first_command.contains(&format!("--session '{first}'")));
    assert!(second_command.contains(&format!("--session '{second}'")));

    let output = workspace.run(&first_command);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body = workspace.filed_body().expect("gh received a body file");
    assert!(body.contains(first), "{body}");
    assert!(
        !body.contains(second),
        "the report must carry one conversation only:\n{body}"
    );
}

/// Without a declared session the script resolves the live one, never a hash.
#[test]
fn a_request_without_a_session_header_asks_the_cli_to_resolve_it() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "messages": report_messages(),
        "tools": opencode_tools(),
    })
    .to_string();
    let response = handle_api_request_with_headers("POST", "/v1/chat/completions", &[], &body);
    let response: Value = serde_json::from_str(&response.body).expect("JSON response");
    let arguments: Value = serde_json::from_str(
        response["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"]
            .as_str()
            .expect("arguments"),
    )
    .expect("arguments");
    let command = arguments["command"].as_str().expect("shell command");
    assert!(command.contains("--session 'latest'"), "{command}");
    assert!(!command.contains("--session 'dialog_"), "{command}");
}

/// The scratch template is expanded by `mktemp`, not filed literally.
///
/// The gist attached to #838 was named `formal-ai-report.XXXXXX.lino`: GNU
/// `mktemp` only substitutes a trailing run of `X`s, so the suffixed template
/// reached GitHub unexpanded.
#[test]
fn the_scratch_template_is_expanded_before_anything_is_written() {
    let session = "ses_scratch_template";
    let workspace = Workspace::new("scratch");
    workspace.install_gh();
    workspace.record_conversation(session);

    let output = workspace.run(&github_command(session));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let arguments = workspace.filed_arguments().expect("gh recorded its argv");
    assert!(!arguments.contains("XXXXXX"), "{arguments}");
    let body_file = arguments
        .lines()
        .skip_while(|line| *line != "--body-file")
        .nth(1)
        .expect("gh was handed a body file");
    assert!(
        Path::new(body_file)
            .file_name()
            .is_some_and(|name| name == "body.md"),
        "the body file must carry a real name: {body_file}"
    );
}
