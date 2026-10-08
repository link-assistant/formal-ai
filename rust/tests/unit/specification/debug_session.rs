//! The step-through debug session (issue #667, R383).
//!
//! `serve --debug-session` binds loopback only, every debug request needs the
//! session token, a paused turn advances one stage per authenticated advance (a
//! repeated or stale advance answers 409 and records nothing), and a release or
//! a client disconnect never leaves a solve paused. Every stage event carries
//! the turn's recipe as Mermaid source with the stage highlighted, the
//! `path:symbol` locations, lines and excerpts of the code that emits that
//! stage in both runtimes (`rust/src/server/debug_stage_sources.rs` over the
//! generated `data/meta/debug-stage-sources.lino`), and the method-registry
//! method the turn's route resolves to with its handlers
//! (`rust/src/server/debug_stage.rs`); a held turn is neither persisted nor
//! answered. The JavaScript twin is
//! `rust/tests/web/server-debug-session.test.mjs`; the protocol is
//! `docs/vscode/debugger.md`.

use std::time::Duration;

use formal_ai::derivation::{answer_derivation_id, store_path};
use formal_ai::server::{
    DebugSession, DebugToken, EXCERPT_LINES, StageView, TurnView, debug_session_banner,
    describe_turn, enable_debug_session, handle_debug_request, is_loopback_host, stage_diagram,
    with_connection_scope,
};
use formal_ai::{ApiAuthConfig, ApiHttpResponse, ThinkingStep, handle_api_request_with_auth};
use serde_json::{Value, json};

const TOKEN: &str = "debug-session-test-token";
const BEARER: &str = "debug-session-bearer";
const SOLVE_ENTRY: &str =
    "rust/src/solver.rs:solve_with_history_probability_store_and_intent_cache";
const EVENT_LOG: &str = "js/worker/formal_ai_worker_solver_events.js:solverEventLog";
const ARITHMETIC: &str = "rust/src/solver_handlers/mod.rs:try_arithmetic";
const CALCULATION_EVENTS: &str =
    "js/worker/formal_ai_worker_solver_events.js:solverCalculationEvents";
const FINALIZE: &str = "rust/src/solver_handlers/mod.rs:finalize_simple";
const INTENT_RECORD: (&str, &str) = (
    "rust/src/intent_formalization.rs:record_intent_formalization",
    "js/agentic/crate/intent_formalization.mjs:recordIntentFormalization",
);

/// Every stage of the served `What is 2 + 2?` turn: `(step, Rust emitter, JS emitter)`.
const TWO_PLUS_TWO: [(&str, &str, &str); 10] = [
    ("impulse", SOLVE_ENTRY, EVENT_LOG),
    ("detect_language", SOLVE_ENTRY, EVENT_LOG),
    ("formalize", INTENT_RECORD.0, INTENT_RECORD.1),
    ("compute", ARITHMETIC, CALCULATION_EVENTS),
    ("compute_engine", ARITHMETIC, CALCULATION_EVENTS),
    ("compute_expression", ARITHMETIC, CALCULATION_EVENTS),
    ("compute_steps", ARITHMETIC, CALCULATION_EVENTS),
    ("dispatch_handler", FINALIZE, EVENT_LOG),
    ("rule_verification", FINALIZE, EVENT_LOG),
    ("deformalize", FINALIZE, EVENT_LOG),
];

fn session() -> DebugSession {
    DebugSession::new(DebugToken {
        token: TOKEN.to_owned(),
        generated: false,
    })
}

fn stages() -> Vec<ThinkingStep> {
    vec![
        ThinkingStep::new(0, "impulse", "What is 2 + 2?", "high", "impulse"),
        ThinkingStep::new(1, "compute", "4", "high", "calculation"),
        ThinkingStep::new(2, "deformalize", "4", "high", "response"),
    ]
}

/// A turn routed to `arithmetic`, with a sub-stage under `compute`.
fn routed() -> Vec<ThinkingStep> {
    let compute = ThinkingStep::new(2, "compute", "2 + 2 = 4", "high", "calculation");
    let engine = ThinkingStep::new(
        3,
        "compute_engine",
        "link-calculator",
        "high",
        "calculation:engine",
    )
    .with_parent(compute.id.clone());
    vec![
        ThinkingStep::new(0, "impulse", "What is 2 + 2?", "high", "impulse"),
        ThinkingStep::new(
            1,
            "formalize",
            "arithmetic",
            "high",
            "intent_formalization:route",
        ),
        compute,
        engine,
        ThinkingStep::new(4, "deformalize", "2 + 2 = 4", "high", "response"),
    ]
}

/// The repository the test runs in (`rust/` is the working directory).
fn repository() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The located excerpt is the file's own text from the definition line on.
///
/// A method's definition line is indented inside its `impl` block (the solve
/// entry `solve_with_history_probability_store_and_intent_cache` is one), so
/// the indentation is trimmed before the visibility, as the JavaScript twin's
/// unanchored `fn <symbol>(` pattern does.
fn assert_excerpt(source: &str, line: u64, excerpt: &str, definition: &str) {
    let file = source.rsplit_once(':').map_or(source, |(file, _)| file);
    let text = std::fs::read_to_string(repository().join(file)).expect("located file");
    let lines: Vec<&str> = text.split('\n').collect();
    let shown: Vec<&str> = excerpt.split('\n').collect();
    let start = usize::try_from(line).expect("line") - 1;
    assert!(
        lines[start]
            .trim_start()
            .trim_start_matches("pub(crate) ")
            .trim_start_matches("pub ")
            .trim_start_matches("export ")
            .starts_with(definition),
        "{}",
        lines[start]
    );
    assert_eq!(shown, lines[start..start + shown.len()]);
    assert!(shown.len() <= EXCERPT_LINES, "an excerpt is capped");
}

/// The text of a string field of an event, empty when absent.
fn text<'event>(event: &'event Value, field: &str) -> &'event str {
    event[field].as_str().unwrap_or("")
}

/// Every stage names its own emitters, located in the files they live in, and
/// the routed method stays a separate field (empty while nothing is routed).
fn assert_stage_sources(events: &[&Value], expected: &[(&str, &str, &str)]) {
    let named: Vec<(&str, &str, &str)> = events
        .iter()
        .map(|event| {
            (
                text(event, "step"),
                text(event, "rust_source"),
                text(event, "js_source"),
            )
        })
        .collect();
    assert_eq!(named, expected);
    for event in events {
        for (runtime, keyword) in [("rust", "fn "), ("js", "function ")] {
            let source = text(event, &format!("{runtime}_source"));
            let symbol = source.rsplit_once(':').map_or("", |(_, symbol)| symbol);
            let line = event[format!("{runtime}_line")].as_u64().unwrap_or(0);
            let excerpt = text(event, &format!("{runtime}_excerpt"));
            assert_excerpt(source, line, excerpt, &format!("{keyword}{symbol}"));
        }
        let routed = text(event, "method") == "arithmetic";
        assert!(routed || text(event, "method").is_empty());
        assert_eq!(
            text(event, "method_rust_source"),
            if routed {
                "rust/src/solver_dispatch.rs:handle_arithmetic"
            } else {
                ""
            }
        );
        assert_eq!(
            text(event, "method_js_source"),
            if routed {
                "js/worker/formal_ai_worker_06.js:tryArithmetic"
            } else {
                ""
            }
        );
    }
}

fn kinds(snapshot: &Value) -> Vec<String> {
    snapshot["events"]
        .as_array()
        .expect("events")
        .iter()
        .map(|event| {
            format!(
                "{}:{}",
                event["kind"].as_str().unwrap_or(""),
                event["stage"]
            )
        })
        .collect()
}

/// Poll until a turn is paused, and return the paused entry.
fn paused_turn(session: &DebugSession) -> Value {
    for _ in 0..600 {
        let snapshot = session.snapshot(None);
        if let Some(paused) = snapshot["paused"]
            .as_array()
            .and_then(|paused| paused.first())
        {
            return paused.clone();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("no turn paused");
}

fn error_message(response: &ApiHttpResponse) -> String {
    let body: Value = serde_json::from_str(&response.body).expect("json error body");
    body["error"]["message"].as_str().unwrap_or("").to_owned()
}

#[test]
fn only_loopback_hosts_may_carry_a_debug_session() {
    for host in [
        "127.0.0.1",
        "127.1.2.3",
        "localhost",
        "LOCALHOST",
        "::1",
        "[::1]",
    ] {
        assert!(is_loopback_host(host), "{host}");
    }
    for host in [
        "0.0.0.0",
        "192.168.1.5",
        "::",
        "example.com",
        "128.0.0.1",
        "127.0.0.256",
        "",
    ] {
        assert!(!is_loopback_host(host), "{host}");
    }
    assert_eq!(
        enable_debug_session("0.0.0.0").map(|_| ()),
        Err(String::from("debug_session_requires_loopback:0.0.0.0"))
    );
}

#[test]
fn the_banner_prints_only_a_generated_token() {
    let given = session();
    assert_eq!(
        debug_session_banner(&given),
        format!("debug_session={}", given.id())
    );
    assert!(given.id().starts_with("debug_session_"));
    let generated = DebugSession::new(DebugToken {
        token: String::from("abc"),
        generated: true,
    });
    assert_eq!(
        debug_session_banner(&generated),
        format!("debug_session={};token=abc", generated.id())
    );
    let fresh = formal_ai::server::generate_debug_token();
    assert_eq!(fresh.len(), 48);
    assert!(fresh.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_ne!(fresh, formal_ai::server::generate_debug_token());
    assert!(given.authorizes(TOKEN));
    assert!(!given.authorizes("debug-session-test-tokem"));
    assert!(!given.authorizes(""));
}

#[test]
fn stepping_off_never_holds_a_turn() {
    let session = session();
    session.gate(&stages(), &mut || true);
    assert_eq!(kinds(&session.snapshot(None)), Vec::<String>::new());
}

#[test]
fn each_advance_reveals_exactly_one_stage_and_a_repeat_records_nothing() {
    let session = session();
    session.pause();
    let stages = stages();
    std::thread::scope(|scope| {
        let held = scope.spawn(|| session.gate(&stages, &mut || true));
        let paused = paused_turn(&session);
        assert_eq!(paused["turn"], "turn_1");
        assert_eq!(paused["stage"], 0);
        assert!(!session.advance("turn_1", Some(1)), "stage 1 is not paused");
        assert!(!session.advance("turn_9", Some(0)), "an unknown turn");
        assert!(
            !session.advance("turn_1", None),
            "an advance names its stage"
        );
        assert!(session.advance("turn_1", Some(0)));
        assert!(!session.advance("turn_1", Some(0)), "stage 0 was advanced");
        assert!(session.advance("turn_1", Some(1)));
        assert!(!held.is_finished(), "held at the last stage");
        assert!(session.advance("turn_1", Some(2)));
        held.join().expect("the turn completes");
    });
    let snapshot = session.snapshot(None);
    assert_eq!(
        kinds(&snapshot),
        [
            "stage_paused:0",
            "stage_advanced:0",
            "stage_paused:1",
            "stage_advanced:1",
            "stage_paused:2",
            "stage_advanced:2",
            "turn_released:2",
        ]
    );
    let three_stage_diagram = [
        "flowchart TD",
        "    s0[\"0 impulse\"]",
        "    s1[\"1 compute\"]",
        "    s2[\"2 deformalize\"]",
        "    s0 --> s1",
        "    s1 --> s2",
        "    classDef current stroke-width:4px",
        "    class s1 current",
    ]
    .join("\n");
    // No route: no method. The JavaScript `calculation` event has one
    // appender, so it is the emitter; the Rust one has several and none is
    // reached from the solver entry, so it records no location.
    let mut paused = snapshot["events"][2].clone();
    assert_eq!(text(&paused, "js_source"), CALCULATION_EVENTS);
    assert_excerpt(
        CALCULATION_EVENTS,
        paused["js_line"].as_u64().unwrap_or(0),
        text(&paused, "js_excerpt"),
        "function solverCalculationEvents(",
    );
    if let Some(fields) = paused.as_object_mut() {
        for field in ["js_excerpt", "js_line", "js_source"] {
            fields.remove(field);
        }
    }
    assert_eq!(
        paused,
        json!({
            "detail": "4", "id": "debug_event_3", "kind": "stage_paused", "method": "",
            "method_js_excerpt": "", "method_js_line": 0, "method_js_source": "",
            "method_rust_excerpt": "", "method_rust_line": 0, "method_rust_source": "",
            "mermaid": three_stage_diagram,
            "rust_excerpt": "", "rust_line": 0, "rust_source": "",
            "session": session.id(), "source": "calculation", "stage": 1,
            "stages": 3, "step": "compute", "turn": "turn_1",
        })
    );
    assert_eq!(snapshot["events"][6]["reason"], "completed");
    assert_eq!(snapshot["next"], 7);
    assert_eq!(
        session.snapshot(Some(6))["events"],
        json!([snapshot["events"][6].clone()])
    );
    assert!(!session.advance("turn_1", Some(2)), "a released turn");
}

#[test]
fn every_stage_carries_the_recipe_diagram_its_own_emitters_and_the_routed_method() {
    let session = session();
    session.pause();
    let stages = routed();
    std::thread::scope(|scope| {
        let held = scope.spawn(|| session.gate(&stages, &mut || true));
        paused_turn(&session);
        for stage in 0..3 {
            assert!(session.advance("turn_1", Some(stage)));
        }
        let paused = session.snapshot(None)["paused"][0].clone();
        assert_eq!(paused["stage"], 3);
        assert_eq!(paused["method"], "arithmetic");
        assert_eq!(
            paused["mermaid"],
            [
                "flowchart TD",
                "    s0[\"0 impulse\"]",
                "    s1[\"1 formalize\"]",
                "    s2[\"2 compute\"]",
                "    s3[\"3 compute_engine\"]",
                "    s4[\"4 deformalize\"]",
                "    s0 --> s1",
                "    s1 --> s2",
                "    s2 -.-> s3",
                "    s2 --> s4",
                "    classDef current stroke-width:4px",
                "    class s3 current",
            ]
            .join("\n")
        );
        let rust = text(&paused, "method_rust_source");
        let js = text(&paused, "method_js_source");
        assert_eq!(rust, "rust/src/solver_dispatch.rs:handle_arithmetic");
        assert_eq!(js, "js/worker/formal_ai_worker_06.js:tryArithmetic");
        let excerpt = text(&paused, "method_rust_excerpt");
        assert_excerpt(
            rust,
            paused["method_rust_line"].as_u64().unwrap_or(0),
            excerpt,
            "fn handle_arithmetic(",
        );
        assert_eq!(
            excerpt.rsplit('\n').next(),
            Some("}"),
            "runs to the closing brace"
        );
        assert_excerpt(
            js,
            paused["method_js_line"].as_u64().unwrap_or(0),
            text(&paused, "method_js_excerpt"),
            "function tryArithmetic(",
        );
        session.release(None);
        held.join().expect("the release frees the turn");
    });
    // Each stage names the code that emits it; the highlight moves with it.
    let events = session.snapshot(None)["events"].clone();
    let staged: Vec<&Value> = events
        .as_array()
        .expect("events")
        .iter()
        .filter(|event| event["kind"] == "stage_paused")
        .collect();
    assert_stage_sources(
        &staged,
        &[
            ("impulse", SOLVE_ENTRY, EVENT_LOG),
            ("formalize", INTENT_RECORD.0, INTENT_RECORD.1),
            ("compute", ARITHMETIC, CALCULATION_EVENTS),
            ("compute_engine", ARITHMETIC, CALCULATION_EVENTS),
        ],
    );
    let highlighted: Vec<&str> = staged
        .iter()
        .map(|event| text(event, "mermaid").rsplit('\n').next().unwrap_or(""))
        .collect();
    assert_eq!(
        highlighted,
        [
            "    class s0 current",
            "    class s1 current",
            "    class s2 current",
            "    class s3 current",
        ]
    );
    // A rule-set method resolves to the rule interpreter in both runtimes; a
    // route the registry does not know records no method, and a stage whose
    // event no function appends records no location at all.
    let rule = describe_turn(&[ThinkingStep::new(
        0,
        "formalize",
        "clarification",
        "high",
        "route",
    )]);
    let rule_rust = rule.rust.expect("rule interpreter");
    let rule_js = rule.js.expect("browser rule handler");
    assert_eq!(
        format!("{}:{}", rule_rust.path, rule_rust.symbol),
        "rust/src/rule_interpreter.rs:run_handler"
    );
    assert_eq!(
        format!("{}:{}", rule_js.path, rule_js.symbol),
        "js/worker/formal_ai_worker_handler_rules.js:tryClarification"
    );
    let unknown = describe_turn(&[ThinkingStep::new(
        0,
        "formalize",
        "greeting",
        "high",
        "no_such_event",
    )]);
    assert_eq!(
        unknown,
        TurnView {
            stages: vec![StageView::default()],
            ..TurnView::default()
        }
    );
    let quoted = stage_diagram(&[ThinkingStep::new(0, "say \"hi\"", "", "high", "x")], 0);
    assert_eq!(
        quoted.split('\n').nth(1),
        Some("    s0[\"0 say #quot;hi#quot;\"]")
    );
}

#[test]
fn a_turn_begun_before_its_solve_waits_at_its_first_stage_then_resumes() {
    let session = session();
    let stages = stages();
    assert_eq!(
        session.begin(&stages[0], &mut || true),
        None,
        "stepping off never opens a turn"
    );
    session.pause();
    std::thread::scope(|scope| {
        let opening = scope.spawn(|| session.begin(&stages[0], &mut || true));
        let paused = paused_turn(&session);
        assert_eq!(
            [
                &paused["turn"],
                &paused["stage"],
                &paused["stages"],
                &paused["step"]
            ],
            [&json!("turn_1"), &json!(0), &json!(1), &json!("impulse")]
        );
        assert!(
            !opening.is_finished(),
            "the solve waits until stage 0 is advanced"
        );
        assert!(session.advance("turn_1", Some(0)));
        let begun = opening.join().expect("the advance lets the solve run");
        assert_eq!(begun.as_deref(), Some("turn_1"));
        assert_eq!(
            session.snapshot(None)["paused"],
            json!([]),
            "a running solve has no paused stage"
        );
        assert!(!session.advance("turn_1", Some(0)), "stage 0 was advanced");
        assert!(
            !session.advance("turn_1", Some(1)),
            "stage 1 is not computed yet"
        );
        let resumed = scope.spawn(|| session.resume("turn_1", &stages, &mut || true));
        let paused = paused_turn(&session);
        assert_eq!(
            [&paused["stage"], &paused["stages"]],
            [&json!(1), &json!(3)]
        );
        assert!(session.advance("turn_1", Some(1)));
        assert!(session.advance("turn_1", Some(2)));
        resumed.join().expect("the turn completes");
    });
    let snapshot = session.snapshot(None);
    assert_eq!(
        kinds(&snapshot),
        [
            "stage_paused:0",
            "stage_advanced:0",
            "stage_paused:1",
            "stage_advanced:1",
            "stage_paused:2",
            "stage_advanced:2",
            "turn_released:2",
        ]
    );
    assert_eq!(snapshot["events"][6]["reason"], "completed");
    // A turn released while its solve runs is not held again.
    std::thread::scope(|scope| {
        let opening = scope.spawn(|| session.begin(&stages[0], &mut || true));
        paused_turn(&session);
        assert!(session.advance("turn_2", Some(0)));
        let running = opening
            .join()
            .expect("the solve runs")
            .expect("an open turn");
        session.release(Some(running.as_str()));
        session.resume(&running, &stages, &mut || true);
    });
    let snapshot = session.snapshot(None);
    let events = snapshot["events"].as_array().expect("events");
    assert_eq!(
        events.last().map(|event| &event["reason"]),
        Some(&json!("released"))
    );
    assert_eq!(snapshot["paused"], json!([]));
}

#[test]
fn a_disconnect_or_a_release_frees_a_paused_turn() {
    let session = session();
    session.pause();
    let stages = stages();
    session.gate(&stages, &mut || false);
    let snapshot = session.snapshot(None);
    assert_eq!(snapshot["events"][1]["kind"], "turn_released");
    assert_eq!(snapshot["events"][1]["reason"], "disconnected");
    std::thread::scope(|scope| {
        let held = scope.spawn(|| session.gate(&stages, &mut || true));
        let paused = paused_turn(&session);
        assert_eq!(paused["turn"], "turn_2");
        session.release(None);
        held.join().expect("the release frees the turn");
    });
    let snapshot = session.snapshot(None);
    assert_eq!(snapshot["stepping"], false);
    assert_eq!(snapshot["paused"], json!([]));
    assert_eq!(snapshot["events"][3]["reason"], "released");
}

#[test]
fn debug_requests_answer_the_twin_errors() {
    let disabled = handle_debug_request(None, "session", "{}");
    assert_eq!(disabled.status_code, 404);
    assert_eq!(error_message(&disabled), "debug_session_disabled");
    let session = session();
    let malformed = handle_debug_request(Some(&session), "session", "{not json");
    assert_eq!(malformed.status_code, 400);
    assert_eq!(error_message(&malformed), "debug_request_invalid");
    for action in ["session", "pause", "advance", "release"] {
        let refused = handle_debug_request(Some(&session), action, r#"{"token":"wrong"}"#);
        assert_eq!(refused.status_code, 401, "{action}");
        assert_eq!(error_message(&refused), "debug_session_token_invalid");
    }
    let stale = handle_debug_request(
        Some(&session),
        "advance",
        &json!({ "token": TOKEN, "turn": "turn_1", "stage": 0 }).to_string(),
    );
    assert_eq!(stale.status_code, 409);
    assert_eq!(error_message(&stale), "debug_stage_not_paused");
    let state = handle_debug_request(
        Some(&session),
        "pause",
        &json!({ "token": TOKEN }).to_string(),
    );
    assert_eq!(state.status_code, 200);
    let body: Value = serde_json::from_str(&state.body).expect("snapshot");
    assert_eq!(
        body,
        json!({
            "events": [], "next": 0, "object": "debug.session", "paused": [],
            "session": session.id(), "stepping": true,
        })
    );
}

/// The served route, end to end: a chat turn solved inside a connection scope
/// is held until its last stage is advanced through `POST /v1/debug/advance`.
#[test]
fn a_served_chat_turn_answers_only_after_its_last_stage() {
    let dir = std::env::temp_dir().join(format!("formal-ai-debug-session-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let memory = dir.join("memory.lino");
    let dialogs = dir.join("dialogs");
    temp_env::with_vars(
        [
            ("FORMAL_AI_MEMORY_PATH", Some(memory.as_os_str())),
            ("FORMAL_AI_DIALOG_LOG_DIR", Some(dialogs.as_os_str())),
            ("FORMAL_AI_RECORD_CHAT", Some(std::ffi::OsStr::new("0"))),
            (
                "FORMAL_AI_DEBUG_SESSION_TOKEN",
                Some(std::ffi::OsStr::new(TOKEN)),
            ),
        ],
        || {
            let session = enable_debug_session("127.0.0.1").expect("loopback");
            let auth = ApiAuthConfig::bearer_token(BEARER);
            let bearer = format!("Bearer {BEARER}");
            let headers = [("authorization", bearer.as_str())];
            let debug = |action: &str, body: &Value| {
                handle_api_request_with_auth(
                    "POST",
                    &format!("/v1/debug/{action}"),
                    &headers,
                    &body.to_string(),
                    &auth,
                )
            };
            // The record the turn's answer is persisted under, removed so the
            // held turn can be seen not to write it.
            let root = std::env::current_dir().expect("working directory");
            let record = store_path(&root, &formal_ai::solve("What is 2 + 2?").derivation_id())
                .expect("record path");
            let _ = std::fs::remove_file(&record);
            assert_eq!(debug("pause", &json!({ "token": TOKEN })).status_code, 200);
            let chat = json!({
                "model": "formal-ai",
                "messages": [{ "role": "user", "content": "What is 2 + 2?" }],
            })
            .to_string();
            std::thread::scope(|scope| {
                let answered = scope.spawn(|| {
                    with_connection_scope(
                        || true,
                        || {
                            handle_api_request_with_auth(
                                "POST",
                                "/v1/chat/completions",
                                &headers,
                                &chat,
                                &auth,
                            )
                        },
                    )
                });
                // Stage 0 is truly suspended: only the prompt is known, the
                // solver has not run, so nothing is routed yet.
                let mut paused = paused_turn(session);
                assert_eq!(
                    [
                        &paused["stage"],
                        &paused["stages"],
                        &paused["step"],
                        &paused["method"]
                    ],
                    [&json!(0), &json!(1), &json!("impulse"), &json!("")]
                );
                let first = json!({ "token": TOKEN, "turn": paused["turn"], "stage": 0 });
                assert_eq!(debug("advance", &first).status_code, 200);
                assert_eq!(debug("advance", &first).status_code, 409);
                // Stages 1.. are revealed from the solved turn, held before it
                // is persisted.
                paused = paused_turn(session);
                let total = paused["stages"].as_u64().expect("stage count");
                assert_eq!(total, TWO_PLUS_TWO.len() as u64);
                assert_eq!(paused["method"], "arithmetic");
                assert_eq!(
                    paused["method_rust_source"],
                    "rust/src/solver_dispatch.rs:handle_arithmetic"
                );
                for stage in 1..total {
                    assert_eq!(paused["stage"], stage);
                    assert!(!answered.is_finished(), "held before stage {stage}");
                    assert!(
                        !record.exists(),
                        "not persisted while stage {stage} is held"
                    );
                    assert!(
                        paused["mermaid"]
                            .as_str()
                            .is_some_and(|mermaid| mermaid
                                .ends_with(&format!("\n    class s{stage} current"))),
                        "stage {stage} is highlighted"
                    );
                    let request = json!({ "token": TOKEN, "turn": paused["turn"], "stage": stage });
                    let advanced = debug("advance", &request);
                    assert_eq!(advanced.status_code, 200);
                    let repeated = debug("advance", &request);
                    assert_eq!(repeated.status_code, 409, "a stage is never advanced twice");
                    if stage + 1 < total {
                        let body: Value = serde_json::from_str(&advanced.body).expect("snapshot");
                        paused = body["paused"][0].clone();
                    }
                }
                // Stepping off before the join: a second solve inside the same
                // request (a restated standing requirement) is never held.
                let released = debug("release", &json!({ "token": TOKEN }));
                let state: Value = serde_json::from_str(&released.body).expect("snapshot");
                assert_eq!(state["stepping"], false);
                let response = answered.join().expect("the turn answers");
                assert_eq!(response.status_code, 200);
                let body: Value = serde_json::from_str(&response.body).expect("completion");
                let content = body["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("");
                assert!(content.contains('4'));
                assert_eq!(
                    store_path(&root, &answer_derivation_id(content)).as_ref(),
                    Some(&record),
                    "the served answer is the one whose record was removed"
                );
                assert!(record.exists(), "the release persists the record");
            });
            let state = debug("session", &json!({ "token": TOKEN }));
            let snapshot: Value = serde_json::from_str(&state.body).expect("snapshot");
            let advanced: Vec<&Value> = snapshot["events"]
                .as_array()
                .expect("events")
                .iter()
                .filter(|event| event["kind"] == "stage_advanced")
                .collect();
            assert_stage_sources(&advanced, &TWO_PLUS_TWO);
            let counts: Vec<u64> = advanced
                .iter()
                .map(|event| event["stages"].as_u64().unwrap_or(0))
                .collect();
            let mut expected = vec![TWO_PLUS_TWO.len() as u64; TWO_PLUS_TWO.len()];
            expected[0] = 1;
            assert_eq!(
                counts, expected,
                "stage 0 was paused before the turn was solved"
            );
        },
    );
    let _ = std::fs::remove_dir_all(&dir);
}
