//! Issue #907: the caller's framing is not the user's request.
//!
//! The gemini CLI prefixes every turn with a `<session_context>` block whose
//! second line reads *"Today's date is Sunday, August 2, 2026 (formatted
//! according to the user's locale)."* Intent routing matched that sentence
//! anywhere in the incoming request, so **every** agent-mode gemini run emitted
//! `run_shell_command({"command":"date"})` and silently dropped the real task.
//!
//! One test per requirement of the issue:
//!
//! 1. intent matching reads the user's request, not client-injected context;
//! 2. a declarative *"X is Y"* statement never fires an intent, in any language;
//! 3. a turn that carries a task gets the task, even when an intent also matches;
//!
//! followed by the whole-task test: the report's exact gemini request, over the
//! real Gemini surface, must write the requested program.
//!
//! # The unmarked-preamble variant
//!
//! Reported against 0.339.1 and 0.345.0 after the fix above shipped
//! ([comment](https://github.com/link-assistant/formal-ai/issues/907#issuecomment-5302919925),
//! upstream [hive-mind#2158](https://github.com/link-assistant/hive-mind/issues/2158)).
//! The fix above separates the caller from the user by the *markup* the client
//! wraps its framing in. Hive Mind's Agent/Codex/Gemini/OpenCode adapters wrapped
//! theirs in nothing: workflow policy and objective arrived concatenated into one
//! untagged `user` message. Production then observed Codex receive
//! `/bin/bash -lc pwd` and exit 0, and five Codex attempts run bare `sudo`, with
//! no requested repository file created in any of them.
//!
//! Two more requirements, from the same comment:
//!
//! 4. a later explicit objective delimiter outranks an earlier unmarked
//!    preamble — the objective is the request, the preamble is framing;
//! 5. a policy sentence that merely *mentions* a privileged command does not
//!    authorize it; an imperative naming it still does.
//!
//! Each case uses a *different* phrasing so a passing run proves the routing is
//! general rather than memorised (CONTRIBUTING rule 4).

use formal_ai::gemini::{
    GeminiGenerateContentRequest, create_gemini_generate_content_response_with_solver_and_memory,
};
use formal_ai::protocol::{ChatMessage, latest_user_request};
use formal_ai::seed;
use formal_ai::{SolverConfig, UniversalSolver};

/// The two capabilities the report's gemini session advertises.
const TOOLS: [&str; 2] = ["run_shell_command", "write_file"];

const REQUEST: &str = "Write a hello world program in Python.";

/// The context block the gemini CLI really sends, verbatim from the report.
const GEMINI_SESSION_CONTEXT: &str = "<session_context>\nThis is the Gemini CLI. We are setting up the context for our chat.\nToday's date is Sunday, August 2, 2026 (formatted according to the user's locale).\nMy operating system is: linux\n</session_context>";

fn agent_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

/// The declarations the report's gemini session advertises, as the CLI sends them.
fn function_declarations() -> serde_json::Value {
    serde_json::json!([{"functionDeclarations": [
        {
            "name": TOOLS[0],
            "description": "Run a shell command",
            "parameters": {
                "type": "OBJECT",
                "properties": {"command": {"type": "STRING"}},
                "required": ["command"]
            }
        },
        {
            "name": TOOLS[1],
            "description": "Write a file",
            "parameters": {
                "type": "OBJECT",
                "properties": {
                    "file_path": {"type": "STRING"},
                    "content": {"type": "STRING"}
                },
                "required": ["file_path", "content"]
            }
        }
    ]}])
}

/// The function call the Gemini surface emits for a turn made of `parts`.
///
/// Routing is exercised over the surface the report uses, so a passing test is
/// evidence about the reported behaviour rather than about an inner helper.
fn gemini_call(parts: &[&str]) -> serde_json::Value {
    let request: GeminiGenerateContentRequest = serde_json::from_value(serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": parts.iter().map(|text| serde_json::json!({"text": text})).collect::<Vec<_>>(),
        }],
        "tools": function_declarations(),
    }))
    .expect("valid Gemini generateContent request");

    let response = create_gemini_generate_content_response_with_solver_and_memory(
        &request,
        "formal-ai",
        &agent_solver(),
        &[],
    );
    response["candidates"][0]["content"]["parts"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|part| part.get("functionCall"))
        .cloned()
        .unwrap_or_else(|| panic!("expected a function call for {parts:?}, got {response}"))
}

/// Requirement 1: what a client wraps in its own context block is the client
/// talking. Every registered marker is checked with a different client's real
/// framing, so the separation is not one CLI's special case.
#[test]
fn client_injected_context_is_not_the_users_request() {
    for (tag, context) in [
        (
            "session_context",
            "This is the Gemini CLI. We are setting up the context for our chat.\nToday's date is Sunday, August 2, 2026 (formatted according to the user's locale).",
        ),
        (
            "system-reminder",
            "As you answer the user's questions, you can use the following context:\n# currentDate\nToday's date is 2026-08-02.",
        ),
        (
            "environment_context",
            "<cwd>/tmp/workspace</cwd>\n<approval_policy>on-request</approval_policy>",
        ),
        (
            "env",
            "Working directory: /tmp/workspace\nToday's date: 2026-08-02",
        ),
        (
            "environment_details",
            "# Current Time\n2026-08-02T20:00:00Z\n# Current Working Directory\n/tmp/workspace",
        ),
    ] {
        let framed = format!("<{tag}>\n{context}\n</{tag}>\n\n{REQUEST}");
        let messages = vec![ChatMessage::user(&framed)];

        assert_eq!(
            latest_user_request(&messages).as_deref(),
            Some(REQUEST),
            "<{tag}> is the client talking, not the user"
        );
        let call = gemini_call(&[&framed]);
        assert_eq!(
            call["name"], "write_file",
            "<{tag}> must not decide the turn"
        );
    }
}

/// Requirement 1, seed side: the markers are declared as data, so a maintainer
/// adds the next client by editing `data/seed/caller-context.lino`.
#[test]
fn caller_context_markers_live_in_seed_data() {
    let vocabulary = seed::caller_context_vocabulary();
    for (tag, client) in [
        ("session_context", "gemini"),
        ("system-reminder", "claude"),
        ("system-reminder", "qwen"),
        ("environment_context", "codex"),
        ("env", "opencode"),
        ("environment_details", "cline"),
    ] {
        let block = vocabulary
            .injected_blocks
            .iter()
            .find(|block| block.tag == tag)
            .unwrap_or_else(|| panic!("missing injected block {tag}"));
        assert_eq!(block.open(), format!("<{tag}>"));
        assert_eq!(block.close(), format!("</{tag}>"));
        assert!(
            block.clients.iter().any(|known| known == client),
            "{tag} should record {client}"
        );
    }
}

/// Requirement 2: stating a fact about the date is not asking for it — in any
/// of the languages the date intent is declared in.
#[test]
fn a_declarative_statement_does_not_fire_an_intent() {
    for (language, statement) in [
        (
            "english",
            "Today's date is Sunday, August 2, 2026 (formatted according to the user's locale).",
        ),
        ("english", "The current date is 2026-08-02."),
        ("english", "The current time is 20:00."),
        ("russian", "Текущее время — 20:00."),
        ("hindi", "आज की तारीख 2 अगस्त 2026 है।"),
        ("chinese", "今天的日期是2026年8月2日。"),
        ("spanish", "La fecha de hoy es domingo."),
    ] {
        let call = gemini_call(&[&format!("{statement}\n\n{REQUEST}")]);
        assert_eq!(
            call["name"], "write_file",
            "a {language} statement must not route: {statement:?} planned {call}"
        );
    }
}

/// Requirement 2, the other direction: really asking still routes to `date`.
/// A guard that silenced the intent altogether would pass the test above.
#[test]
fn asking_for_the_date_still_runs_date() {
    for (language, question) in [
        ("english", "what is the date?"),
        ("english", "show me today's date"),
        ("english", "what day is it"),
        ("english", "print the current date"),
        ("russian", "какое сегодня число?"),
        ("hindi", "आज की तारीख क्या है?"),
        ("chinese", "今天的日期是什么？"),
        ("spanish", "¿cuál es la fecha de hoy?"),
    ] {
        let call = gemini_call(&[question]);
        assert_eq!(call["name"], "run_shell_command", "{language}: {question}");
        assert_eq!(call["args"]["command"], "date", "{language}: {question}");
    }
}

/// Requirement 3: when a turn carries a task, the task wins — an intent cue
/// riding along in the same turn does not replace it.
#[test]
fn a_turn_that_carries_a_task_gets_the_task() {
    for prompt in [
        "Show me the current date. Write a hello world program in Python please.",
        "Today's date is Sunday. Create a file main.py that prints Hello, world!",
        "The current time is 20:00. Write a Python script that prints Hello, world!",
    ] {
        let call = gemini_call(&[prompt]);
        assert_eq!(call["name"], "write_file", "{prompt} planned {call}");
        assert!(
            call["args"]["content"]
                .as_str()
                .is_some_and(|content| content.contains("Hello, world!")),
            "{prompt} planned {call}"
        );
    }
}

/// The Hive Mind preamble shape, verbatim from the reproduction in the #907
/// follow-up comment: workflow policy, a workspace path, then the objective —
/// all in one `user` message with no markup separating them.
const HIVE_MIND_PREAMBLE: &str = "\
You are an AI issue solver.
When running sudo commands, run them in the background.
Your prepared working directory: /tmp/example

Issue to solve: https://github.com/example/example/issues/1
Your prepared branch: issue-1-example
Your prepared Pull Request: https://github.com/example/example/pull/2

Proceed.";

/// Requirement 4: an unmarked preamble is still the caller talking. Everything
/// before the explicit objective delimiter is framing; the objective after it is
/// the request.
///
/// The production reproduction is the assertion: this exact message made Codex
/// receive `/bin/bash -lc pwd` and exit 0 without planning the repository work.
/// Nothing in the preamble may become the command, and neither of the two
/// commands the corpus recorded — `pwd` from *"working directory"*, `sudo` from
/// the policy sentence — may be planned at all.
#[test]
fn an_unmarked_caller_preamble_does_not_outrank_a_later_objective() {
    let planned = planned_command(HIVE_MIND_PREAMBLE);

    assert_ne!(
        planned.as_deref(),
        Some("pwd"),
        "the caller's workspace path is not a request for the working directory",
    );
    assert_ne!(
        planned.as_deref(),
        Some("sudo"),
        "the caller's sudo policy is not a request to run sudo",
    );
    assert_eq!(
        formal_ai::agentic_coding::general_planner::objective_text(HIVE_MIND_PREAMBLE),
        "https://github.com/example/example/issues/1\n\
         Your prepared branch: issue-1-example\n\
         Your prepared Pull Request: https://github.com/example/example/pull/2\n\n\
         Proceed.",
        "the objective is the text after the delimiter, not the preamble before it",
    );
}

/// Requirement 4, the boundary itself: the delimiter is what separates them, so
/// the same preamble with the objective removed still plans no command from the
/// framing alone. A guard keyed to the preamble's exact words would pass the
/// test above and fail this one.
#[test]
fn caller_framing_alone_never_becomes_a_command() {
    for framing in [
        "You are an AI issue solver.\nWhen running sudo commands, run them in the background.",
        "Before you execute rm, ask the operator first.",
        "Whenever you run docker, pass --rm so containers are not left behind.",
        "Never run chmod on files outside the workspace.",
        "Your prepared working directory: /tmp/example",
    ] {
        assert_eq!(
            planned_command(framing),
            None,
            "caller framing planned a command: {framing:?}",
        );
    }
}

/// Sentence scoping is what stops run context leaking across a boundary, and
/// this pins the boundary itself: an explicitly introduced command still passes
/// through whole, because the passthrough prefix claims it before sentence
/// matching is ever consulted.
#[test]
fn an_explicitly_introduced_command_survives_sentence_scoping() {
    for (request, expected) in [
        ("run git status", "git status"),
        ("execute pwd", "pwd"),
        ("run cat notes.txt", "cat notes.txt"),
    ] {
        assert_eq!(
            planned_command(request).as_deref(),
            Some(expected),
            "{request:?} names its command explicitly and must pass through",
        );
    }
}

/// Requirement 5, the other direction: a user who actually asks for one of these
/// commands still gets it. A guard that simply refused every privileged token
/// would pass the two tests above while breaking the feature.
#[test]
fn an_imperative_still_selects_the_command_it_names() {
    for (request, expected) in [
        ("run sudo", "sudo"),
        ("execute chmod", "chmod"),
        ("execute docker", "docker"),
        ("print the current working directory", "pwd"),
    ] {
        assert_eq!(
            planned_command(request).as_deref(),
            Some(expected),
            "{request:?} names {expected} and must still select it",
        );
    }
}

/// A harness declares where the workspace is; a user asks for something. Both
/// use a colon, so the value decides which is which — a declaration names one
/// absolute path, because that is the only kind worth stating.
#[test]
fn a_declared_absolute_path_is_not_a_request() {
    for declaration in [
        "Your prepared working directory: /tmp/example",
        "current directory: /srv/app",
    ] {
        assert_eq!(
            planned_command(declaration),
            None,
            "a declared workspace path planned a command: {declaration:?}",
        );
    }
}

/// A request may punctuate itself with a colon, and a great many do. Earlier
/// attempts at this guard keyed on the colon alone, then on a four-verb request
/// list, then on subject position, and each one silently swallowed some of
/// these.
///
/// The first two rows are the load-bearing ones: `count` and `create` belong to
/// no request-verb list, so a guard keyed on such a list drops them while the
/// `print`/`show` rows below sail through. Keeping both kinds here is what makes
/// the case adversarial rather than self-confirming — the value after the colon
/// is the only signal that separates all four from a caller declaration.
#[test]
fn a_request_that_punctuates_itself_with_a_colon_still_routes() {
    for request in [
        "count lines: Cargo.toml",
        "create a directory called: build",
        "print the current directory: I need the absolute path",
        "show me the working directory: then list its files",
    ] {
        assert!(
            planned_command(request).is_some(),
            "{request:?} is a request, not a caller declaration",
        );
    }
}

/// Policy filtering is whole-prompt, so a request arriving *alongside* policy
/// still reaches the router. Without this, the conservative filter would silence
/// every turn a caller prefixed with its rules.
#[test]
fn a_request_beside_a_policy_sentence_still_routes() {
    assert_eq!(
        planned_command("Never run rm outside the workspace. Now execute pwd.").as_deref(),
        Some("pwd"),
        "policy in one sentence must not silence the request in the next",
    );
}

/// Every seeded policy lead, in every language it is declared in. Asserting the
/// seed *contains* a lead proves nothing about routing, which is what the
/// earlier version of this file did.
///
/// Two prompts per language, covering english, russian, chinese, hindi and
/// spanish. The non-Latin scripts announce themselves, but spanish shares its
/// alphabet with english, so it is named here as well as written below.
#[test]
fn every_policy_lead_suppresses_its_command_in_every_language() {
    for policy in [
        // english
        "If you run rm, ask the operator first",
        "After you run docker, clean up the containers",
        "While running chmod, stay inside the workspace",
        "Unless asked, never touch the workspace root",
        "Always run docker with --rm",
        "Do not run rm outside the workspace",
        // russian
        "Когда запускаешь sudo, делай это в фоне",
        "Никогда не запускай rm вне рабочей папки",
        // chinese
        "如果 运行 docker，请加 --rm",
        "当 运行 sudo 时，请在后台执行",
        // hindi
        "जब sudo चलाओ, तो उसे पृष्ठभूमि में रखो",
        "कभी भी कार्यस्थान के बाहर rm मत चलाओ",
        // spanish
        "Nunca ejecutes rm fuera del espacio de trabajo",
        "Siempre ejecuta docker con --rm",
    ] {
        assert_eq!(
            planned_command(policy),
            None,
            "policy planned a command: {policy:?}",
        );
    }
}

/// The other half of the same rule, and the one that matters most to a user: a
/// request is still a request when it carries a condition. An earlier version of
/// this filter keyed on the opening word alone and answered these with nothing
/// at all, which is worse than answering imperfectly — the user asked for
/// something and got silence.
///
/// The separator is where the command is named. A conditional clause ends at its
/// comma; what follows is the order the condition qualifies.
#[test]
fn a_conditional_request_still_selects_the_command_it_orders() {
    for (request, expected) in [
        ("If the build fails, run cargo test", "cargo test"),
        ("After you finish, run git status", "git status"),
        ("When you are ready, execute pwd", "pwd"),
        ("Before anything else, run ls", "ls"),
    ] {
        assert_eq!(
            planned_command(request).as_deref(),
            Some(expected),
            "{request:?} orders {expected} after its condition and must select it",
        );
    }
}

/// `run` and `execute` are ordinary English verbs, so a sentence *about*
/// commands reaches the passthrough branch exactly like a real command does.
/// Passing its prose through produced commands such as `all tests in the
/// background`.
#[test]
fn a_sentence_about_commands_is_not_passed_through_as_one() {
    for sentence in [
        "Run all tests in the background",
        "Execute nothing without asking the operator first",
        "Run commands with sudo only when necessary",
        "Execute everything in the workspace",
    ] {
        assert_eq!(
            planned_command(sentence),
            None,
            "prose was passed through as a command: {sentence:?}",
        );
    }
}

/// The guard may not become a binary allowlist: the passthrough exists so a
/// client can run a command this crate has never heard of, and the sandbox — not
/// a maintained list — decides whether it may.
#[test]
fn an_unknown_binary_still_passes_through() {
    for (request, expected) in [
        ("run mytool --flag", "mytool --flag"),
        ("execute ./build.sh", "./build.sh"),
        ("run npm run build", "npm run build"),
        ("run git log --oneline -10", "git log --oneline -10"),
    ] {
        assert_eq!(
            planned_command(request).as_deref(),
            Some(expected),
            "{request:?} names a command line and must pass through verbatim",
        );
    }
}

/// Requirement 5, seed side: the policy leads are data, so a maintainer adds the
/// next conditional form by editing `data/seed/caller-context.lino` rather than
/// this crate.
#[test]
fn policy_leads_live_in_seed_data() {
    let leads = seed::caller_context_vocabulary().policy_leads;
    for lead in ["when", "if", "never", "когда", "如果"] {
        assert!(
            leads.iter().any(|known| known == lead),
            "policy lead {lead:?} should be declared in seed data: {leads:?}",
        );
    }
}

/// The command the Gemini surface plans for `prompt`, or [`None`] when it plans
/// something other than a shell call.
fn planned_command(prompt: &str) -> Option<String> {
    let request: GeminiGenerateContentRequest = serde_json::from_value(serde_json::json!({
        "contents": [{"role": "user", "parts": [{"text": prompt}]}],
        "tools": function_declarations(),
    }))
    .expect("valid Gemini generateContent request");

    let response = create_gemini_generate_content_response_with_solver_and_memory(
        &request,
        "formal-ai",
        &agent_solver(),
        &[],
    );
    response["candidates"][0]["content"]["parts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|part| part.get("functionCall"))
        .find(|call| call["name"] == TOOLS[0])
        .and_then(|call| call["args"]["command"].as_str())
        .map(str::to_owned)
}

/// The whole task: the report's request, over the real Gemini surface, with the
/// caller framing the gemini CLI actually sends.
#[test]
fn the_gemini_cli_session_context_no_longer_hijacks_the_request() {
    let call = gemini_call(&[GEMINI_SESSION_CONTEXT, REQUEST]);

    assert_eq!(call["name"], "write_file", "{call}");
    assert_eq!(call["args"]["file_path"], "main.py", "{call}");
    assert!(
        call["args"]["content"]
            .as_str()
            .is_some_and(|content| content.contains("Hello, world!")),
        "{call}"
    );
}
