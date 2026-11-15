//! Issue #1559: a system prompt of one agent session must never become a
//! learned standing requirement that is replayed in another repository's
//! session. Twin of `rust/tests/web/issue-1559-standing-requirement-scope.test.mjs`.
//!
//! The recorded prompts are the solver prompt of the Scala test session
//! (`rust/tests/fixtures/issue-1559/scala-solver-prompt.txt`) and the same
//! prompt for the Rust test session of the issue's evidence: another
//! repository and another branch.

use std::ffi::OsStr;
use std::path::PathBuf;

use formal_ai::dreaming::bounds::{
    bounded_statement, contains_task_identifier, is_session_instruction, is_topic_stopword,
    is_valid_topic, rule_maximum_characters, statement_topic,
};
use formal_ai::standing_scope::{
    RequestScope, events_in_scope, recorded_scope_evidence, repository_of, request_scope,
    scope_key, scope_pairs,
};
use formal_ai::{
    ChatCompletionRequest, ChatMessage, DreamingConfig, MemoryEvent, MemoryStore, ResponsesRequest,
    RetainedAmendment, SolverConfig, SyncStore, UniversalSolver, apply_dreaming_plan,
    create_chat_completion_with_solver_and_memory, create_response_with_solver_and_memory,
    export_memory_links_notation, handle_api_request, plan_memory_dreaming,
    replay_answer_with_amendments, retained_amendments,
};
use serde_json::{Value, json};

const SCALA_REPOSITORY: &str = "konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b";
const SCALA_BRANCH: &str = "issue-1-1f3e3886bcb8";
const RUST_REPOSITORY: &str = "konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c";
const RUST_BRANCH: &str = "issue-1-09b0c76bd0e4";
const SCALA_PROMPT: &str = include_str!("../fixtures/issue-1559/scala-solver-prompt.txt");
const AMENDMENT_LINE: &str = "Learned standing requirement";
const AGENT_SYSTEM: &str = "You are Agent, the best coding agent on the planet.";
const SHARE_VARIABLE: &str = "FORMAL_AI_SHARE_STANDING_REQUIREMENTS";

fn rust_prompt() -> String {
    SCALA_PROMPT
        .replace(SCALA_REPOSITORY, RUST_REPOSITORY)
        .replace(SCALA_BRANCH, RUST_BRANCH)
}

fn tools() -> Value {
    json!([{
        "type": "function",
        "function": {
            "name": "bash",
            "description": "Run a shell command",
            "parameters": {
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"]
            }
        }
    }])
}

fn session(prompt: &str) -> Vec<ChatMessage> {
    vec![
        ChatMessage::new("system", AGENT_SYSTEM),
        ChatMessage::user(prompt),
    ]
}

/// The stored amendment of the issue's evidence: topic `you`, the whole prompt
/// as its rule.
fn legacy_amendment() -> MemoryEvent {
    MemoryEvent {
        id: String::from("amendment_legacy_you"),
        kind: Some(String::from("meta_algorithm_amendment")),
        role: Some(String::from("system")),
        content: Some(SCALA_PROMPT.to_owned()),
        inputs: Some(String::from("topic=you")),
        outputs: Some(format!("rule={SCALA_PROMPT}")),
        evidence: vec![
            String::from("chat_user_legacy"),
            String::from("recipe:data/meta/dreaming-recipe.lino"),
        ],
        ..MemoryEvent::default()
    }
}

/// A private memory file; removed when dropped.
struct Host {
    dir: PathBuf,
    path: PathBuf,
}

impl Host {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "formal-ai-issue-1559-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("memory.lino");
        Self { dir, path }
    }

    /// POST one chat completion through the HTTP handler, as the server does.
    fn chat(&self, body: &Value, agent_mode: bool) -> String {
        let mut body = body.clone();
        body["model"] = json!("formal-ai");
        let body = body.to_string();
        let agent: Option<&OsStr> = agent_mode.then(|| OsStr::new("1"));
        let dialogs = self.dir.join("dialogs");
        let response = temp_env::with_vars(
            [
                ("FORMAL_AI_MEMORY_PATH", Some(self.path.as_os_str())),
                ("FORMAL_AI_DIALOG_LOG_DIR", Some(dialogs.as_os_str())),
                ("FORMAL_AI_AGENT_MODE", agent),
                ("FORMAL_AI_RECORD_CHAT", None),
                (SHARE_VARIABLE, None),
            ],
            || handle_api_request("POST", "/v1/chat/completions", &body),
        );
        assert_eq!(response.status_code, 200, "{}", response.body);
        response.body
    }

    fn content(&self, user: Option<&str>, prompt: &str) -> String {
        let mut body = json!({"messages": [{"role": "user", "content": prompt}]});
        if let Some(user) = user {
            body["user"] = json!(user);
        }
        let text = self.chat(&body, false);
        let completion: Value = serde_json::from_str(&text).expect("chat completion json");
        completion["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    }

    fn events(&self) -> Vec<MemoryEvent> {
        SyncStore::open_at(&self.path).events().to_vec()
    }

    /// One dreaming pass over the stored memory: the same plan and application
    /// the idle runtime performs, without its foreground gate (other tests of
    /// this binary may be answering requests at the same time).
    fn dream(&self) {
        let mut store = MemoryStore::load_from_file(&self.path).expect("load memory");
        let plan = plan_memory_dreaming(store.events(), &DreamingConfig::default());
        let _ = apply_dreaming_plan(&mut store, &plan);
        store.save_to_file(&self.path).expect("save memory");
    }

    fn append(&self, event: MemoryEvent) {
        let mut store = SyncStore::open_at(&self.path);
        store
            .import_links_notation(&export_memory_links_notation(&[event]))
            .expect("append event");
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn amendments_of(events: &[MemoryEvent]) -> Vec<&MemoryEvent> {
    events
        .iter()
        .filter(|event| event.kind.as_deref() == Some("meta_algorithm_amendment"))
        .collect()
}

fn value_of(field: Option<&str>, key: &str) -> String {
    let field = field.unwrap_or_default();
    field
        .strip_prefix(&format!("{key}="))
        .unwrap_or(field)
        .to_owned()
}

fn scope_key_of(event: &MemoryEvent) -> String {
    scope_key(&scope_pairs(&event.evidence))
}

fn assert_bounded_amendments(events: &[MemoryEvent]) {
    for amendment in amendments_of(events) {
        let topic = value_of(amendment.inputs.as_deref(), "topic");
        let rule = value_of(amendment.outputs.as_deref(), "rule");
        assert!(
            is_valid_topic(&topic),
            "pronoun or function-word topic stored: {topic}"
        );
        assert!(
            rule.chars().count() <= rule_maximum_characters(),
            "rule over the limit: {}",
            rule.len()
        );
        assert!(
            !contains_task_identifier(&rule),
            "rule names one task: {rule}"
        );
        for identifier in [
            SCALA_REPOSITORY,
            SCALA_BRANCH,
            RUST_REPOSITORY,
            RUST_BRANCH,
            "You are an AI issue solver",
        ] {
            assert!(!rule.contains(identifier), "rule carries {identifier}");
        }
    }
}

fn user_scope(user: &str, messages: &[ChatMessage]) -> RequestScope {
    let body = json!({"user": user}).to_string();
    temp_env::with_var(SHARE_VARIABLE, None::<&str>, || {
        request_scope(&body, messages, true)
    })
}

#[test]
fn the_solver_prompt_opens_like_a_session_instruction_and_yields_no_bounded_statement() {
    assert!(is_session_instruction(SCALA_PROMPT));
    assert_eq!(bounded_statement(SCALA_PROMPT), None);
    // Without its opener, no single sentence of the prompt survives the bounds
    // with a task identifier in it.
    let body = SCALA_PROMPT.lines().skip(1).collect::<Vec<_>>().join("\n");
    if let Some(statement) = bounded_statement(&body) {
        assert!(statement.chars().count() <= rule_maximum_characters());
        assert!(!contains_task_identifier(&statement), "{statement}");
    }
    assert_eq!(
        bounded_statement(&format!("Always push only to branch {SCALA_BRANCH}.")),
        None
    );
    assert_eq!(
        bounded_statement(&format!("Always use gh pr list --repo {SCALA_REPOSITORY}.")),
        None
    );
    assert_eq!(
        bounded_statement("Always keep logs in /tmp/gh-issue-solver-1794699168252/ci-logs."),
        None
    );
    assert_eq!(
        bounded_statement("Never close issue #1 without a test."),
        None
    );
    assert_eq!(
        bounded_statement(&format!(
            "Always {}politely.",
            "answer briefly and ".repeat(20)
        )),
        None
    );
    assert_eq!(
        bounded_statement("Always answer in Russian.").as_deref(),
        Some("Always answer in Russian.")
    );
    assert_eq!(
        repository_of(SCALA_PROMPT),
        Some(format!("github.com/{SCALA_REPOSITORY}"))
    );
    assert_eq!(
        repository_of(&rust_prompt()),
        Some(format!("github.com/{RUST_REPOSITORY}"))
    );
}

#[test]
fn pronouns_and_function_words_are_never_topics_in_en_ru_hi_and_zh() {
    for (statement, topic) in [
        ("You must always answer in Russian.", "answer"),
        ("You should always run the linter.", "run"),
        ("Ты всегда должен отвечать по-русски.", "отвечать"),
        ("Вы должны всегда писать тесты.", "писать"),
        ("तुम्हें हमेशा परीक्षण लिखने चाहिए।", "परीक्षण"),
        ("你必须总是用中文回答。", "回答"),
        ("你 必须 始终 写 测试", "测试"),
        ("It must never break the build.", "break"),
    ] {
        assert_eq!(
            statement_topic(statement).as_deref(),
            Some(topic),
            "{statement}"
        );
    }
    for pronoun in [
        "you",
        "it",
        "they",
        "ты",
        "вы",
        "он",
        "आप",
        "तुम",
        "你",
        "您",
        "他们",
    ] {
        assert!(is_topic_stopword(pronoun), "{pronoun}");
        assert!(!is_valid_topic(pronoun), "{pronoun}");
    }
    assert!(is_valid_topic("answer"));
}

#[test]
fn replaying_the_two_recorded_prompts_shows_no_text_of_the_first_prompt_in_the_second_session() {
    let host = Host::new("two-sessions");
    let rust_prompt = rust_prompt();
    host.chat(
        &json!({
            "messages": [
                {"role": "system", "content": AGENT_SYSTEM},
                {"role": "user", "content": SCALA_PROMPT}
            ],
            "tools": tools()
        }),
        true,
    );

    let events = host.events();
    let recorded = events
        .iter()
        .filter(|event| event.kind.as_deref() == Some("message"))
        .collect::<Vec<_>>();
    assert_eq!(recorded.len(), 1, "{recorded:?}");
    assert_eq!(
        recorded[0].role.as_deref(),
        Some("instruction"),
        "the solver prompt is marked as an instruction at receipt"
    );
    assert_eq!(
        scope_pairs(&recorded[0].evidence)
            .into_iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>(),
        ["repository", "session"]
    );
    assert!(
        recorded[0]
            .evidence
            .contains(&format!("scope:repository=github.com/{SCALA_REPOSITORY}"))
    );

    host.dream();
    let after_dreaming = host.events();
    assert!(
        amendments_of(&after_dreaming).is_empty(),
        "no requirement is learned from a session instruction"
    );
    assert_bounded_amendments(&after_dreaming);

    // The amendment the issue found on the Hive Mind host is in the store too.
    host.append(legacy_amendment());
    let second = host.chat(
        &json!({
            "messages": [
                {"role": "system", "content": AGENT_SYSTEM},
                {"role": "user", "content": rust_prompt}
            ],
            "tools": tools()
        }),
        true,
    );
    for leaked in [SCALA_REPOSITORY, SCALA_BRANCH, AMENDMENT_LINE] {
        assert!(
            !second.contains(leaked),
            "the second session shows {leaked}"
        );
    }

    // The working-session summary the issue quoted is the agentic final answer
    // of the second session, given the memory that session sees.
    let messages = vec![
        ChatMessage::new("system", AGENT_SYSTEM),
        ChatMessage::user(rust_prompt.as_str()),
        ChatMessage::assistant("Implemented the greeting."),
        ChatMessage::user("Summarize the work: run ls in the terminal"),
    ];
    let scope = temp_env::with_var(SHARE_VARIABLE, None::<&str>, || {
        request_scope("{}", &messages, true)
    });
    let request: ChatCompletionRequest = serde_json::from_value(json!({
        "model": "formal-ai",
        "messages": messages,
        "tools": [{
            "type": "function",
            "function": {"name": "read_file", "parameters": {"type": "object"}}
        }]
    }))
    .expect("valid tool-bearing chat request");
    let agent = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let stored = host.events();
    let summary = create_chat_completion_with_solver_and_memory(
        &request,
        &agent,
        &events_in_scope(&stored, Some(&scope)),
    );
    let summary = serde_json::to_string(&summary).expect("summary json");
    for leaked in [
        SCALA_REPOSITORY,
        SCALA_BRANCH,
        AMENDMENT_LINE,
        "You are an AI issue solver",
    ] {
        assert!(
            !summary.contains(leaked),
            "the second summary shows {leaked}"
        );
    }
}

#[test]
fn a_requirement_stated_inside_one_agent_session_stays_in_that_session() {
    let host = Host::new("one-session");
    host.chat(
        &json!({
            "messages": [
                {"role": "system", "content": AGENT_SYSTEM},
                {"role": "user", "content": SCALA_PROMPT},
                {"role": "user", "content": "Always run the formatter before committing."}
            ],
            "tools": tools()
        }),
        true,
    );
    host.dream();
    let events = host.events();
    let amendments = amendments_of(&events);
    assert_eq!(amendments.len(), 1, "{amendments:?}");
    let amendment = amendments[0];
    assert_eq!(value_of(amendment.inputs.as_deref(), "topic"), "run");
    assert_eq!(
        value_of(amendment.outputs.as_deref(), "rule"),
        "Always run the formatter before committing."
    );
    assert!(
        amendment
            .evidence
            .contains(&format!("scope:repository=github.com/{SCALA_REPOSITORY}"))
    );
    assert_bounded_amendments(&events);

    let (scala, rust) = temp_env::with_var(SHARE_VARIABLE, None::<&str>, || {
        (
            request_scope("{}", &session(SCALA_PROMPT), true),
            request_scope("{}", &session(&rust_prompt()), true),
        )
    });
    assert_eq!(
        amendments_of(&events_in_scope(&events, Some(&scala))).len(),
        1
    );
    assert!(amendments_of(&events_in_scope(&events, Some(&rust))).is_empty());
    assert!(amendments_of(&events_in_scope(&events, None)).is_empty());

    let second = host.chat(
        &json!({
            "messages": [
                {"role": "system", "content": AGENT_SYSTEM},
                {"role": "user", "content": format!("{}\nPlease run the formatter.", rust_prompt())}
            ],
            "tools": tools()
        }),
        true,
    );
    assert!(!second.contains("formatter before committing"));
    assert!(!second.contains(AMENDMENT_LINE));
}

#[test]
fn sharing_standing_requirements_between_agent_sessions_is_an_explicit_opt_in() {
    let messages = session(SCALA_PROMPT);
    let isolated = temp_env::with_var(SHARE_VARIABLE, None::<&str>, || {
        request_scope("{}", &messages, true)
    });
    assert_eq!(
        recorded_scope_evidence(&isolated)
            .iter()
            .map(|entry| entry.split('=').next().unwrap_or_default().to_owned())
            .collect::<Vec<_>>(),
        ["scope:repository", "scope:session"]
    );
    let shared = temp_env::with_var(SHARE_VARIABLE, Some("1"), || {
        request_scope("{}", &messages, true)
    });
    assert!(recorded_scope_evidence(&shared).is_empty());
    assert_eq!(
        recorded_scope_evidence(&user_scope("alice", &messages)),
        ["scope:user=alice"]
    );
    let chat = temp_env::with_var(SHARE_VARIABLE, None::<&str>, || {
        request_scope("{}", &messages, false)
    });
    assert!(recorded_scope_evidence(&chat).is_empty());
}

#[test]
fn the_stored_amendment_of_the_issue_is_never_applied() {
    let legacy = [legacy_amendment()];
    assert!(retained_amendments(&legacy).is_empty());
    let request: ChatCompletionRequest = serde_json::from_value(json!({
        "messages": [{"role": "user", "content": "Can you summarize what you changed?"}]
    }))
    .expect("valid chat request");
    let completion = create_chat_completion_with_solver_and_memory(
        &request,
        &UniversalSolver::default(),
        &legacy,
    );
    let answer = completion.choices[0].message.content.plain_text();
    assert!(!answer.contains(AMENDMENT_LINE), "{answer}");
    assert!(!answer.contains(SCALA_REPOSITORY), "{answer}");
}

#[test]
fn always_answer_in_russian_is_learned_and_applied_in_that_users_later_conversations_only() {
    let host = Host::new("russian");
    host.content(Some("alice"), "Always answer in Russian.");
    host.dream();
    let events = host.events();
    let amendments = amendments_of(&events);
    assert_eq!(amendments.len(), 1, "{amendments:?}");
    assert_eq!(amendments[0].inputs.as_deref(), Some("topic=answer"));
    assert_eq!(
        amendments[0].outputs.as_deref(),
        Some("rule=Always answer in Russian.")
    );
    assert!(
        amendments[0]
            .evidence
            .contains(&String::from("scope:user=alice"))
    );
    assert_bounded_amendments(&events);

    let alice = host.content(Some("alice"), "What is 2 + 2?");
    assert!(
        alice.contains("Learned standing requirement (answer): Always answer in Russian."),
        "{alice}"
    );
    let prime = host.content(Some("alice"), "Name a prime number.");
    assert!(prime.contains("Always answer in Russian."), "{prime}");
    let bob = host.content(Some("bob"), "What is 2 + 2?");
    assert!(!bob.contains("Russian"), "{bob}");
    let anonymous = host.content(None, "What is 2 + 2?");
    assert!(!anonymous.contains("Russian"), "{anonymous}");
}

#[test]
fn a_statement_made_in_two_scopes_stays_scoped_until_replay_verifies_a_generalization() {
    let host = Host::new("two-scopes");
    host.content(Some("alice"), "Always answer in Russian.");
    host.content(Some("bob"), "Always answer in Russian.");
    host.dream();
    // No recorded task replays the rule, so nothing verifies a generalization.
    let events = host.events();
    let mut scopes = amendments_of(&events)
        .into_iter()
        .map(scope_key_of)
        .collect::<Vec<_>>();
    scopes.sort();
    assert_eq!(scopes, ["user=alice", "user=bob"]);
    assert!(!amendments_of(&events).iter().any(|amendment| {
        amendment
            .evidence
            .iter()
            .any(|entry| entry == "scope:generalized")
    }));
    let carol = host.content(Some("carol"), "What is 2 + 2?");
    assert!(!carol.contains("Russian"), "{carol}");
}

#[test]
fn a_statement_made_in_two_scopes_and_verified_by_replay_becomes_one_generalized_amendment() {
    let rule = "Always include a LaTeX verification step.";
    let input = "Explain a latex proof by induction";
    let requirement = |id: &str, user: &str| MemoryEvent {
        id: id.to_owned(),
        kind: Some(String::from("message")),
        role: Some(String::from("user")),
        content: Some(rule.to_owned()),
        conversation_title: Some(String::from("latex")),
        evidence: vec![format!("scope:user={user}")],
        ..MemoryEvent::default()
    };
    let verified_run = MemoryEvent {
        id: String::from("run-verified"),
        kind: Some(String::from("test_run")),
        role: Some(String::from("derived")),
        content: Some(input.to_owned()),
        inputs: Some(input.to_owned()),
        outputs: Some(replay_answer_with_amendments(
            input,
            &[RetainedAmendment {
                id: String::from("expected"),
                topic: String::from("latex"),
                rule: rule.to_owned(),
            }],
        )),
        conversation_title: Some(String::from("latex")),
        ..MemoryEvent::default()
    };
    let learn = |events: Vec<MemoryEvent>| -> Vec<MemoryEvent> {
        let mut store = MemoryStore::from_events(events);
        let plan = plan_memory_dreaming(store.events(), &DreamingConfig::default());
        let _ = apply_dreaming_plan(&mut store, &plan);
        amendments_of(store.events()).into_iter().cloned().collect()
    };

    let scoped = learn(vec![
        requirement("req-alice", "alice"),
        requirement("req-bob", "bob"),
    ]);
    let mut scopes = scoped.iter().map(scope_key_of).collect::<Vec<_>>();
    scopes.sort();
    assert_eq!(scopes, ["user=alice", "user=bob"]);

    let generalized = learn(vec![
        requirement("req-alice", "alice"),
        requirement("req-bob", "bob"),
        verified_run,
    ]);
    assert_eq!(generalized.len(), 1, "{generalized:?}");
    assert!(scope_pairs(&generalized[0].evidence).is_empty());
    assert!(
        generalized[0]
            .evidence
            .contains(&String::from("scope:generalized"))
    );
    assert_eq!(
        generalized[0]
            .evidence
            .iter()
            .filter(|entry| entry.starts_with("req-"))
            .collect::<Vec<_>>(),
        ["req-alice", "req-bob"]
    );
    let carol = RequestScope {
        user: Some(String::from("carol")),
        ..RequestScope::default()
    };
    assert_eq!(
        retained_amendments(&events_in_scope(&generalized, Some(&carol))).len(),
        1
    );
}

#[test]
fn an_agentic_final_answer_lists_applied_amendments_in_its_trace_never_in_its_text() {
    let amendment = MemoryEvent {
        id: String::from("amendment_scoped_answer"),
        kind: Some(String::from("meta_algorithm_amendment")),
        role: Some(String::from("system")),
        content: Some(String::from("Always answer in Russian.")),
        inputs: Some(String::from("topic=answer")),
        outputs: Some(String::from("rule=Always answer in Russian.")),
        evidence: vec![
            String::from("chat_user_1"),
            String::from("scope:user=alice"),
            String::from("recipe:data/meta/dreaming-recipe.lino"),
        ],
        ..MemoryEvent::default()
    };
    let memory = [amendment];
    let prompt = "run ls in the terminal";
    let messages = vec![ChatMessage::user(prompt)];
    let alice = user_scope("alice", &messages);
    let bob = user_scope("bob", &messages);
    // A recognised shell task with only a non-shell tool advertised makes the
    // planner emit its final answer immediately.
    let request: ChatCompletionRequest = serde_json::from_value(json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": prompt}],
        "tools": [{
            "type": "function",
            "function": {"name": "read_file", "parameters": {"type": "object"}}
        }]
    }))
    .expect("valid tool-bearing chat request");
    let agent = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });

    let chat = create_chat_completion_with_solver_and_memory(
        &request,
        &agent,
        &events_in_scope(&memory, Some(&alice)),
    );
    assert_eq!(chat.choices[0].finish_reason, "stop");
    let message = &chat.choices[0].message;
    let text = message.content.plain_text();
    assert!(!text.contains("Always answer in Russian"), "{text}");
    assert!(!text.contains(AMENDMENT_LINE), "{text}");
    assert_eq!(
        message
            .thinking_steps
            .iter()
            .map(|step| step.source_event.as_str())
            .collect::<Vec<_>>(),
        ["meta_algorithm_amendment:amendment_scoped_answer"]
    );
    assert_eq!(
        message.thinking_steps[0].detail,
        "standing_requirement:answer"
    );
    assert!(!message.reasoning_content.is_empty());
    assert!(
        !message
            .reasoning_content
            .contains("Always answer in Russian")
    );

    let responses: ResponsesRequest = serde_json::from_value(json!({
        "model": "formal-ai",
        "input": prompt,
        "tools": [{"type": "function", "name": "read_file", "parameters": {"type": "object"}}]
    }))
    .expect("valid tool-bearing responses request");
    let response = create_response_with_solver_and_memory(
        &responses,
        &agent,
        &events_in_scope(&memory, Some(&alice)),
    );
    assert!(
        response.evidence_links.contains(&String::from(
            "meta_algorithm_amendment:amendment_scoped_answer"
        )),
        "{:?}",
        response.evidence_links
    );
    let output = serde_json::to_string(&response.output).expect("output json");
    assert!(!output.contains("Always answer in Russian"), "{output}");

    let other = create_chat_completion_with_solver_and_memory(
        &request,
        &agent,
        &events_in_scope(&memory, Some(&bob)),
    );
    assert!(other.choices[0].message.thinking_steps.is_empty());
    assert!(
        !other.choices[0]
            .message
            .content
            .plain_text()
            .contains("Russian")
    );
}
