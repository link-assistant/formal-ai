// The step-through debug session of the JavaScript server (issue #667, R383,
// js/server/debug-session.mjs; protocol in docs/vscode/debugger.md): the
// `--debug-session` flag binds loopback only, every debug request needs the
// session token, a paused turn advances one stage per authenticated advance
// (a repeated or stale advance answers 409 and records nothing), and release
// or a client disconnect never leaves a solve paused. The Rust twin is
// rust/tests/unit/specification/debug_session.rs.

import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, before, describe, test } from "node:test";

import { DebugSession, debugSessionBanner, debugTokenFrom, isLoopbackHost } from "../../../js/server/debug-session.mjs";
import { parseArgs, startServer } from "../../../js/server/main.mjs";

const TOKEN = "debug-session-test-token";
const STAGES = [
  { step: "impulse", detail: "What is 2 + 2?", source_event: "impulse" },
  { step: "compute", detail: "4", source_event: "calculation" },
  { step: "deformalize", detail: "4", source_event: "response" },
];

const kinds = (session) => session.events.map((event) => `${event.kind}:${event.stage}`);

describe("the session state machine (DebugSession)", () => {
  test("only loopback hosts may carry a debug session", () => {
    for (const host of ["127.0.0.1", "127.1.2.3", "localhost", "LOCALHOST", "::1", "[::1]"]) {
      assert.equal(isLoopbackHost(host), true, host);
    }
    for (const host of ["0.0.0.0", "192.168.1.5", "::", "example.com", "128.0.0.1", "127.0.0.256", ""]) {
      assert.equal(isLoopbackHost(host), false, host);
    }
  });

  test("the token comes from the launcher, else it is generated and printed once", () => {
    const given = debugTokenFrom({ FORMAL_AI_DEBUG_SESSION_TOKEN: ` ${TOKEN} ` });
    assert.deepEqual(given, { token: TOKEN, generated: false });
    const session = new DebugSession(given);
    assert.equal(debugSessionBanner(session), `debug_session=${session.id}`);
    const generated = debugTokenFrom({});
    assert.equal(generated.generated, true);
    assert.match(generated.token, /^[0-9a-f]{48}$/);
    const fresh = new DebugSession(generated);
    assert.equal(debugSessionBanner(fresh), `debug_session=${fresh.id};token=${generated.token}`);
    assert.ok(fresh.authorizes(generated.token));
    assert.ok(!fresh.authorizes(TOKEN));
    assert.ok(!fresh.authorizes(undefined));
  });

  test("stepping off never holds a turn", async () => {
    const session = new DebugSession(TOKEN);
    await session.gate(STAGES);
    assert.deepEqual(session.events, []);
  });

  test("each advance reveals exactly one stage and a repeated advance records nothing", async () => {
    const session = new DebugSession(TOKEN);
    session.pause();
    let finished = false;
    const held = session.gate(STAGES).then(() => {
      finished = true;
    });
    assert.deepEqual(kinds(session), ["stage_paused:0"]);
    assert.equal(session.advance("turn_1", 1), false, "stage 1 is not the paused stage");
    assert.equal(session.advance("turn_9", 0), false, "an unknown turn");
    assert.equal(session.advance("turn_1", 0), true);
    assert.equal(session.advance("turn_1", 0), false, "stage 0 was already advanced");
    assert.deepEqual(kinds(session), ["stage_paused:0", "stage_advanced:0", "stage_paused:1"]);
    assert.equal(session.advance("turn_1", 1), true);
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(finished, false, "the turn is still held at its last stage");
    assert.equal(session.advance("turn_1", 2), true);
    await held;
    assert.deepEqual(kinds(session), [
      "stage_paused:0", "stage_advanced:0", "stage_paused:1", "stage_advanced:1",
      "stage_paused:2", "stage_advanced:2", "turn_released:2",
    ]);
    const last = session.events.at(-1);
    assert.equal(last.reason, "completed");
    assert.equal(last.session, session.id);
    assert.equal(last.turn, "turn_1");
    assert.equal(session.advance("turn_1", 2), false, "a released turn never advances");
    const paused = session.events[2];
    assert.deepEqual(paused, {
      detail: "4", id: "debug_event_3", kind: "stage_paused", session: session.id,
      source: "calculation", stage: 1, stages: 3, step: "compute", turn: "turn_1",
    });
    assert.deepEqual(session.snapshot(6).events.map((event) => event.id), ["debug_event_7"]);
  });

  test("a disconnect or a release frees a paused turn", async () => {
    const session = new DebugSession(TOKEN);
    session.pause();
    const disconnect = new AbortController();
    const first = session.gate(STAGES, disconnect.signal);
    disconnect.abort();
    await first;
    assert.equal(session.events.at(-1).reason, "disconnected");
    const second = session.gate(STAGES);
    assert.equal(session.snapshot().paused.length, 1);
    session.release();
    await second;
    assert.equal(session.events.at(-1).reason, "released");
    assert.equal(session.stepping, false, "a release without a turn stops stepping");
    assert.deepEqual(session.snapshot().paused, []);
  });
});

describe("the --debug-session server", () => {
  let server;
  let base;
  let home;

  before(async () => {
    home = mkdtempSync(path.join(os.tmpdir(), "formal-ai-debug-session-"));
    const env = {
      ...process.env,
      HOME: home,
      FORMAL_AI_MEMORY_PATH: path.join(home, "memory.lino"),
      FORMAL_AI_DIALOG_LOG_DIR: path.join(home, "dialogs"),
      FORMAL_AI_RECORD_CHAT: "0",
      FORMAL_AI_DEBUG_SESSION_TOKEN: TOKEN,
    };
    delete env.FORMAL_AI_API_BEARER_TOKEN;
    delete env.FORMAL_AI_HTTP_BEARER_TOKEN;
    delete env.FORMAL_AI_API_TOKEN;
    ({ server, url: base } = await startServer({ port: 0, env, debugSession: true }));
  });

  after(() => {
    server?.close();
    if (home) rmSync(home, { recursive: true, force: true });
  });

  async function debug(action, body) {
    const response = await fetch(`${base}/v1/debug/${action}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    return { status: response.status, body: JSON.parse(await response.text()) };
  }

  function chat(signal) {
    return fetch(`${base}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ model: "formal-ai", messages: [{ role: "user", content: "What is 2 + 2?" }] }),
      signal,
    });
  }

  async function pausedTurn() {
    for (let attempt = 0; attempt < 600; attempt += 1) {
      const { body } = await debug("session", { token: TOKEN });
      if (body.paused.length) return body.paused[0];
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    throw new Error("no turn paused");
  }

  test("the flag parses, and a non-loopback bind is refused", async () => {
    assert.equal(parseArgs(["serve", "--debug-session"], {}).debugSession, true);
    assert.equal(parseArgs(["serve"], {}).debugSession, false);
    await assert.rejects(startServer({ host: "0.0.0.0", port: 0, debugSession: true }), /^Error: debug_session_requires_loopback:0\.0\.0\.0$/);
  });

  test("every debug request needs the session token", async () => {
    for (const action of ["session", "pause", "advance", "release"]) {
      const refused = await debug(action, { token: "wrong" });
      assert.equal(refused.status, 401, action);
      assert.equal(refused.body.error.message, "debug_session_token_invalid");
    }
    const malformed = await fetch(`${base}/v1/debug/session`, { method: "POST", body: "{not json" });
    assert.equal(malformed.status, 400);
    const state = await debug("session", { token: TOKEN });
    assert.equal(state.status, 200);
    assert.equal(state.body.object, "debug.session");
    assert.equal(state.body.stepping, false);
  });

  test("a paused chat turn answers only after its last stage is advanced", async () => {
    assert.equal((await debug("pause", { token: TOKEN })).body.stepping, true);
    const before = (await debug("session", { token: TOKEN })).body.next;
    let answered = false;
    const pending = chat().then(async (response) => {
      answered = true;
      return JSON.parse(await response.text());
    });
    let paused = await pausedTurn();
    const total = paused.stages;
    assert.ok(total >= 2, `the turn has stages (${total})`);
    for (let stage = 0; stage < total; stage += 1) {
      assert.equal(paused.stage, stage);
      assert.equal(answered, false, `held before stage ${stage} is advanced`);
      const advanced = await debug("advance", { token: TOKEN, turn: paused.turn, stage });
      assert.equal(advanced.status, 200);
      const repeated = await debug("advance", { token: TOKEN, turn: paused.turn, stage });
      assert.equal(repeated.status, 409, "a repeated advance never records a stage twice");
      assert.equal(repeated.body.error.message, "debug_stage_not_paused");
      if (stage + 1 < total) paused = advanced.body.paused[0];
    }
    const completion = await pending;
    assert.match(completion.choices[0].message.content, /4/);
    const events = (await debug("session", { token: TOKEN, since: before })).body.events;
    const advancedStages = events.filter((event) => event.kind === "stage_advanced").map((event) => event.stage);
    assert.deepEqual(advancedStages, [...Array(total).keys()]);
    assert.equal(events.at(-1).kind, "turn_released");
    assert.equal(events.at(-1).reason, "completed");
  });

  test("a disconnected client and a release never leave a solve paused", async () => {
    await debug("pause", { token: TOKEN });
    const abort = new AbortController();
    const dropped = chat(abort.signal).catch((error) => error);
    await pausedTurn();
    abort.abort();
    await dropped;
    for (let attempt = 0; attempt < 100; attempt += 1) {
      const state = (await debug("session", { token: TOKEN })).body;
      if (!state.paused.length) break;
      await new Promise((resolve) => setTimeout(resolve, 20));
    }
    let state = (await debug("session", { token: TOKEN })).body;
    assert.deepEqual(state.paused, []);
    assert.equal(state.events.at(-1).reason, "disconnected");

    const pending = chat();
    await pausedTurn();
    state = (await debug("release", { token: TOKEN })).body;
    assert.equal(state.stepping, false);
    assert.deepEqual(state.paused, []);
    assert.equal(state.events.at(-1).reason, "released");
    assert.equal((await pending).status, 200);
  });
});
