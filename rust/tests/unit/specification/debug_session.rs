//! The step-through debug session (issue #667, R383).
//!
//! `serve --debug-session` binds loopback only, every debug request needs the
//! session token, a paused turn advances one stage per authenticated advance (a
//! repeated or stale advance answers 409 and records nothing), and a release or
//! a client disconnect never leaves a solve paused. The JavaScript twin is
//! `rust/tests/web/server-debug-session.test.mjs`; the protocol is
//! `docs/vscode/debugger.md`.

use std::time::Duration;

use formal_ai::server::{
    DebugSession, DebugToken, debug_session_banner, enable_debug_session, handle_debug_request,
    is_loopback_host, with_connection_scope,
};
use formal_ai::{ApiAuthConfig, ApiHttpResponse, ThinkingStep, handle_api_request_with_auth};
use serde_json::{Value, json};

const TOKEN: &str = "debug-session-test-token";
const BEARER: &str = "debug-session-bearer";

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
    assert_eq!(
        snapshot["events"][2],
        json!({
            "detail": "4", "id": "debug_event_3", "kind": "stage_paused",
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
                let mut paused = paused_turn(session);
                let total = paused["stages"].as_u64().expect("stage count");
                assert!(total >= 2, "the turn has stages ({total})");
                for stage in 0..total {
                    assert_eq!(paused["stage"], stage);
                    assert!(!answered.is_finished(), "held before stage {stage}");
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
                assert!(
                    body["choices"][0]["message"]["content"]
                        .as_str()
                        .is_some_and(|content| content.contains('4'))
                );
            });
        },
    );
    let _ = std::fs::remove_dir_all(&dir);
}
