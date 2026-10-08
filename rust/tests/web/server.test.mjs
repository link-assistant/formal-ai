// The JavaScript server (js/server/, R1013): every route in
// data/meta/server-routes.lino answers on a real socket with the status,
// content type and envelope the Rust server gives, and the shared rules
// (bearer auth, MCP origin, CORS, deprecation, SSE framing, unsupported
// models, malformed bodies) hold. scripts/check-server-parity.mjs compares
// the two servers byte for byte; this suite keeps the JavaScript side honest
// on its own, with no Rust binary.

import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, before, test } from "node:test";

import { startServer } from "../../../js/server/main.mjs";
import { concretePaths, matchRoute, serverRoutes } from "../../../js/server/routes.mjs";
import { toCompactJson, toPrettyJson, f64, sortedKeys } from "../../../js/server/json.mjs";
import { stableId } from "../../../js/server/ids.mjs";
import { serverMessage } from "../../../js/server/messages.mjs";

const TOKEN = "server-test-token";
const PROMPT = "What is 2 + 2?";
const chatBody = (extra = {}) =>
  JSON.stringify({ model: "formal-ai", messages: [{ role: "user", content: PROMPT }], ...extra });
const geminiBody = JSON.stringify({ contents: [{ role: "user", parts: [{ text: PROMPT }] }] });

// One probe per route: the body it is sent and the answer it must give.
const PROBES = {
  options_preflight: { status: 204, type: "application/json", empty: true },
  head_probe: { status: 200, type: "application/json", empty: true },
  head_probe_vendor: { status: 200, type: "application/json", empty: true },
  api_hello: { status: 200, type: "application/json", json: (body) => assert.equal(body.message, "hello") },
  health: { status: 200, type: "application/json", json: (body) => assert.equal(body.status, "ok") },
  models: { status: 200, type: "application/json", json: (body) => assert.equal(body.object, "list") },
  network: { status: 200, type: "application/json", json: (body) => assert.ok(Array.isArray(body.nodes)) },
  network_graph_alias: { status: 200, type: "application/json", json: (body) => assert.ok(Array.isArray(body.nodes)) },
  bundle: { status: 200, type: "text/plain", text: (body) => assert.ok(body.startsWith("formal_ai_seed_bundle\n")) },
  links: { status: 200, type: "text/plain" },
  links_query: { status: 200, type: "text/plain", body: JSON.stringify({ query: "MATCH (a)-[r]->(b) RETURN a, r, b" }) },
  memory: { status: 200, type: "text/plain", text: (body) => assert.ok(body.startsWith("demo_memory")) },
  memory_since: { status: 200, type: "text/plain", text: (body) => assert.ok(body.startsWith("demo_memory")) },
  memory_import: {
    status: 200,
    type: "application/json",
    body: 'demo_memory\n  event "server_test_event"\n    role "user"\n    content "Server test"\n',
    json: (body) => assert.equal(body.object, "memory.import"),
  },
  anthropic_messages: {
    status: 200,
    type: "application/json",
    body: JSON.stringify({ model: "formal-ai", max_tokens: 64, messages: [{ role: "user", content: PROMPT }] }),
    json: (body) => {
      assert.equal(body.type, "message");
      assert.equal(body.role, "assistant");
    },
  },
  chat_completions: {
    status: 200,
    type: "application/json",
    body: chatBody(),
    json: (body) => {
      assert.equal(body.object, "chat.completion");
      assert.equal(body.id, stableId("chatcmpl", PROMPT));
      assert.equal(body.choices[0].message.role, "assistant");
      assert.match(body.choices[0].message.content, /4/);
    },
  },
  responses: {
    status: 200,
    type: "application/json",
    body: JSON.stringify({ model: "formal-ai", input: PROMPT }),
    json: (body) => assert.equal(body.object, "response"),
  },
  mcp: {
    status: 200,
    type: "application/json",
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/list" }),
    json: (body) => assert.equal(body.jsonrpc, "2.0"),
  },
  mcp_stream: { status: 405, type: "application/json", error: "mcp_sse_unsupported" },
  telegram_webhook: { status: 200, type: "application/json", body: JSON.stringify({ update_id: 1 }) },
  conversation_context: { status: 404, type: "application/json" },
  conversation_learn: { status: 404, type: "application/json" },
  debug_session: { status: 404, type: "application/json", error: "debug_session_disabled" },
  gemini_models: { status: 200, type: "application/json", json: (body) => assert.ok(Array.isArray(body.models)) },
  gemini_model: { status: 200, type: "application/json", json: (body) => assert.equal(body.name, "models/formal-ai") },
  gemini_generate_content: { status: 200, type: "application/json", body: geminiBody, json: (body) => assert.ok(body.candidates) },
  gemini_stream_generate_content: { status: 200, type: "text/event-stream", body: geminiBody },
  vertex_models: { status: 200, type: "application/json" },
  vertex_generate_content: { status: 200, type: "application/json", body: geminiBody },
  vertex_stream_generate_content: { status: 200, type: "text/event-stream", body: geminiBody },
};

let server;
let base;
let home;

before(async () => {
  home = mkdtempSync(path.join(os.tmpdir(), "formal-ai-js-server-"));
  const env = {
    ...process.env,
    HOME: home,
    FORMAL_AI_API_BEARER_TOKEN: TOKEN,
    FORMAL_AI_MEMORY_PATH: path.join(home, "memory.lino"),
    FORMAL_AI_DIALOG_LOG_DIR: path.join(home, "dialogs"),
    FORMAL_AI_RECORD_CHAT: "0",
  };
  ({ server, url: base } = await startServer({ port: 0, env }));
});

after(() => {
  server?.close();
  if (home) rmSync(home, { recursive: true, force: true });
});

async function call(method, requestPath, { body = null, headers = {}, auth = true } = {}) {
  const init = { method, headers: { ...headers } };
  if (auth) init.headers.authorization = `Bearer ${TOKEN}`;
  if (body !== null) {
    init.body = body;
    init.headers["content-type"] = init.headers["content-type"] || "application/json";
  }
  const response = await fetch(`${base}${requestPath}`, init);
  return { status: response.status, headers: response.headers, body: await response.text() };
}

test("the route manifest names a handler-backed route for every row", () => {
  const routes = serverRoutes();
  assert.ok(routes.length >= 29, "the manifest carries the whole Rust surface");
  for (const route of routes) {
    assert.ok(PROBES[route.id], `${route.id} has a probe in this suite`);
    for (const concrete of concretePaths(route)) {
      assert.equal(matchRoute(route.method, concrete)?.route.id, route.id, `${route.method} ${concrete}`);
    }
  }
  assert.equal(matchRoute("GET", "/api/gemini/v1beta/models/x:generateContent"), null, "a model action is not metadata");
  assert.equal(matchRoute("POST", "/v1/conversations/x"), null, "only /learn is a conversation POST");
});

test("every manifest route answers with its expected status and content type", async () => {
  for (const route of serverRoutes()) {
    const probe = PROBES[route.id];
    for (const concrete of concretePaths(route)) {
      const response = await call(route.method, concrete, { body: probe.body ?? (route.method === "POST" ? "{}" : null) });
      const label = `${route.method} ${concrete}`;
      assert.equal(response.status, probe.status, `${label}: ${response.body.slice(0, 200)}`);
      assert.equal(response.headers.get("content-type"), probe.type, label);
      assert.equal(response.headers.get("access-control-allow-origin"), "*", label);
      assert.equal(response.headers.get("connection"), "close", label);
      if (probe.empty) assert.equal(response.body, "", label);
      if (probe.json) probe.json(JSON.parse(response.body));
      if (probe.text) probe.text(response.body);
      if (probe.error) assert.equal(JSON.parse(response.body).error.message, serverMessage(probe.error), label);
      assert.equal(response.headers.get("deprecation"), route.deprecated ? "true" : null, label);
    }
  }
});

test("bearer routes refuse a missing or wrong token and accept an API key header", async () => {
  const missing = await call("GET", "/v1/models", { auth: false });
  assert.equal(missing.status, 401);
  assert.equal(missing.body, toCompactJson({ error: { message: serverMessage("bearer_token_invalid"), type: "formal_ai_error" } }));
  const wrong = await call("GET", "/v1/models", { auth: false, headers: { authorization: "Bearer nope" } });
  assert.equal(wrong.status, 401);
  for (const header of ["x-api-key", "x-goog-api-key", "anthropic-api-key"]) {
    const keyed = await call("GET", "/v1/models", { auth: false, headers: { [header]: TOKEN } });
    assert.equal(keyed.status, 200, header);
  }
  const open = await call("GET", "/health", { auth: false });
  assert.equal(open.status, 200, "health needs no token");
  const unknown = await call("GET", "/v1/server-test-missing", { auth: false });
  assert.equal(unknown.status, 401, "the bearer gate precedes routing, as natively");
});

test("unknown routes, foreign MCP origins and unsupported models answer with the Rust errors", async () => {
  const missing = await call("GET", "/server-test-missing", { auth: false });
  assert.equal(missing.status, 404);
  assert.equal(JSON.parse(missing.body).error.message, serverMessage("route_not_found"));
  const foreign = await call("POST", "/mcp", {
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/list" }),
    headers: { origin: "http://elsewhere.invalid" },
  });
  assert.equal(foreign.status, 403);
  assert.equal(JSON.parse(foreign.body).error.message, serverMessage("mcp_origin_not_allowed"));
  const unsupported = await call("POST", "/v1/chat/completions", {
    body: JSON.stringify({ model: "gpt-unknown", messages: [{ role: "user", content: PROMPT }] }),
  });
  assert.equal(unsupported.status, 400);
  assert.equal(
    JSON.parse(unsupported.body).error.message,
    serverMessage("unsupported_model", { model: "gpt-unknown", canonical: "formal-ai" }),
  );
  const malformed = await call("POST", "/v1/chat/completions", { body: "{not json" });
  assert.equal(malformed.status, 400);
  assert.match(JSON.parse(malformed.body).error.message, /^invalid chat request: /);
});

test("chat completions stream as chat.completion.chunk frames ending in [DONE]", async () => {
  const response = await call("POST", "/v1/chat/completions", {
    body: chatBody({ stream: true, stream_options: { include_usage: true } }),
  });
  assert.equal(response.status, 200);
  assert.equal(response.headers.get("content-type"), "text/event-stream");
  const frames = response.body.split("\n\n").filter(Boolean);
  assert.equal(frames.at(-1), "data: [DONE]");
  const chunks = frames.slice(0, -1).map((frame) => JSON.parse(frame.replace(/^data: /, "")));
  assert.deepEqual(Object.keys(chunks[0]), ["choices", "created", "id", "model", "object"], "json! keys sort");
  assert.deepEqual(chunks[0].choices[0].delta, { role: "assistant" });
  assert.equal(chunks.at(-2).choices[0].finish_reason, "stop");
  assert.deepEqual(chunks.at(-1).choices, []);
  assert.ok(chunks.at(-1).usage.total_tokens > 0);
});

test("a tool-bearing request without agent mode is refused by policy", async () => {
  const response = await call("POST", "/v1/chat/completions", {
    body: chatBody({ tools: [{ type: "function", function: { name: "bash", parameters: {} } }] }),
  });
  const message = JSON.parse(response.body).choices[0].message;
  assert.equal(message.content, serverMessage("tool_call_refused"));
  assert.equal(message.thinking_steps[0].step, "policy_refusal");
});

test("the JSON renderer prints the way serde_json does", () => {
  assert.equal(toPrettyJson({ a: [], b: {}, c: [1, f64(0)], d: "é\n" }), '{\n  "a": [],\n  "b": {},\n  "c": [\n    1,\n    0.0\n  ],\n  "d": "é\\n"\n}');
  assert.equal(toCompactJson(sortedKeys({ z: 1, a: { y: null, b: true } })), '{"a":{"b":true,"y":null},"z":1}');
  assert.equal(stableId("chatcmpl", PROMPT), "chatcmpl_b888e028816d6739", "the Rust FNV-1a id");
});
