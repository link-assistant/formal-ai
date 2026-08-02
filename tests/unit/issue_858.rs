//! Regression coverage for issue #858: Claude Code's `/recap` away summary.

use std::process::Command;

use formal_ai::seed::{self, ROLE_CONVERSATION_RETURN_RECAP};
use formal_ai::summarization::{summarize_dialog_plain, DialogTurn};
use formal_ai::{
    create_anthropic_message_with_solver, solve_with_history, AnthropicContentBlock,
    AnthropicMessagesRequest, ConversationTurn, SolverConfig, UniversalSolver,
};

const CLAUDE_AWAY_RECAP_PROMPT: &str = "The user stepped away and is coming back. Recap in under 40 words, 1-2 plain sentences, no markdown. Lead with the overall goal and current task, then the one next action. Skip root-cause narrative, fix internals, secondary to-dos, and em-dash tangents.";

fn agent_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

fn response_text(content: &[AnthropicContentBlock]) -> String {
    content
        .iter()
        .filter_map(|block| match block {
            AnthropicContentBlock::Text { text } => Some(text.as_str()),
            AnthropicContentBlock::Thinking { .. } | AnthropicContentBlock::ToolUse { .. } => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn sentence_count(text: &str) -> usize {
    let mut count = 0;
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if matches!(character, '。' | '！' | '？')
            || (matches!(character, '.' | '!' | '?')
                && characters.peek().is_none_or(|next| next.is_whitespace()))
        {
            count += 1;
        }
    }
    count
}

#[test]
fn claude_code_away_recap_returns_a_bounded_plain_summary() {
    let request: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 1024,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "<system-reminder>\nAs you answer the user's questions, you can use the following context:\n# currentDate\nToday's date is 2026-08-02.\n\nIMPORTANT: this context may not be relevant to the task.\n</system-reminder>\n\n"
                    },
                    {
                        "type": "text",
                        "text": "Create and verify a Rust hello-world program in main.rs."
                    }
                ]
            },
            {
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "I will write main.rs and run it."},
                    {
                        "type": "tool_use",
                        "id": "toolu_write",
                        "name": "Write",
                        "input": {"file_path": "main.rs", "content": "fn main() { println!(\"Hello, world!\"); }"}
                    }
                ]
            },
            {
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "toolu_write",
                    "content": "File created successfully at: main.rs"
                }]
            },
            {
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "I will compile and execute the program."},
                    {
                        "type": "tool_use",
                        "id": "toolu_bash",
                        "name": "Bash",
                        "input": {"command": "rustc main.rs && ./main"}
                    }
                ]
            },
            {
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "toolu_bash",
                    "content": "Hello, world!"
                }]
            },
            {
                "role": "assistant",
                "content": "The Rust hello-world program in main.rs is complete and verified."
            },
            {"role": "user", "content": CLAUDE_AWAY_RECAP_PROMPT}
        ],
        "tools": [
            {
                "name": "Write",
                "description": "Write a file",
                "input_schema": {"type": "object"}
            },
            {
                "name": "Bash",
                "description": "Run a shell command",
                "input_schema": {"type": "object"}
            }
        ]
    }))
    .expect("valid Anthropic Messages request");

    let message = create_anthropic_message_with_solver(&request, &agent_solver());
    let text = response_text(&message.content);
    let lowercase = text.to_lowercase();
    let sentence_count = sentence_count(&text);

    assert_eq!(
        message.stop_reason, "end_turn",
        "recap must not call a tool"
    );
    assert!(
        message
            .content
            .iter()
            .all(|block| !matches!(block, AnthropicContentBlock::ToolUse { .. })),
        "recap must be text-only: {message:?}"
    );
    assert!(
        lowercase.contains("rust") && lowercase.contains("main.rs"),
        "recap should retain the overall goal: {text}"
    );
    assert!(
        lowercase.contains("complete") || lowercase.contains("verified"),
        "recap should retain the current status: {text}"
    );
    assert!(
        !lowercase.contains("following context")
            && !lowercase.contains("currentdate")
            && !lowercase.contains("today's date"),
        "client-injected context must not displace the user's goal: {text}"
    );
    assert!(
        text.split_whitespace().count() < 40,
        "Claude requests fewer than 40 words: {text}"
    );
    assert!(
        (1..=2).contains(&sentence_count),
        "Claude requests one or two plain sentences: {text}"
    );
    assert!(
        !text.contains(['#', '`'])
            && text
                .lines()
                .all(|line| !line.trim_start().starts_with(['-', '*'])),
        "Claude requests no markdown: {text}"
    );
}

#[test]
fn returning_user_recap_is_a_multilingual_semantic_role() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    for Case { language, prompt } in [
        Case {
            language: "en",
            prompt: "i am back after stepping away",
        },
        Case {
            language: "ru",
            prompt: "я вернулся после перерыва",
        },
        Case {
            language: "hi",
            prompt: "मैं विराम के बाद वापस आया हूँ",
        },
        Case {
            language: "zh",
            prompt: "我离开后回来了",
        },
        Case {
            language: "es",
            prompt: "he vuelto después de ausentarme",
        },
    ] {
        assert!(
            seed::lexicon().mentions_role(ROLE_CONVERSATION_RETURN_RECAP, prompt),
            "missing returning-user recap role for {language}: {prompt}"
        );
    }
}

#[test]
fn plain_dialog_summary_removes_markdown_and_honors_budgets() {
    let turns = vec![
        DialogTurn::user(
            "Implement the compact recap path in `src/anthropic.rs` and verify it with tests.",
        ),
        DialogTurn::assistant(
            "## Current status\n\n- The implementation is complete and the targeted regression passes.\n- A secondary cleanup can wait.",
        ),
    ];

    let summary = summarize_dialog_plain(&turns, 20, 2);

    assert!(summary.split_whitespace().count() <= 20, "{summary}");
    assert!(!summary.contains(['#', '`', '*']), "{summary}");
    assert!(summary.contains("compact recap"), "{summary}");
    assert!(summary.contains("implementation is complete"), "{summary}");
}

#[test]
fn ordinary_summary_keeps_the_existing_detailed_report() {
    let history = [
        ConversationTurn::user("Create and verify a Rust hello-world program in main.rs."),
        ConversationTurn::assistant(
            "The Rust hello-world program in main.rs is complete and verified.",
        ),
    ];

    let answer = solve_with_history("Summarize", &history);

    assert_eq!(answer.intent, "summarize_conversation");
    assert!(answer.answer.starts_with("Conversation summary:"));
    assert!(answer.answer.contains("Title:"));
    assert!(answer.answer.contains("User turns:"));
}

#[test]
fn browser_worker_matches_the_rust_recap_contract() {
    let output = Command::new("node")
        .arg("experiments/issue-858-worker-recap-parity.mjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run browser-worker recap parity harness");

    assert!(
        output.status.success(),
        "worker parity failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
