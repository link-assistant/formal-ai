//! Regression coverage for issue #715: agentic code requests must mutate the
//! workspace through the CLI's file tools instead of returning stale prose.

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};

fn one_call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::ToolCalls(mut calls)) => {
            assert_eq!(calls.len(), 1, "expected one tool call: {calls:?}");
            calls.remove(0)
        }
        other => panic!("expected one tool call, got {other:?}"),
    }
}

fn args(call: &PlannedToolCall) -> serde_json::Value {
    serde_json::from_str(&call.arguments).expect("tool arguments must be JSON")
}

#[test]
fn code_generation_writes_a_real_workspace_file_for_every_catalog_language() {
    for (language, path, code_fragment) in [
        ("Rust", "main.rs", "fn main()"),
        ("Python", "main.py", "print("),
        ("JavaScript", "main.js", "console.log"),
        ("TypeScript", "hello.ts", "console.log"),
        ("Go", "main.go", "package main"),
        ("C", "main.c", "int main"),
        ("C++", "main.cpp", "int main"),
        ("Java", "Main.java", "class Main"),
        ("C#", "Program.cs", "class Program"),
        ("Ruby", "main.rb", "puts"),
    ] {
        let prompt = format!("Give me a hello world program in {language}");
        let call = one_call(&[ChatMessage::user(&prompt)], &["read", "write"]);
        assert_eq!(call.tool, "write", "{language}: {prompt}");
        let value = args(&call);
        assert_eq!(value["filePath"], path, "{language}: {value}");
        assert!(
            value["content"]
                .as_str()
                .is_some_and(|source| source.contains(code_fragment)),
            "{language}: generated source missing {code_fragment:?}: {value}"
        );
    }

    // OpenCode serializes its positional prompt with surrounding JSON quotes.
    let call = one_call(
        &[ChatMessage::user(
            "\"Give me a hello world program in Rust\"",
        )],
        &["read", "write"],
    );
    assert_eq!(call.tool, "write");
}

#[test]
fn contextual_change_rewrites_workspace_source_for_every_catalog_language() {
    for language in [
        "Rust",
        "Python",
        "JavaScript",
        "TypeScript",
        "Go",
        "C",
        "C++",
        "Java",
        "C#",
        "Ruby",
    ] {
        let initial = format!("Give me a hello world program in {language}");
        let generated = one_call(&[ChatMessage::user(&initial)], &["read", "write"]);
        let generated_args = args(&generated);
        let path = generated_args["filePath"].as_str().unwrap().to_owned();
        let source = generated_args["content"].as_str().unwrap().to_owned();
        let mut messages = vec![
            ChatMessage::user(initial),
            ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                "write-initial".to_owned(),
                "write".to_owned(),
                generated.arguments,
            )]),
            ChatMessage::tool_result("write-initial", "write", format!("Wrote {path}")),
            ChatMessage::assistant(format!("Wrote `{path}`.")),
            ChatMessage::user("Change the output message to `Hello 2`."),
        ];

        let read = one_call(&messages, &["read", "write"]);
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "read-current".to_owned(),
            "read".to_owned(),
            read.arguments,
        )]));
        messages.push(ChatMessage::tool_result(
            "read-current",
            "read",
            source.clone(),
        ));

        let write = one_call(&messages, &["read", "write"]);
        let value = args(&write);
        assert_eq!(value["filePath"], path, "{language}");
        let updated = value["content"].as_str().unwrap();
        assert!(updated.contains("Hello 2"), "{language}: {updated}");
        assert!(!updated.contains("Hello, world!"), "{language}: {updated}");
    }
}

fn rust_conversation_with_latest_user(prompt: &str) -> Vec<ChatMessage> {
    let source = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    vec![
        ChatMessage::user("Give me a hello world program in Rust"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "write-1".to_owned(),
            "write".to_owned(),
            serde_json::json!({"filePath": "main.rs", "content": source}).to_string(),
        )]),
        ChatMessage::tool_result("write-1", "write", "Wrote main.rs"),
        ChatMessage::assistant("Wrote `main.rs`."),
        ChatMessage::user(prompt),
    ]
}

fn rewrite_current_rust_source(prompt: &str, source: &str) -> String {
    let mut messages = rust_conversation_with_latest_user(prompt);
    let read = one_call(&messages, &["read", "write"]);
    assert_eq!(read.tool, "read", "{prompt}");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read-rewrite".to_owned(),
        "read".to_owned(),
        read.arguments,
    )]));
    messages.push(ChatMessage::tool_result("read-rewrite", "read", source));

    let write = one_call(&messages, &["read", "write"]);
    assert_eq!(write.tool, "write", "{prompt}");
    args(&write)["content"].as_str().unwrap().to_owned()
}

#[test]
fn contextual_code_change_reads_then_writes_the_active_file() {
    let mut messages =
        rust_conversation_with_latest_user("Change the output message to 'Hello 2'.");

    let read = one_call(&messages, &["read", "write"]);
    assert_eq!(read.tool, "read");
    assert_eq!(args(&read)["filePath"], "main.rs");

    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read-2".to_owned(),
        "read".to_owned(),
        read.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "read-2",
        "read",
        "<path>/workspace/main.rs</path>\n<type>file</type>\n<content>\n1: fn main() {\n2:     println!(\"Hello, world!\");\n3: }\n\n(End of file - total 3 lines)\n</content>",
    ));

    let write = one_call(&messages, &["read", "write"]);
    assert_eq!(write.tool, "write");
    let value = args(&write);
    assert_eq!(value["filePath"], "main.rs");
    let source = value["content"].as_str().unwrap();
    assert!(source.contains("println!(\"Hello 2\");"), "{source}");
    assert!(!source.contains("Hello, world!"), "{source}");
    assert!(!source.contains("<content>"), "{source}");

    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "write-2".to_owned(),
        "write".to_owned(),
        write.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "write-2",
        "write",
        "Wrote file successfully.",
    ));
    let final_answer = match plan_chat_step(&messages, &["read", "write"]) {
        Some(AgenticPlan::Final(answer)) => answer,
        other => panic!("expected final mutation trace, got {other:?}"),
    };
    // Values are quoted the way Links Notation quotes: only when the value needs
    // it, with a delimiter the value itself does not contain.
    assert!(final_answer.contains("normal_markov_program"));
    assert!(final_answer.contains("target main.rs"));
    assert!(final_answer.contains("rewrite_rule \"0\""));
    assert!(final_answer.contains("pattern 'Hello, world!'"));
    assert!(final_answer.contains("replacement 'Hello 2'"));
    assert!(final_answer.contains("halt 'TerminalRule(0)'"));
    assert!(final_answer.contains("steps 1"));
}

#[test]
fn contextual_change_is_language_independent_across_ten_phrasings_each() {
    let groups = [
        [
            "Change the output message to 'English 1'.",
            "Update the printed text to 'English 2'.",
            "Make it display 'English 3'.",
            "Have the program say 'English 4'.",
            "Switch the console message to 'English 5'.",
            "Use 'English 6' as the output.",
            "Modify what it prints to 'English 7'.",
            "Set the displayed message to 'English 8'.",
            "Rewrite the output as 'English 9'.",
            "Print 'English 10' instead.",
        ],
        [
            "Измени вывод на 'Русский 1'.",
            "Обнови печатаемый текст на 'Русский 2'.",
            "Пусть программа выводит 'Русский 3'.",
            "Замени сообщение на 'Русский 4'.",
            "Сделай выводом 'Русский 5'.",
            "Используй для вывода 'Русский 6'.",
            "Поменяй печать на 'Русский 7'.",
            "Установи сообщение 'Русский 8'.",
            "Перепиши вывод как 'Русский 9'.",
            "Вместо прежнего напечатай 'Русский 10'.",
        ],
        [
            "आउटपुट संदेश को 'हिन्दी 1' में बदलें।",
            "छपे हुए पाठ को 'हिन्दी 2' करें।",
            "प्रोग्राम से 'हिन्दी 3' दिखाएँ।",
            "संदेश को 'हिन्दी 4' से बदलें।",
            "आउटपुट में 'हिन्दी 5' रखें।",
            "आउटपुट के लिए 'हिन्दी 6' उपयोग करें।",
            "प्रिंट को 'हिन्दी 7' में बदलें।",
            "दिखाया संदेश 'हिन्दी 8' सेट करें।",
            "आउटपुट को 'हिन्दी 9' लिखें।",
            "इसके बजाय 'हिन्दी 10' छापें।",
        ],
        [
            "把输出消息改成'中文 1'。",
            "将打印文本更新为'中文 2'。",
            "让程序显示'中文 3'。",
            "把消息替换为'中文 4'。",
            "使用'中文 5'作为输出。",
            "输出请用'中文 6'。",
            "将打印内容改成'中文 7'。",
            "把显示消息设为'中文 8'。",
            "将输出重写为'中文 9'。",
            "改为打印'中文 10'。",
        ],
    ];

    for prompt in groups.into_iter().flatten() {
        let read = one_call(
            &rust_conversation_with_latest_user(prompt),
            &["read_file", "write_file"],
        );
        assert_eq!(read.tool, "read_file", "{prompt}");
        assert_eq!(args(&read)["path"], "main.rs", "{prompt}");
    }
}

#[test]
fn explicit_old_new_change_can_rewrite_an_arbitrary_code_fragment() {
    let mut messages = rust_conversation_with_latest_user(
        "In the current code replace 'println!(\"Hello, world!\");' with 'eprintln!(\"done\");'.",
    );
    let read = one_call(&messages, &["read", "write"]);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read-3".to_owned(),
        "read".to_owned(),
        read.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "read-3",
        "read",
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    ));

    let write = one_call(&messages, &["read", "write"]);
    let source = args(&write)["content"].as_str().unwrap().to_owned();
    assert!(source.contains("eprintln!(\"done\");"), "{source}");
    assert!(!source.contains("println!(\"Hello, world!\");"), "{source}");
}

#[test]
fn empty_pattern_creates_content() {
    let source = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    let inserted = rewrite_current_rust_source(
        "In the current code replace '' with '// generated file\n'.",
        source,
    );
    assert_eq!(inserted, format!("// generated file\n{source}"));
}

#[test]
fn nonempty_pattern_can_be_deleted() {
    let source = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    let deleted = rewrite_current_rust_source(
        r#"In the current code replace '    println!("Hello, world!");
' with ''."#,
        source,
    );
    assert_eq!(deleted, "fn main() {\n}\n");
}

#[test]
fn ordered_substitutions_apply_as_one_general_rewrite_program() {
    let source = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    let rewritten = rewrite_current_rust_source(
        "Apply these ordered substitutions to the current code: 'Hello' to 'Hi', then 'world' to 'team'.",
        source,
    );

    assert_eq!(rewritten, "fn main() {\n    println!(\"Hi, team!\");\n}\n");
}

/// The link-cli-dialect query is the meta-language representation a request is
/// lowered to. Accepting it directly proves the layer is real rather than a
/// rendering, and it is the only way to state link-cli's `()` shorthands.
#[test]
fn substitution_query_is_accepted_as_the_request_itself() {
    let source = "fn main() {\n    println!(\"Hello, world!\");\n}\n";

    let updated =
        rewrite_current_rust_source(r#"(("Hello, world!")) ((terminal: "Hello 2"))"#, source);
    assert_eq!(updated, "fn main() {\n    println!(\"Hello 2\");\n}\n");

    // Deletion: a non-empty sequence substituted to no sequence.
    let deleted = rewrite_current_rust_source(
        "((terminal: \"    println!(\\\"Hello, world!\\\");\\n\")) ()",
        source,
    );
    assert_eq!(deleted, "fn main() {\n}\n");

    // Creation: the empty sequence substituted to a non-empty one.
    let created = rewrite_current_rust_source(r#"() ((terminal: "// generated\n"))"#, source);
    assert_eq!(
        created,
        "// generated\nfn main() {\n    println!(\"Hello, world!\");\n}\n"
    );
}

/// Every harness reaches the same lowering through its own tool names, because
/// routing is by capability rather than by a hard-coded vocabulary.
#[test]
fn substitution_query_lowers_identically_on_every_harness_vocabulary() {
    let source = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    for tools in [
        ["read", "write"],
        ["read_file", "write_file"],
        ["view_file", "create_file"],
        ["file_read", "Write"],
    ] {
        let query = r#"(("Hello, world!")) ((terminal: "Hello 2"))"#;
        let mut messages = rust_conversation_with_latest_user(query);

        let read = one_call(&messages, &tools);
        assert_eq!(read.tool, tools[0], "{tools:?}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "read-harness".to_owned(),
            tools[0].to_owned(),
            read.arguments,
        )]));
        messages.push(ChatMessage::tool_result("read-harness", tools[0], source));

        let write = one_call(&messages, &tools);
        assert_eq!(write.tool, tools[1], "{tools:?}");
        assert_eq!(
            args(&write)["content"].as_str().unwrap(),
            "fn main() {\n    println!(\"Hello 2\");\n}\n",
            "{tools:?}",
        );
    }
}

/// The final trace publishes the meta-language query and each rule's CRUD
/// effect, so the lowering is auditable end to end.
#[test]
fn mutation_trace_publishes_the_substitution_query_and_effects() {
    let mut messages =
        rust_conversation_with_latest_user("Change the output message to 'Hello 2'.");
    let source = "fn main() {\n    println!(\"Hello, world!\");\n}\n";

    let read = one_call(&messages, &["read", "write"]);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read-trace".to_owned(),
        "read".to_owned(),
        read.arguments,
    )]));
    messages.push(ChatMessage::tool_result("read-trace", "read", source));
    let write = one_call(&messages, &["read", "write"]);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "write-trace".to_owned(),
        "write".to_owned(),
        write.arguments,
    )]));
    messages.push(ChatMessage::tool_result("write-trace", "write", "Wrote."));

    let final_answer = match plan_chat_step(&messages, &["read", "write"]) {
        Some(AgenticPlan::Final(answer)) => answer,
        other => panic!("expected final mutation trace, got {other:?}"),
    };
    // Links Notation picks a delimiter the value does not contain, so a query
    // full of double quotes is carried in single quotes rather than backslashed.
    assert!(
        final_answer
            .contains(r#"substitution_query '(("Hello, world!")) ((terminal: "Hello 2"))'"#),
        "{final_answer}"
    );
    assert!(final_answer.contains("effect update"), "{final_answer}");
}

#[test]
fn inspected_links_record_is_derived_written_and_read_back_before_completion() {
    let task = "Inspect failures.lino. Author recurrence-proposal.lino as valid Links Notation. \
        Include top-level recurrence_schema and nested boundary_condition, transition, \
        termination_measure, validation, and provenance fields.";
    let source = r#"recurrence_failure_taxonomy
  validation_gap
    symptom "browser cannot execute Python"
    shared_capability "prove structural descent and evaluate finite source testers"
  required_record
    boundary_condition "predicate, threshold, boundary expression"
    recursive_transition "self-call offsets and combining operation"
    termination "every recursive argument decreases toward the boundary"
    provenance "source URL, license, content hash, fetched time"
"#;
    let mut messages = vec![ChatMessage::user(task)];

    let read_source = one_call(&messages, &["read", "write"]);
    assert_eq!(read_source.tool, "read");
    assert_eq!(args(&read_source)["filePath"], "failures.lino");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read-source",
        "read",
        read_source.arguments,
    )]));
    let decorated_source = format!(
        "<file>\n{}\n\n(End of file - total {} lines)\n</file>",
        source
            .lines()
            .enumerate()
            .map(|(index, line)| format!("{}| {line}", index + 1))
            .collect::<Vec<_>>()
            .join("\n"),
        source.lines().count()
    );
    messages.push(ChatMessage::tool_result(
        "read-source",
        "read",
        decorated_source,
    ));

    let write = one_call(&messages, &["read", "write"]);
    assert_eq!(write.tool, "write");
    let written = args(&write);
    assert_eq!(written["filePath"], "recurrence-proposal.lino");
    let content = written["content"].as_str().unwrap().to_owned();
    assert!(content.starts_with("recurrence_schema\n"), "{content}");
    assert!(content.contains("source_observation"), "{content}");
    assert!(content.contains("derived_formalization"), "{content}");
    assert!(
        content.contains("boundary_condition \"predicate, threshold, boundary expression\""),
        "{content}"
    );
    assert!(
        content.contains("transition \"self-call offsets and combining operation\""),
        "{content}"
    );
    assert!(
        content.contains(
            "termination_measure \"every recursive argument decreases toward the boundary\""
        ),
        "{content}"
    );
    assert!(content.contains("validation \"browser cannot execute Python; prove structural descent and evaluate finite source testers\""), "{content}");
    assert!(
        content.contains("provenance \"source URL, license, content hash, fetched time\""),
        "{content}"
    );

    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "write-derived",
        "write",
        write.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "write-derived",
        "write",
        "Wrote recurrence-proposal.lino",
    ));

    let read_back = one_call(&messages, &["read", "write"]);
    assert_eq!(read_back.tool, "read");
    assert_eq!(args(&read_back)["filePath"], "recurrence-proposal.lino");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read-derived",
        "read",
        read_back.arguments,
    )]));
    messages.push(ChatMessage::tool_result("read-derived", "read", content));

    let final_answer = match plan_chat_step(&messages, &["read", "write"]) {
        Some(AgenticPlan::Final(answer)) => answer,
        other => panic!("expected verified final answer, got {other:?}"),
    };
    assert!(final_answer.contains("verified"), "{final_answer}");
}

#[test]
fn incidental_dotted_error_class_is_not_an_inspected_source_file() {
    // A live Agent CLI review named an upstream error class inline and asked
    // for the output to be read back. File shape alone made the structured
    // document route bind `org.example.TransientFailure` as its input and try
    // to open it. An inspected source must be named beside the inspection
    // action; a dotted technical identifier in a different sentence is data,
    // not a workspace path.
    let task = "Inspect incident-source.lino. \
        A library reported exactly org.example.TransientFailure: with no diagnostic. \
        Author incident-review.lino as valid Links Notation with root incident_review and \
        fields classification, positive_signature, and negative_boundary. \
        Write only incident-review.lino, then read it back.";

    let call = one_call(&[ChatMessage::user(task)], &["read", "write"]);
    assert_eq!(call.tool, "read");
    assert_eq!(args(&call)["filePath"], "incident-source.lino");
}

#[test]
fn inspected_record_collections_preserve_cardinality_scope_and_repeated_fields() {
    let task = "Inspect recurrence-source.lino. Author recurrence-cache-index.lino as valid Links Notation. \
        Derive one recurrence_cache_index entry for every recurrence; preserve label, identifier, \
        abstract_implementation, source_url, license, termination_measure, predecessor_offset, \
        and nonexistent_source fields when present.";
    let source = r#"recurrence_source_cache
  recurrence "Z1"
    label "alpha"
    identifier "alpha"
    termination_measure "n"
    predecessor_offset "1"
    predecessor_offset "2"
    abstract_implementation "I1"
    source_url "https://example.test/I1"
    license "CC0-1.0"
  recurrence "Z2"
    label "beta"
    identifier "beta"
    termination_measure "m"
    predecessor_offset "1"
    abstract_implementation "I2"
    source_url "https://example.test/I2"
    license "CC0-1.0"
"#;
    let mut messages = vec![ChatMessage::user(task)];

    let read_source = one_call(&messages, &["read", "write"]);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "read-collection",
        "read",
        read_source.arguments,
    )]));
    messages.push(ChatMessage::tool_result("read-collection", "read", source));

    let write = one_call(&messages, &["read", "write"]);
    assert_eq!(write.tool, "write");
    let content = args(&write)["content"].as_str().unwrap().to_owned();
    assert_eq!(
        content.matches("  derived_formalization ").count(),
        2,
        "{content}"
    );
    assert!(
        content.contains("derived_formalization \"Z1\""),
        "{content}"
    );
    assert!(
        content.contains("derived_formalization \"Z2\""),
        "{content}"
    );
    assert!(content.contains("label \"alpha\""), "{content}");
    assert!(content.contains("label \"beta\""), "{content}");
    assert!(
        content.contains("source_url \"https://example.test/I1\""),
        "{content}"
    );
    assert!(
        content.contains("source_url \"https://example.test/I2\""),
        "{content}"
    );
    assert_eq!(
        content.matches("predecessor_offset").count(),
        3,
        "{content}"
    );
    assert!(!content.contains("nonexistent_source"), "{content}");
    assert!(!content.contains("not established"), "{content}");
}
