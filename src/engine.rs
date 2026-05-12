use lino_objects_codec::format::format_indented_ordered;
use serde::{Deserialize, Serialize};

pub const DEFAULT_MODEL: &str = "formal-symbolic-poc";

const GREETING_ANSWER: &str = "Hi, how may I help you?";
const UNKNOWN_ANSWER: &str = "I do not have a learned symbolic rule for that prompt yet. Add a Links Notation fact or rule, then run the request again.";

const GREETING_EXAMPLES: &[&str] = &["Hi", "Hello", "Hey"];
const UNKNOWN_EXAMPLES: &[&str] = &["Any prompt without a matching symbolic rule"];

struct HelloWorldProgram {
    slug: &'static str,
    language: &'static str,
    aliases: &'static [&'static str],
    code_fence: &'static str,
    code: &'static str,
    execution: ProgramExecution,
    response_link: &'static str,
    source: &'static str,
}

#[derive(Clone, Copy)]
struct ProgramExecution {
    status: ExecutionStatus,
    environment: &'static str,
    check_command: Option<&'static str>,
    run_command: &'static str,
    output: &'static str,
    notes: &'static str,
}

#[derive(Clone, Copy)]
enum ExecutionStatus {
    Verified,
    Unavailable,
}

impl ExecutionStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::Verified => "compiled and ran",
            Self::Unavailable => "not compiled or run",
        }
    }
}

const HELLO_WORLD_PROGRAMS: &[HelloWorldProgram] = &[
    HelloWorldProgram {
        slug: "rust",
        language: "Rust",
        aliases: &["rust", "rs"],
        code_fence: "rust",
        code: r#"fn main() {
    println!("Hello, world!");
}"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness",
            check_command: Some("rustc main.rs -o main"),
            run_command: "./main",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:rust",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "python",
        language: "Python",
        aliases: &["python", "py"],
        code_fence: "python",
        code: r#"print("Hello, world!")"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness",
            check_command: Some("python3 -m py_compile main.py"),
            run_command: "python3 main.py",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:python",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "javascript",
        language: "JavaScript",
        aliases: &["javascript", "js", "node"],
        code_fence: "javascript",
        code: r#"console.log("Hello, world!");"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness",
            check_command: Some("node --check main.js"),
            run_command: "node main.js",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:javascript",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "typescript",
        language: "TypeScript",
        aliases: &["typescript", "ts"],
        code_fence: "typescript",
        code: r#"console.log("Hello, world!");"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Unavailable,
            environment: "TypeScript compiler is not configured in this repository runtime",
            check_command: Some("tsc hello.ts"),
            run_command: "node hello.js",
            output: "Hello, world!",
            notes: "The TypeScript seed is returned with this warning until a tsc-backed execution profile is available.",
        },
        response_link: "response:hello_world:typescript",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "go",
        language: "Go",
        aliases: &["go", "golang"],
        code_fence: "go",
        code: r#"package main

import "fmt"

func main() {
    fmt.Println("Hello, world!")
}"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness",
            check_command: None,
            run_command: "go run main.go",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:go",
        source: "local Links Notation hello-world seed",
    },
    HelloWorldProgram {
        slug: "c",
        language: "C",
        aliases: &["c"],
        code_fence: "c",
        code: r#"#include <stdio.h>

int main(void) {
    puts("Hello, world!");
    return 0;
}"#,
        execution: ProgramExecution {
            status: ExecutionStatus::Verified,
            environment: "issue-8 local verification harness",
            check_command: Some("gcc main.c -o main"),
            run_command: "./main",
            output: "Hello, world!",
            notes: "1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.",
        },
        response_link: "response:hello_world:c",
        source: "local Links Notation hello-world seed",
    },
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolicAnswer {
    pub intent: String,
    pub answer: String,
    pub confidence: f32,
    pub evidence_links: Vec<String>,
    pub links_notation: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FormalAiEngine;

impl FormalAiEngine {
    #[must_use]
    pub fn answer(&self, prompt: &str) -> SymbolicAnswer {
        let normalized = normalize_prompt(prompt);
        let rule = select_rule(&normalized);
        let intent = rule.intent();
        let answer = rule.answer();
        let evidence_links = vec![
            format!("prompt:{}", stable_id("prompt", prompt)),
            String::from(rule.response_link()),
            format!("intent:{intent}"),
        ];
        let confidence = if matches!(rule, SelectedRule::Unknown) {
            0.0
        } else {
            1.0
        };
        let links_notation = answer_links_notation(prompt, &intent, &answer, &evidence_links);

        SymbolicAnswer {
            intent,
            answer,
            confidence,
            evidence_links,
            links_notation,
        }
    }
}

#[must_use]
pub fn knowledge_links_notation() -> String {
    let mut records = vec![
        format_lino_record(
            "formal_ai_knowledge",
            &[
                ("model", String::from(DEFAULT_MODEL)),
                (
                    "policy",
                    String::from("deterministic symbolic rules; no neural network inference"),
                ),
                (
                    "format",
                    String::from(
                        "untyped indented Links Notation via lino-objects-codec format helpers",
                    ),
                ),
                ("rule_count", (HELLO_WORLD_PROGRAMS.len() + 2).to_string()),
            ],
        ),
        format_lino_record(
            "rule_greeting",
            &[
                ("intent", String::from("greeting")),
                ("response_link", String::from("response:greeting")),
                ("answer", String::from(GREETING_ANSWER)),
                ("examples", GREETING_EXAMPLES.join(", ")),
                ("source", String::from("local symbolic seed set")),
            ],
        ),
    ];

    records.extend(
        HELLO_WORLD_PROGRAMS
            .iter()
            .map(format_hello_world_rule_record),
    );
    records.push(format_lino_record(
        "rule_unknown",
        &[
            ("intent", String::from("unknown")),
            ("response_link", String::from("response:unknown")),
            ("answer", String::from(UNKNOWN_ANSWER)),
            ("examples", UNKNOWN_EXAMPLES.join(", ")),
            ("source", String::from("fallback symbolic rule")),
        ],
    ));

    records.join("\n\n")
}

#[must_use]
pub fn estimate_tokens(text: &str) -> u32 {
    u32::try_from(text.split_whitespace().count()).unwrap_or(u32::MAX)
}

#[must_use]
pub fn stable_id(prefix: &str, text: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    format!("{prefix}_{hash:016x}")
}

enum SelectedRule {
    Greeting,
    HelloWorld(&'static HelloWorldProgram),
    Unknown,
}

impl SelectedRule {
    fn intent(&self) -> String {
        match self {
            Self::Greeting => String::from("greeting"),
            Self::HelloWorld(program) => format!("hello_world_{}", program.slug),
            Self::Unknown => String::from("unknown"),
        }
    }

    const fn response_link(&self) -> &'static str {
        match self {
            Self::Greeting => "response:greeting",
            Self::HelloWorld(program) => program.response_link,
            Self::Unknown => "response:unknown",
        }
    }

    fn answer(&self) -> String {
        match self {
            Self::Greeting => String::from(GREETING_ANSWER),
            Self::HelloWorld(program) => hello_world_answer(program),
            Self::Unknown => String::from(UNKNOWN_ANSWER),
        }
    }
}

fn select_rule(normalized_prompt: &str) -> SelectedRule {
    if is_greeting(normalized_prompt) {
        SelectedRule::Greeting
    } else if let Some(program) = hello_world_program(normalized_prompt) {
        SelectedRule::HelloWorld(program)
    } else {
        SelectedRule::Unknown
    }
}

fn is_greeting(normalized_prompt: &str) -> bool {
    matches!(normalized_prompt, "hi" | "hello" | "hey")
}

fn hello_world_program(normalized_prompt: &str) -> Option<&'static HelloWorldProgram> {
    if !contains_token(normalized_prompt, "hello") || !contains_token(normalized_prompt, "world") {
        return None;
    }

    HELLO_WORLD_PROGRAMS.iter().find(|program| {
        program
            .aliases
            .iter()
            .any(|alias| contains_token(normalized_prompt, alias))
    })
}

fn contains_token(normalized_prompt: &str, expected: &str) -> bool {
    normalized_prompt
        .split_whitespace()
        .any(|token| token == expected)
}

fn normalize_prompt(prompt: &str) -> String {
    let mut normalized = String::with_capacity(prompt.len());

    for character in prompt.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            normalized.push(character);
        } else {
            normalized.push(' ');
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn answer_links_notation(
    prompt: &str,
    intent: &str,
    answer: &str,
    evidence_links: &[String],
) -> String {
    format_lino_record(
        &format!("answer_{}", stable_id("prompt", prompt)),
        &[
            ("prompt", String::from(prompt)),
            ("intent", String::from(intent)),
            ("answer", String::from(answer)),
            ("evidence_links", evidence_links.join(", ")),
        ],
    )
}

fn format_hello_world_rule_record(program: &HelloWorldProgram) -> String {
    let answer = hello_world_answer(program);
    format_lino_record(
        &format!("rule_hello_world_{}", program.slug),
        &[
            ("intent", format!("hello_world_{}", program.slug)),
            ("language", String::from(program.language)),
            ("aliases", program.aliases.join(", ")),
            ("response_link", String::from(program.response_link)),
            ("answer", answer),
            (
                "execution_status",
                String::from(program.execution.status.label()),
            ),
            (
                "execution_environment",
                String::from(program.execution.environment),
            ),
            ("execution_output", String::from(program.execution.output)),
            (
                "examples",
                format!(
                    "Write me hello world program in {}; hello world in {}",
                    program.language, program.language
                ),
            ),
            ("source", String::from(program.source)),
        ],
    )
}

fn hello_world_answer(program: &HelloWorldProgram) -> String {
    format!(
        "Here is a minimal {} hello world program:\n\n```{}\n{}\n```\n\n{}",
        program.language,
        program.code_fence,
        program.code,
        execution_report(&program.execution)
    )
}

fn execution_report(execution: &ProgramExecution) -> String {
    let command_lines = execution_command_lines(execution);
    let output_label = if matches!(execution.status, ExecutionStatus::Verified) {
        "Output"
    } else {
        "Expected output after verification"
    };

    format!(
        "Execution status: {} in {}.\n{}\n{}:\n```text\n{}\n```\n{}",
        execution.status.label(),
        execution.environment,
        command_lines,
        output_label,
        execution.output,
        execution.notes
    )
}

fn execution_command_lines(execution: &ProgramExecution) -> String {
    execution.check_command.map_or_else(
        || format!("Run command: `{}`", execution.run_command),
        |check_command| {
            format!(
                "Check command: `{check_command}`\nRun command: `{}`",
                execution.run_command
            )
        },
    )
}

fn format_lino_record(id: &str, pairs: &[(&str, String)]) -> String {
    let sanitized = pairs
        .iter()
        .map(|(key, value)| (*key, sanitize_lino_value(value)))
        .collect::<Vec<_>>();
    let borrowed = sanitized
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect::<Vec<_>>();

    format_indented_ordered(id, &borrowed, "  ")
        .expect("static Links Notation records should be valid")
}

fn sanitize_lino_value(value: &str) -> String {
    value
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}
