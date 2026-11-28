use super::*;

pub(super) fn g69_an_addition_that_quotes_no_text_earns_a_question() {
    for prompt in [
        "add hello to config.txt",
        "append hello to config.txt",
        "add hello to the end of config.txt",
    ] {
        let run = drive(prompt, &[("config.txt", "a = 1\n")]);
        assert!(run.tools.is_empty(), "{prompt}: {:?}", run.tools);
        assert_eq!(run.files["config.txt"], "a = 1\n", "{prompt}");
        assert_eq!(
            run.answer.as_deref(),
            Some(
                "What exact text should go into `config.txt`, and where? The request names the addition without quoting it, so I cannot tell the text to add from words that describe it, and nothing was changed. Quote the text and say where it goes, for example: Append the line «…» to config.txt."
            ),
            "{prompt}"
        );
    }
    let named = drive(
        "Create a test for add in t.mjs and run it.",
        &[("m.mjs", "export const add = (a, b) => a + b;\n")],
    );
    assert!(
        !named
            .tools
            .iter()
            .any(|tool| tool == "write" || tool == "edit"),
        "{:?}",
        named.tools
    );
    assert!(!named.files.contains_key("t.mjs"));
    let quoted = drive(
        "Append the line 'b = 2' to config.txt.",
        &[("config.txt", "a = 1\n")],
    );
    assert_eq!(quoted.files["config.txt"], "a = 1\nb = 2\n");
}

pub(super) fn g70_no_seed_surface_holds_a_seeded_contraction_written_apart() {
    let pairs = seeded_contractions();
    assert!(
        pairs
            .iter()
            .any(|(contracted, expansion)| contracted == "can t" && expansion == "cannot")
    );
    let mut contracted = Vec::new();
    for (path, text) in formal_ai::seed::seed_files() {
        if path == "data/seed/languages.lino" {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            let Some(value) = ["text ", "phrase ", "cue ", "followup_request "]
                .iter()
                .find_map(|key| trimmed.strip_prefix(key))
                .and_then(|value| value.strip_prefix('"')?.strip_suffix('"'))
            else {
                continue;
            };
            let padded = format!(" {value} ");
            if pairs
                .iter()
                .any(|(pair, _)| padded.contains(&format!(" {pair} ")))
            {
                contracted.push(format!("{path}:{}", index + 1));
            }
        }
    }
    assert!(contracted.is_empty(), "{contracted:?}");
}

pub(super) fn g70_a_provider_denial_written_with_a_contraction_is_a_failure() {
    let tools = ["bash", "read", "webfetch", "websearch"];
    let mut messages = vec![ChatMessage::user("Fetch https://example.test/x")];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &tools) else {
        panic!("the fetch is planned");
    };
    assert_eq!(calls[0].tool, "webfetch");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "c0",
        "webfetch",
        calls[0].arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(
        "c0",
        "webfetch",
        "You can't perform that action at this time.",
    ));
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("the denial is answered");
    };
    assert!(
        answer.starts_with("The command failed: You can't perform that action at this time."),
        "{answer}"
    );
}

pub(super) fn g78_a_replace_whose_new_text_holds_the_old_is_a_verified_edit() {
    for (prompt, name, source, expected, answer) in [
        (
            "In s.txt replace 'key 3' with 'key 33'",
            "s.txt",
            "a\nkey 3\nb\n",
            "a\nkey 33\nb\n",
            "Replaced `key 3` with `key 33` in `s.txt` and observed the result.",
        ),
        (
            "Replace `run();` with `guard(); run();` in s.mjs.",
            "s.mjs",
            "if (a) {\n  run();\n}\n",
            "if (a) {\n  guard(); run();\n}\n",
            "Replaced `run();` with `guard(); run();` in `s.mjs` and observed the result.",
        ),
    ] {
        let run = drive(prompt, &[(name, source)]);
        assert_eq!(run.tools, ["read", "edit", "bash"], "{prompt}");
        assert_eq!(run.files[name], expected, "{prompt}");
        assert_eq!(run.answer.as_deref(), Some(answer), "{prompt}");
    }
}
