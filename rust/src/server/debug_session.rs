//! The loopback-only step-through debug session (issue #667, R383).
//!
//! `serve --debug-session` holds every solved turn before its derivation record
//! is persisted and its answer returned, and hands its stages out one at a
//! time, each `POST /v1/debug/advance` revealing the next. Every stage event
//! carries the turn's recipe diagram, the Rust and JavaScript source locations
//! of the code that emits that stage, and the routed method with its handler
//! in both runtimes (`debug_stage`). The JavaScript twin is
//! `js/server/debug-session.mjs`; the protocol is documented in
//! `docs/vscode/debugger.md`.
//!
//! - The session token comes from `FORMAL_AI_DEBUG_SESSION_TOKEN`, else it is
//!   generated and printed once on stderr; every debug request carries it in
//!   its JSON body (`token`), beside the server's ordinary bearer gate.
//! - Stepping starts off: `pause` turns it on, so the next solved turn stops at
//!   its first stage (`stage_paused`); `advance {turn, stage}` records
//!   `stage_advanced` for exactly the paused stage and pauses at the next, and
//!   a stale or repeated advance answers 409 and records nothing; after the
//!   last stage the turn is released (`turn_released`, reason `completed`).
//! - `release` (one turn, or every turn and stepping off) and a client that
//!   disconnects (`disconnected`) release a paused turn, so no solve stays
//!   paused once nobody can advance it.
//! - Only solves inside a connection scope are gated: background work
//!   (dreaming, the CLI) never pauses.

use std::cell::{Cell, RefCell};
use std::collections::hash_map::RandomState;
use std::fmt::Write as _;
use std::hash::{BuildHasher as _, Hasher as _};
use std::sync::{Condvar, Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::Duration;

use serde_json::{Map, Value, json};
use sha2::{Digest as _, Sha256};

use super::debug_stage::{TurnView, describe_turn, location_fields, stage_diagram};
use super::{ApiHttpResponse, error_response, json_response};
use crate::thinking::ThinkingStep;

/// The environment variable a launcher hands the session token through.
pub const DEBUG_TOKEN_ENV: &str = "FORMAL_AI_DEBUG_SESSION_TOKEN";

const STAGE_PAUSED: &str = "stage_paused";
const STAGE_ADVANCED: &str = "stage_advanced";
const TURN_RELEASED: &str = "turn_released";
const IMPULSE: &str = "impulse";
const IMPULSE_LEVEL: &str = "high";
const DEBUG_ROUTE_PREFIX: &str = "/v1/debug/";
const DEBUG_ACTIONS: [&str; 4] = ["session", "pause", "advance", "release"];

/// How often a held turn re-checks that its client is still connected.
const LIVENESS_POLL: Duration = Duration::from_millis(100);

/// Mirrors `isLoopbackHost`: `localhost`, `::1` or an address in 127.0.0.0/8.
#[must_use]
pub fn is_loopback_host(host: &str) -> bool {
    let lower = host.trim().to_ascii_lowercase();
    let value = lower
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .unwrap_or(&lower);
    if value == "localhost" || value == "::1" {
        return true;
    }
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() == 4
        && parts[0] == "127"
        && parts.iter().all(|part| {
            (1..=3).contains(&part.len())
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && part.parse::<u16>().is_ok_and(|number| number <= 255)
        })
}

/// A session token and whether the server generated it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugToken {
    /// The secret every debug request must carry.
    pub token: String,
    /// True when the server generated it, so the banner prints it once.
    pub generated: bool,
}

/// A fresh session token: 24 bytes of a SHA-256 over per-process random hash
/// keys, the clock and the process id, as hex.
#[must_use]
pub fn generate_debug_token() -> String {
    let mut digest = Sha256::new();
    for salt in 0..4_u64 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u64(salt);
        digest.update(hasher.finish().to_le_bytes());
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    digest.update(nanos.to_le_bytes());
    digest.update(std::process::id().to_le_bytes());
    digest
        .finalize()
        .iter()
        .take(24)
        .fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}

/// Mirrors `debugTokenFrom`: the launcher's token, else a generated one.
#[must_use]
pub fn debug_token_from_env() -> DebugToken {
    let given = std::env::var(DEBUG_TOKEN_ENV).unwrap_or_default();
    let given = given.trim();
    if given.is_empty() {
        DebugToken {
            token: generate_debug_token(),
            generated: true,
        }
    } else {
        DebugToken {
            token: given.to_owned(),
            generated: false,
        }
    }
}

#[derive(Debug)]
struct Turn {
    id: String,
    stages: Vec<ThinkingStep>,
    view: TurnView,
    current: usize,
    /// Begun before its solve: only the first stage is known yet.
    open: bool,
    /// Its first stage was advanced and its solve is running: nothing paused.
    running: bool,
}

#[derive(Debug, Default)]
struct State {
    stepping: bool,
    events: Vec<Value>,
    turns: Vec<Turn>,
    turn_count: u64,
}

/// Mirrors `stageFields`: what an event says about one stage.
///
/// The step, the turn's recipe diagram with this stage highlighted, the source
/// locations of the code that emits the stage (`rust_*`, `js_*`), and the
/// method the turn's route resolves to with its handler locations (`method`,
/// `method_rust_*`, `method_js_*`) (`debug_stage`).
fn stage_fields(turn: &Turn, index: usize) -> Value {
    let stage = turn.stages.get(index);
    let mut fields = Map::new();
    fields.insert(String::from("stage"), Value::from(index));
    fields.insert(String::from("stages"), Value::from(turn.stages.len()));
    for (key, text) in [
        ("step", stage.map_or("", |stage| stage.step.as_str())),
        ("detail", stage.map_or("", |stage| stage.detail.as_str())),
        (
            "source",
            stage.map_or("", |stage| stage.source_event.as_str()),
        ),
        ("method", turn.view.method.as_str()),
    ] {
        fields.insert(String::from(key), Value::from(text));
    }
    fields.insert(
        String::from("mermaid"),
        Value::from(stage_diagram(&turn.stages, index)),
    );
    let view = turn.view.stages.get(index);
    location_fields(
        &mut fields,
        "rust",
        view.and_then(|view| view.rust.as_ref()),
    );
    location_fields(&mut fields, "js", view.and_then(|view| view.js.as_ref()));
    location_fields(&mut fields, "method_rust", turn.view.rust.as_ref());
    location_fields(&mut fields, "method_js", turn.view.js.as_ref());
    Value::Object(fields)
}

impl State {
    fn record(&mut self, session: &str, kind: &str, turn: &str, fields: Value) {
        let mut event = json!({
            "id": format!("debug_event_{}", self.events.len() + 1),
            "kind": kind,
            "session": session,
            "turn": turn,
        });
        if let (Some(event), Value::Object(fields)) = (event.as_object_mut(), fields) {
            event.extend(fields);
        }
        self.events.push(event);
    }

    fn finish(&mut self, session: &str, turn_id: &str, reason: &str) {
        let Some(position) = self.turns.iter().position(|turn| turn.id == turn_id) else {
            return;
        };
        let turn = self.turns.remove(position);
        self.record(
            session,
            TURN_RELEASED,
            &turn.id,
            json!({ "reason": reason, "stage": turn.current, "stages": turn.stages.len() }),
        );
    }

    /// Whether `turn_id` is still held: present and not running its solve.
    fn holds(&self, turn_id: &str) -> bool {
        self.turns
            .iter()
            .any(|turn| turn.id == turn_id && !turn.running)
    }

    /// Mirrors `start`: open a turn over `stages`, paused at its first stage.
    fn start(&mut self, session: &str, stages: &[ThinkingStep], open: bool) -> String {
        self.turn_count += 1;
        let turn_id = format!("turn_{}", self.turn_count);
        let turn = Turn {
            id: turn_id.clone(),
            stages: stages.to_vec(),
            view: describe_turn(stages),
            current: 0,
            open,
            running: false,
        };
        let fields = stage_fields(&turn, 0);
        self.record(session, STAGE_PAUSED, &turn_id, fields);
        self.turns.push(turn);
        turn_id
    }
}

/// Mirrors `class DebugSession`: the session state behind one lock, with a
/// condition variable that wakes held turns on every change.
#[derive(Debug)]
pub struct DebugSession {
    token: DebugToken,
    id: String,
    state: Mutex<State>,
    changed: Condvar,
}

impl DebugSession {
    /// A session with stepping off and no events.
    #[must_use]
    pub fn new(token: DebugToken) -> Self {
        let id = crate::engine::stable_id("debug_session", &token.token);
        Self {
            token,
            id,
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
        }
    }

    /// The public session id (`debug_session_<16 hex>`).
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Whether `token` is this session's token (constant-time comparison).
    #[must_use]
    pub fn authorizes(&self, token: &str) -> bool {
        let expected = self.token.token.as_bytes();
        let given = token.as_bytes();
        expected.len() == given.len()
            && expected
                .iter()
                .zip(given)
                .fold(0_u8, |difference, (left, right)| {
                    difference | (left ^ right)
                })
                == 0
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Turn stepping on: the next solved turn pauses at its first stage.
    pub fn pause(&self) {
        self.lock().stepping = true;
    }

    /// Mirrors `hold`: wait until the turn is no longer held — its last known
    /// stage advanced, or the turn released (a disconnect releases it).
    fn hold<'a>(
        &'a self,
        mut state: MutexGuard<'a, State>,
        turn_id: &str,
        alive: &mut dyn FnMut() -> bool,
    ) {
        self.changed.notify_all();
        while state.holds(turn_id) {
            state = self
                .changed
                .wait_timeout(state, LIVENESS_POLL)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
            if !state.holds(turn_id) {
                break;
            }
            drop(state);
            let connected = alive();
            state = self.lock();
            if !connected {
                state.finish(&self.id, turn_id, "disconnected");
                self.changed.notify_all();
            }
        }
        drop(state);
    }

    /// Suspend a turn before the solver runs: the turn opens with only its
    /// first stage (`impulse`) known and paused. Returns once that stage is
    /// advanced (the turn is then running and the solve proceeds) or the turn
    /// is released; `None` when stepping is off. [`Self::resume`] continues
    /// the turn with the solved stages. Mirrors `begin`.
    // The guard moves into `hold` through the closure, which clippy reads as
    // a temporary that outlives its last use.
    #[allow(clippy::significant_drop_tightening)]
    pub fn begin(&self, first: &ThinkingStep, alive: &mut dyn FnMut() -> bool) -> Option<String> {
        let mut state = self.lock();
        if !state.stepping {
            return None;
        }
        let turn_id = state.start(&self.id, std::slice::from_ref(first), true);
        // The hold is the tail expression, so the guard's last use ends the
        // scope (`gate` likewise ends on its hold).
        Some(turn_id).inspect(|turn_id| self.hold(state, turn_id, alive))
    }

    /// Continue a turn [`Self::begin`] suspended, now that the solver has
    /// computed all of its stages: hold it at stage 1 until every remaining
    /// stage is advanced. A released turn, or one with no stage left, returns
    /// at once. Mirrors `resume`.
    pub fn resume(&self, turn_id: &str, stages: &[ThinkingStep], alive: &mut dyn FnMut() -> bool) {
        let mut state = self.lock();
        let Some(position) = state.turns.iter().position(|turn| turn.id == turn_id) else {
            return;
        };
        if stages.len() < 2 {
            state.finish(&self.id, turn_id, "completed");
            drop(state);
            self.changed.notify_all();
            return;
        }
        let turn = &mut state.turns[position];
        turn.stages = stages.to_vec();
        turn.view = describe_turn(stages);
        turn.current = 1;
        turn.open = false;
        turn.running = false;
        let fields = stage_fields(turn, 1);
        state.record(&self.id, STAGE_PAUSED, turn_id, fields);
        self.hold(state, turn_id, alive);
    }

    /// Hold one solved turn until every stage is advanced or the turn is
    /// released; `alive` answers false once the requesting client is gone.
    /// Returns at once when stepping is off or there is no stage.
    pub fn gate(&self, stages: &[ThinkingStep], alive: &mut dyn FnMut() -> bool) {
        let mut state = self.lock();
        if !state.stepping || stages.is_empty() {
            return;
        }
        let turn_id = state.start(&self.id, stages, false);
        self.hold(state, &turn_id, alive);
    }

    /// Release a turn whose solve ended without resuming it (`abandoned`).
    fn abandon(&self, turn_id: &str) {
        self.lock().finish(&self.id, turn_id, "abandoned");
        self.changed.notify_all();
    }

    /// Advance the paused stage `requested` of `turn_id`. False — nothing
    /// recorded — when that stage is not the one paused (already advanced,
    /// unknown turn, or a turn whose solve is running).
    #[must_use]
    pub fn advance(&self, turn_id: &str, requested: Option<u64>) -> bool {
        let mut state = self.lock();
        let Some(position) = state
            .turns
            .iter()
            .position(|turn| turn.id == turn_id && !turn.running)
        else {
            return false;
        };
        let current = state.turns[position].current;
        if requested.and_then(|index| usize::try_from(index).ok()) != Some(current) {
            return false;
        }
        let fields = stage_fields(&state.turns[position], current);
        state.record(&self.id, STAGE_ADVANCED, turn_id, fields);
        if current + 1 < state.turns[position].stages.len() {
            state.turns[position].current = current + 1;
            let fields = stage_fields(&state.turns[position], current + 1);
            state.record(&self.id, STAGE_PAUSED, turn_id, fields);
        } else if state.turns[position].open {
            state.turns[position].running = true;
        } else {
            state.finish(&self.id, turn_id, "completed");
        }
        drop(state);
        self.changed.notify_all();
        true
    }

    /// Release one turn, or — with no turn — every turn, and stop stepping.
    pub fn release(&self, turn_id: Option<&str>) {
        let mut state = self.lock();
        if let Some(turn_id) = turn_id {
            state.finish(&self.id, turn_id, "released");
        } else {
            state.stepping = false;
            let held: Vec<String> = state.turns.iter().map(|turn| turn.id.clone()).collect();
            for turn_id in held {
                state.finish(&self.id, &turn_id, "released");
            }
        }
        drop(state);
        self.changed.notify_all();
    }

    /// The session state and the events after the first `since`.
    #[must_use]
    pub fn snapshot(&self, since: Option<u64>) -> Value {
        let state = self.lock();
        let from = since
            .and_then(|since| usize::try_from(since).ok())
            .unwrap_or(0)
            .min(state.events.len());
        let paused: Vec<Value> = state
            .turns
            .iter()
            .filter(|turn| !turn.running)
            .map(|turn| {
                let mut fields = stage_fields(turn, turn.current);
                if let Some(fields) = fields.as_object_mut() {
                    fields.insert(String::from("turn"), Value::from(turn.id.clone()));
                }
                fields
            })
            .collect();
        json!({
            "object": "debug.session",
            "session": self.id,
            "stepping": state.stepping,
            "paused": paused,
            "events": state.events[from..],
            "next": state.events.len(),
        })
    }
}

static SESSION: OnceLock<DebugSession> = OnceLock::new();

/// Start the process's debug session for a server bound to `host`.
///
/// # Errors
/// `debug_session_requires_loopback:<host>` when `host` is not a loopback
/// address: a debug session never listens beyond this machine.
pub fn enable_debug_session(host: &str) -> Result<&'static DebugSession, String> {
    if !is_loopback_host(host) {
        return Err(format!("debug_session_requires_loopback:{host}"));
    }
    Ok(SESSION.get_or_init(|| DebugSession::new(debug_token_from_env())))
}

/// The process's debug session, when `serve --debug-session` started one.
#[must_use]
pub fn active_debug_session() -> Option<&'static DebugSession> {
    SESSION.get()
}

/// Mirrors `debugSessionBanner`: the stderr line naming the session, with the
/// token only when the server generated it.
#[must_use]
pub fn debug_session_banner(session: &DebugSession) -> String {
    if session.token.generated {
        format!("debug_session={};token={}", session.id, session.token.token)
    } else {
        format!("debug_session={}", session.id)
    }
}

type Liveness = Box<dyn FnMut() -> bool>;

thread_local! {
    static CONNECTION: RefCell<Option<Liveness>> = const { RefCell::new(None) };
    /// The turn [`begin_turn`] opened for this connection's outermost solve.
    static OPEN_TURN: RefCell<Option<String>> = const { RefCell::new(None) };
    /// How many solves of this thread are running, outermost first.
    static SOLVE_DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// Run `run` inside one connection's scope; `alive` answers false once the
/// client has disconnected. Mirrors `runInRequestScope`.
#[must_use]
pub fn with_connection_scope<T>(
    alive: impl FnMut() -> bool + 'static,
    run: impl FnOnce() -> T,
) -> T {
    let previous = CONNECTION.with(|slot| slot.borrow_mut().replace(Box::new(alive)));
    let result = run();
    CONNECTION.with(|slot| *slot.borrow_mut() = previous);
    result
}

/// Run `hold` with this connection's liveness check, when the server runs a
/// debug session and the solve belongs to a connection.
fn with_liveness<T>(hold: impl FnOnce(&DebugSession, &mut dyn FnMut() -> bool) -> T) -> Option<T> {
    let session = SESSION.get()?;
    let mut alive = CONNECTION.with(|slot| slot.borrow_mut().take())?;
    let result = hold(session, &mut alive);
    CONNECTION.with(|slot| *slot.borrow_mut() = Some(alive));
    Some(result)
}

/// One running solve: the outermost releases the turn it began, if its solve
/// ended without resuming it.
#[derive(Debug)]
pub struct SolveTurn {
    outermost: bool,
}

impl Drop for SolveTurn {
    fn drop(&mut self) {
        SOLVE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
        if !self.outermost {
            return;
        }
        if let (Some(session), Some(turn_id)) = (
            SESSION.get(),
            OPEN_TURN.with(|slot| slot.borrow_mut().take()),
        ) {
            session.abandon(&turn_id);
        }
    }
}

/// Mirrors `beginTurn`: suspend a connection's outermost solve before it runs,
/// at the turn's first stage (`impulse`, the prompt), when the server runs a
/// debug session. The open turn waits for [`gate_turn`]; the returned guard
/// marks the solve running until it is dropped (and mirrors `endTurn`).
#[must_use]
pub fn begin_turn(prompt: &str) -> SolveTurn {
    let outermost = SOLVE_DEPTH.with(|depth| {
        depth.set(depth.get() + 1);
        depth.get() == 1
    });
    if outermost && OPEN_TURN.with(|slot| slot.borrow().is_none()) {
        let first = ThinkingStep::new(0, IMPULSE, prompt, IMPULSE_LEVEL, IMPULSE);
        let opened = with_liveness(|session, alive| session.begin(&first, alive)).flatten();
        OPEN_TURN.with(|slot| *slot.borrow_mut() = opened);
    }
    SolveTurn { outermost }
}

/// Mirrors `gateTurn`: hold a solved turn when the server runs a debug session
/// and the solve belongs to a connection — continuing the turn
/// [`begin_turn`] opened for the outermost solve, else holding a new one.
pub fn gate_turn(stages: &[ThinkingStep]) {
    let outermost = SOLVE_DEPTH.with(|depth| depth.get() <= 1);
    let open = outermost
        .then(|| OPEN_TURN.with(|slot| slot.borrow_mut().take()))
        .flatten();
    with_liveness(|session, alive| match &open {
        Some(turn_id) => session.resume(turn_id, stages, alive),
        None => session.gate(stages, alive),
    });
}

/// The text of a JSON field the way `String(value)` reads it in the twin.
fn field_text(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), ToOwned::to_owned)
}

/// `POST /v1/debug/{session|pause|advance|release}`, or `None` for any other
/// route.
#[must_use]
pub fn handle_debug_route(method: &str, path: &str, body: &str) -> Option<ApiHttpResponse> {
    let action = path.strip_prefix(DEBUG_ROUTE_PREFIX)?;
    if method != "POST" || !DEBUG_ACTIONS.contains(&action) {
        return None;
    }
    Some(handle_debug_request(SESSION.get(), action, body))
}

/// Answer one debug request against `session`. Mirrors `handleDebug`.
#[must_use]
pub fn handle_debug_request(
    session: Option<&DebugSession>,
    action: &str,
    body: &str,
) -> ApiHttpResponse {
    let Some(session) = session else {
        return error_response(404, "debug_session_disabled");
    };
    let Ok(Value::Object(request)) = serde_json::from_str::<Value>(body) else {
        return error_response(400, "debug_request_invalid");
    };
    let token = request.get("token").and_then(Value::as_str).unwrap_or("");
    if !session.authorizes(token) {
        return error_response(401, "debug_session_token_invalid");
    }
    let turn = request
        .get("turn")
        .filter(|value| !value.is_null())
        .map(field_text);
    match action {
        "pause" => session.pause(),
        "release" => session.release(turn.as_deref()),
        "advance" => {
            let stage = request.get("stage").and_then(Value::as_u64);
            if !session.advance(turn.as_deref().unwrap_or(""), stage) {
                return error_response(409, "debug_stage_not_paused");
            }
        }
        _ => {}
    }
    json_response(
        200,
        &session.snapshot(request.get("since").and_then(Value::as_u64)),
    )
}
