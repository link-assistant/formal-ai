use formal_ai::{
    ChatCompletionRequest, ChatMessage, ConversationTurn, FormalAiEngine, ResponsesRequest,
    UniversalSolver, create_chat_completion, create_response, handle_api_request,
    knowledge_links_notation,
};
use lino_objects_codec::format::parse_indented;

mod seed_and_memory;

const PYTHON_SCRIPT_ANSWER: &str = r#"Here is a minimal Python script:

```python
print("Hello, world!")
```

Execution status: compiled and ran in issue-8 local verification harness (isolated sandbox).
Check command: `python3 -m py_compile main.py`
Run command: `python3 main.py`
Output:
```text
Hello, world!
```
1 iteration completed under the 1 minute execution budget; no timeout reduction was needed."#;

const RUST_SCRIPT_ANSWER: &str = r#"Here is a minimal Rust script:

```rust
fn main() {
    println!("Hello, world!");
}
```

Execution status: compiled and ran in issue-8 local verification harness (isolated sandbox).
Check command: `rustc main.rs -o main`
Run command: `./main`
Output:
```text
Hello, world!
```
1 iteration completed under the 1 minute execution budget; no timeout reduction was needed."#;

const JAVASCRIPT_SCRIPT_ANSWER: &str = r#"Here is a minimal JavaScript script:

```javascript
console.log("Hello, world!");
```

Execution status: compiled and ran in issue-8 local verification harness (isolated sandbox).
Check command: `node --check main.js`
Run command: `node main.js`
Output:
```text
Hello, world!
```
1 iteration completed under the 1 minute execution budget; no timeout reduction was needed."#;

const OWLBEAR_PROJECT_PLAN: &str = r#"Implementation plan pending approval for a extension targeting Owlbear.

Formalized meaning:
```lino
software_project_request
  action "write"
  artifact "extension"
  artifact_surface "extension"
  target "Owlbear"
  domain "tabletop_game_unit_tracker"
  delivery_mode code_generation
  implementation_language "typescript"
  approval_state proposed
  approval_required true
  approval_gate "generated_code"
  approval_gate "implementation_plan"
  approval_gate "task_formalization"
  requirement "I need extensions that can track hp for different units"
  requirement_category "state_tracking"
  requirement "That can track Protection and Resistance stacks on unit an will reduce damage count on those stats"
  requirement_category "state_tracking"
  requirement "Also this extension should track cooldown of some abilities"
  requirement_category "state_tracking"
  subtask "R1 [state_tracking] Model state fields and pure transitions for I need extensions that can track hp for different units"
  subtask "R2 [state_tracking] Model state fields and pure transitions for That can track Protection and Resistance stacks on unit an will reduce damage count on those stats"
  subtask "R3 [state_tracking] Model state fields and pure transitions for Also this extension should track cooldown of some abilities"
  state_model "unit_state"
  command "apply_damage"
  command "set_stacks"
  command "tick_cooldowns"
  validation "damage_mitigation_floor_at_zero"
  validation "cooldowns_decrement_without_negative_rounds"
```

Reasoning steps:
1. Classify the impulse as a request to write a extension instead of a fact lookup.
2. Bind the target environment to Owlbear and keep the first response reviewable.
3. Extract 3 requirement(s) into the meaning record before planning.
4. Decompose the requirement graph into 3 implementation subtask(s) with category labels.
5. Select delivery mode code_generation and approval gates: generated_code, implementation_plan, task_formalization.
6. Map HP, Protection, Resistance, damage, and cooldown phrases to a unit-state domain model.
7. Ask for approval before producing code, scripts, manual instructions, or execution steps.

Requirement model:
1. [state_tracking] I need extensions that can track hp for different units
2. [state_tracking] That can track Protection and Resistance stacks on unit an will reduce damage count on those stats
3. [state_tracking] Also this extension should track cooldown of some abilities

Subtasks:
1. R1 -> Model state fields and pure transitions for I need extensions that can track hp for different units
2. R2 -> Model state fields and pure transitions for That can track Protection and Resistance stacks on unit an will reduce damage count on those stats
3. R3 -> Model state fields and pure transitions for Also this extension should track cooldown of some abilities

Approval gates:
- generated_code
- implementation_plan
- task_formalization

Proposed plan:
1. Review the formalized task, requirement graph, approval gates, and delivery mode with the user.
2. Confirm the Owlbear storage and selected-token API boundaries.
3. Define `UnitState` with HP, max HP, Protection, Resistance, and cooldowns.
4. Write pure transition functions for damage mitigation, stack edits, and round ticks.
5. Add tests for zero damage, overkill damage, stack changes, and cooldown expiry.
6. Wire the tested core into the extension panel and host persistence.

Reply `approve plan` to generate the starter implementation, or describe what to change."#;

pub const BROWSER_EXTENSION_PROJECT_PLAN: &str = r#"Implementation plan pending approval for a browser extension targeting the requested environment.

Formalized meaning:
```lino
software_project_request
  action "build"
  artifact "browser extension"
  artifact_surface "browser extension"
  target "the requested environment"
  domain "software_project"
  delivery_mode code_generation
  implementation_language "typescript"
  approval_state proposed
  approval_required true
  approval_gate "generated_code"
  approval_gate "implementation_plan"
  approval_gate "task_formalization"
  requirement "Build a browser extension that tracks reading progress and exports CSV"
  requirement_category "state_tracking"
  subtask "R1 [state_tracking] Model state fields and pure transitions for Build a browser extension that tracks reading progress and exports CSV"
  state_model "project_records"
  command "create_record"
  command "update_record"
  command "export_state"
  validation "pure_state_transitions_before_host_api"
```

Reasoning steps:
1. Classify the impulse as a request to build a browser extension instead of a fact lookup.
2. Bind the target environment to the requested environment and keep the first response reviewable.
3. Extract 1 requirement(s) into the meaning record before planning.
4. Decompose the requirement graph into 1 implementation subtask(s) with category labels.
5. Select delivery mode code_generation and approval gates: generated_code, implementation_plan, task_formalization.
6. Ask for approval before producing code, scripts, manual instructions, or execution steps.

Requirement model:
1. [state_tracking] Build a browser extension that tracks reading progress and exports CSV

Subtasks:
1. R1 -> Model state fields and pure transitions for Build a browser extension that tracks reading progress and exports CSV

Approval gates:
- generated_code
- implementation_plan
- task_formalization

Proposed plan:
1. Review the formalized task, requirement graph, approval gates, and delivery mode with the user.
2. Confirm the host API and data boundaries for the requested environment.
3. Define the smallest serializable state records for the requirements.
4. Implement state_tracking: Model state fields and pure transitions for Build a browser extension that tracks reading progress and exports CSV.
5. Generate a typescript starter core plus language-appropriate repository initialization and checks.
6. Keep shell, Docker, or WebVM commands behind the configured approval gates.

Reply `approve plan` to generate the starter implementation, or describe what to change."#;

#[test]
fn greeting_prompt_returns_symbolic_greeting() {
    let response = FormalAiEngine.answer("Hi");

    assert_eq!(response.intent, "greeting");
    assert_eq!(response.answer, "Hi, how may I help you?");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "response:greeting")
    );
}

#[test]
fn shabbat_shalom_greeting_is_recognized_as_greeting() {
    for prompt in ["шабат шалом!", "шабат шалом", "шалом"] {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "greeting",
            "prompt {:?} should be recognized as a greeting, got intent {:?}",
            prompt, response.intent
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:greeting"),
            "prompt {prompt:?} response should cite response:greeting",
        );
    }
}

// Issue #152: "how are you?" small talk used to fall through to the unknown
// fallback. The follow-up review made this a supported-language invariant.
// Issue #676: it now routes to a dedicated `wellbeing` intent so the reply is
// distinct from a bare "Hello" greeting, still across every supported language.
#[test]
fn how_are_you_prompt_is_recognized_as_wellbeing_in_supported_languages() {
    let cases = [
        ("How are you?", "language:en"),
        ("Как твои дела?", "language:ru"),
        ("आप कैसे हैं?", "language:hi"),
        ("你好吗?", "language:zh"),
    ];

    for (prompt, language_link) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "wellbeing",
            "small-talk prompt {prompt:?} should be recognized as wellbeing, got intent {:?}",
            response.intent
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:wellbeing"),
            "response should cite response:wellbeing for {prompt:?}, got {:?}",
            response.evidence_links
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == language_link),
            "response should keep {language_link} for {prompt:?}, got {:?}",
            response.evidence_links
        );
    }
}

// Issue #67: "пока" and similar farewell words were returned as unknown intent.
#[test]
fn farewell_prompts_are_recognized_as_farewell() {
    let cases = [
        ("пока", "ru"),
        ("до свидания", "ru"),
        ("bye", "en"),
        ("goodbye", "en"),
    ];

    for (prompt, expected_language) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "farewell",
            "prompt {:?} should be recognized as farewell, got intent {:?}",
            prompt, response.intent
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:farewell"),
            "prompt {prompt:?} response should cite response:farewell",
        );
        if expected_language == "ru" {
            assert!(
                response.answer.contains("свидания") || response.answer.contains("Пока"),
                "Russian farewell {prompt:?} should get a Russian answer, got: {}",
                response.answer
            );
        }
    }
}

#[test]
fn identity_questions_return_standard_self_description() {
    let cases = [
        "Who are you?",
        "what are you",
        "Tell me about yourself",
        "What is formal-ai?",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(response.intent, "identity");
        assert!(response.answer.contains("formal-ai"));
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:identity")
        );
    }
}

#[test]
fn how_you_work_prompts_return_meta_explanation() {
    let cases = [
        ("покажи как ты работаешь?", "ru"),
        ("как ты работаешь?", "ru"),
        ("how do you work?", "en"),
        ("show me how you work", "en"),
    ];

    for (prompt, expected_language) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(
            response.intent, "meta_explanation",
            "prompt '{prompt}' should resolve to meta_explanation, got '{}'",
            response.intent
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "response:meta_explanation"),
            "prompt '{prompt}' should include evidence link response:meta_explanation"
        );
        // Russian prompts must respond in Russian
        if expected_language == "ru" {
            assert!(
                response.answer.contains("работаешь")
                    || response.answer.contains("правил")
                    || response.answer.contains("Notation"),
                "Russian prompt '{prompt}' should get a Russian answer, got: {}",
                response.answer
            );
        }
    }
}

#[test]
fn rust_hello_world_prompt_returns_code_block() {
    let response = FormalAiEngine.answer("Write me hello world program in Rust");

    assert_eq!(response.intent, "write_program");
    assert!(
        response
            .links_notation
            .contains("program_parameter:language rust")
    );
    assert!(
        response
            .links_notation
            .contains("program_parameter:task hello_world")
    );
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("fn main()"));
    assert!(response.answer.contains("println!(\"Hello, world!\");"));
    assert!(
        response
            .answer
            .contains("Execution status: compiled and ran")
    );
    assert!(response.answer.contains("Output:"));
    assert!(response.answer.contains("```text\nHello, world!\n```"));
}

// Issue #31: Queries about KISS in a programming context should return the
// software design principle, not the rock band KISS.
#[test]
fn kiss_in_programming_context_returns_design_principle_not_band() {
    let cases = [
        // Exact issue report (Russian, misspelled programming word, "в рамках" delimiter)
        "что такое Kiss в рамках програмирования",
        // English equivalents
        "what is KISS in programming",
        "what is kiss in software development",
    ];

    for prompt in cases {
        let response = FormalAiEngine.answer(prompt);

        // Must resolve as a concept lookup (offline, deterministic — no Wikipedia
        // network call needed because the KISS principle is in the concept corpus).
        assert!(
            response.intent == "concept_lookup_in_context" || response.intent == "concept_lookup",
            "[{prompt}] unexpected intent: {}",
            response.intent
        );
        // Answer must mention the design principle, not the rock band.
        assert!(
            response.answer.contains("принцип")
                || response.answer.contains("KISS")
                || response.answer.contains("simple"),
            "[{prompt}] answer does not mention the design principle: {}",
            response.answer
        );
        assert!(
            !response.answer.contains("рок-группа") && !response.answer.contains("rock band"),
            "[{prompt}] answer incorrectly describes the rock band: {}",
            response.answer
        );
    }
}

#[test]
fn hello_world_prompt_supports_multiple_programming_languages() {
    let cases = [
        ("Write hello world in Python", "python", "```python"),
        (
            "Create a hello world example in JavaScript",
            "javascript",
            "```javascript",
        ),
        ("hello world in Go", "go", "```go"),
    ];

    for (prompt, language, code_fence) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(response.intent, "write_program");
        assert!(
            response
                .links_notation
                .contains(&format!("program_parameter:language {language}")),
            "prompt: {prompt:?} — missing language parameter in trace: {}",
            response.links_notation
        );
        assert!(response.answer.contains(code_fence));
        assert!(response.answer.contains("Hello, world!"));
    }
}

#[test]
fn write_script_prompt_returns_code_block() {
    // Regression test for issue #35: "Напиши скрипт на питоне" was returning
    // intent: unknown instead of routing to a code answer.
    let cases = [
        (
            "Напиши скрипт на питоне",
            "write_script_python",
            "```python",
            PYTHON_SCRIPT_ANSWER,
        ),
        (
            "Write a script in Python",
            "write_script_python",
            "```python",
            PYTHON_SCRIPT_ANSWER,
        ),
        (
            "Write a script in Rust",
            "write_script_rust",
            "```rust",
            RUST_SCRIPT_ANSWER,
        ),
        (
            "Write me some code in JavaScript",
            "write_script_javascript",
            "```javascript",
            JAVASCRIPT_SCRIPT_ANSWER,
        ),
        (
            "написать скрипт на javascript",
            "write_script_javascript",
            "```javascript",
            JAVASCRIPT_SCRIPT_ANSWER,
        ),
    ];

    for (prompt, intent, code_fence, expected_answer) in cases {
        let response = FormalAiEngine.answer(prompt);

        assert_eq!(response.answer, expected_answer, "prompt: {prompt:?}");
        assert_eq!(
            response.intent, intent,
            "prompt: {prompt:?} — answer was: {}",
            response.answer
        );
        assert!(
            response.answer.contains(code_fence),
            "prompt: {prompt:?} — expected {code_fence} in answer: {}",
            response.answer
        );
        assert_ne!(
            response.intent, "unknown",
            "prompt: {prompt:?} — got unknown intent"
        );
    }
}

#[test]
fn software_project_request_returns_reviewable_plan() {
    // Regression test for issue #80: a request to write an Owlbear/D&D
    // extension was returning intent: unknown. This must be handled as a
    // generalized software project request, not a memoized prompt.
    let prompt = concat!(
        "Hi, can you write for me extension for owlbear? I am currently leading some dnd games ",
        "and i want to try wargame. So, i need extensions that can track hp for different units, ",
        "that can track Protection and Resistance stacks on unit an will reduce damage count on ",
        "those stats. Also this extension should track cooldown of some abilities"
    );

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(response.answer, OWLBEAR_PROJECT_PLAN);
    assert_eq!(
        response.intent, "software_project_plan",
        "answer was: {}",
        response.answer
    );
    assert_ne!(response.intent, "unknown");
    assert!(response.answer.contains("Formalized meaning"));
    assert!(response.answer.contains("software_project_request"));
    assert!(response.answer.contains("Reasoning steps"));
    assert!(response.answer.contains("Proposed plan"));
    assert!(response.answer.contains("Owlbear"));
    assert!(response.answer.contains("HP"));
    assert!(response.answer.contains("Protection"));
    assert!(response.answer.contains("Resistance"));
    assert!(response.answer.contains("cooldown"));
    assert!(response.answer.contains("approve plan"));
    assert!(!response.answer.contains("mitigateDamage"));
}

#[test]
fn software_project_variations_do_not_return_unknown() {
    let prompts = [
        "Build a browser extension that tracks reading progress and exports CSV",
        "Create a Discord bot for scheduling game sessions with reminders",
        "Implement a small web app for tracking invoices and overdue payments",
        "Make a plugin for a tabletop map that tracks unit status effects",
        "Develop a command line tool for renaming photos by date",
    ];

    for prompt in prompts {
        let response = FormalAiEngine.answer(prompt);
        if prompt == prompts[0] {
            assert_eq!(response.answer, BROWSER_EXTENSION_PROJECT_PLAN);
        }
        assert_eq!(
            response.intent, "software_project_plan",
            "prompt: {prompt:?} answer: {}",
            response.answer
        );
        assert_ne!(response.intent, "unknown");
        assert!(response.answer.contains("Formalized meaning"));
        assert!(response.answer.contains("Proposed plan"));
        assert!(response.answer.contains("approve plan"));
    }
}

#[test]
fn software_project_approval_returns_implementation_starter() {
    let solver = UniversalSolver::default();
    let prompt = concat!(
        "Write an extension for Owlbear that tracks HP, Protection, Resistance, ",
        "damage mitigation, and cooldowns for tabletop units"
    );

    let plan = solver.solve(prompt);
    let history = [
        ConversationTurn::user(prompt),
        ConversationTurn::assistant(plan.answer),
    ];
    let implementation = solver.solve_with_history("approve plan", &history);

    assert_eq!(
        implementation.intent, "software_project_implementation",
        "answer was: {}",
        implementation.answer
    );
    assert!(implementation.answer.contains("approval_state approved"));
    assert!(implementation.answer.contains("```typescript"));
    assert!(implementation.answer.contains("mitigateDamage"));
    assert!(implementation.answer.contains("tickCooldowns"));
}

#[test]
fn chat_completion_has_openai_compatible_shape() {
    let request = ChatCompletionRequest {
        model: Some(String::from("formal-ai")),
        messages: vec![ChatMessage::user("Hello")],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };

    let completion = create_chat_completion(&request);

    assert_eq!(completion.object, "chat.completion");
    assert_eq!(completion.model, "formal-ai");
    assert_eq!(completion.choices[0].finish_reason, "stop");
    assert_eq!(
        completion.choices[0].message.content.plain_text(),
        "Hi, how may I help you?"
    );
    assert!(completion.usage.total_tokens >= completion.usage.prompt_tokens);
}

#[test]
fn responses_api_shape_contains_output_text() {
    let request = ResponsesRequest {
        model: Some(String::from("formal-ai")),
        input: serde_json::Value::String(String::from("Write hello world in Rust")),
        instructions: None,
        temperature: None,
        stream: false,
        ..ResponsesRequest::default()
    };

    let response = create_response(&request);

    assert_eq!(response.object, "response");
    assert_eq!(response.status, "completed");
    let messages = response.output_messages();
    assert_eq!(messages[0].role, "assistant");
    assert_eq!(messages[0].content[0].kind, "output_text");
    assert!(messages[0].content[0].text.contains("```rust"));
}

#[test]
fn knowledge_export_is_valid_links_notation() {
    let notation = knowledge_links_notation();
    let records = notation.split("\n\n").collect::<Vec<_>>();
    let (id, root) = parse_indented(records[0]).expect("root record should parse");

    assert_eq!(id, "formal_ai_knowledge");
    assert_eq!(root.get("model").map(String::as_str), Some("formal-ai"));
    assert!(records.iter().any(|record| {
        let Ok((_id, parsed)) = parse_indented(record) else {
            return false;
        };

        parsed.get("intent").map(String::as_str) == Some("write_program")
            && parsed.get("parameters").map(String::as_str) == Some("language, task")
    }));
    assert!(!notation.contains("(str "));
}

#[test]
fn server_handler_supports_chat_completions_route() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();

    let response = handle_api_request("POST", "/v1/chat/completions", &body);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be JSON");
    assert_eq!(json["object"], "chat.completion");
    assert_eq!(
        json["choices"][0]["message"]["content"],
        "Hi, how may I help you?"
    );
}

#[test]
fn telegram_webhook_supports_private_messages() {
    let body = serde_json::json!({
        "update_id": 1000,
        "message": {
            "message_id": 7,
            "date": 1,
            "chat": {"id": 42, "type": "private"},
            "text": "Hi"
        }
    })
    .to_string();

    let response = handle_api_request("POST", "/telegram/webhook", &body);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be JSON");
    assert_eq!(json["method"], "sendMessage");
    assert_eq!(json["chat_id"], 42);
    assert_eq!(json["parse_mode"], "HTML");
    let text = json["text"].as_str().expect("text should be a string");
    assert!(text.starts_with("Hi, how may I help you?"));
    assert!(text.contains("/trace "));
}

#[test]
fn telegram_webhook_supports_public_chat_code_replies() {
    let body = serde_json::json!({
        "update_id": 1001,
        "message": {
            "message_id": 8,
            "date": 1,
            "chat": {"id": -100_123, "type": "supergroup", "title": "formal-ai"},
            "text": "Write me hello world program in Rust"
        }
    })
    .to_string();

    let response = handle_api_request("POST", "/telegram/webhook", &body);

    assert_eq!(response.status_code, 200);
    let json: serde_json::Value =
        serde_json::from_str(&response.body).expect("response should be JSON");
    assert_eq!(json["method"], "sendMessage");
    assert_eq!(json["chat_id"], -100_123);
    assert_eq!(json["parse_mode"], "HTML");
    let text = json["text"]
        .as_str()
        .expect("telegram reply text should be a string");
    assert!(text.contains("<pre><code class=\"language-rust\">"));
    assert!(text.contains("Execution status: compiled and ran"));
    assert!(text.contains("Hello, world!"));
}
