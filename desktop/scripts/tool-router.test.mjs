import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  createToolRouter,
  isPermitted,
  SANDBOX_IMAGE,
  SUPPORTED_TOOLS,
} = require("../lib/tool-router.cjs");

function requestForTool(tool) {
  if (tool === "http_fetch" || tool === "url_navigate") {
    return { tool, input: { url: "https://example.com" } };
  }
  if (tool === "read_local_file") {
    return { tool, input: { path: "README.md" } };
  }
  return { tool, input: { command: "echo hi" } };
}

test("default-deny: an ungranted tool call is refused and nothing executes", async () => {
  let fetched = false;
  const router = createToolRouter({
    fetchImpl: async () => {
      fetched = true;
      return { status: 200, text: async () => "body" };
    },
  });
  const result = await router.invoke({ tool: "http_fetch", input: { url: "https://example.com" } });
  assert.equal(result.ok, false);
  assert.equal(result.status, "refused");
  assert.equal(result.executed, false);
  assert.equal(fetched, false, "fetch must not run when the tool is denied");
});

test("default-deny: empty grants refuse every supported tool before side effects", async () => {
  let effects = 0;
  const router = createToolRouter({
    fetchImpl: async () => {
      effects += 1;
      return { status: 200, text: async () => "body" };
    },
    readFile: async () => {
      effects += 1;
      return "body";
    },
    dockerAvailable: () => {
      effects += 1;
      return true;
    },
    runInSandbox: async () => {
      effects += 1;
      return { exitCode: 0, output: "body" };
    },
  });
  router.setGrants({});

  for (const tool of SUPPORTED_TOOLS) {
    const result = await router.invoke(requestForTool(tool));
    assert.equal(result.ok, false, `${tool} must be refused`);
    assert.equal(result.status, "refused", `${tool} status`);
    assert.equal(result.executed, false, `${tool} executed flag`);
  }
  assert.equal(effects, 0, "no fetch, file, docker, or sandbox effect may run");
});

test("with permission granted, http_fetch is served by the local process", async () => {
  const router = createToolRouter({
    fetchImpl: async (url) => ({ status: 200, text: async () => `fetched ${url}` }),
  });
  router.setGrants({ http_fetch: true });
  const result = await router.invoke({ tool: "http_fetch", input: { url: "https://example.com" } });
  assert.equal(result.ok, true);
  assert.equal(result.executed, true);
  assert.equal(result.servedBy, "local-process");
  assert.equal(result.httpStatus, 200);
  assert.match(result.body, /https:\/\/example\.com/);
});

test("an `all` grant opts every tool in at once", () => {
  assert.equal(isPermitted({ all: true }, "shell"), true);
  assert.equal(isPermitted({ all: false }, "shell"), false);
  assert.equal(isPermitted({ shell: true }, "shell"), true);
  assert.equal(isPermitted({}, "shell"), false);
  assert.equal(isPermitted(null, "shell"), false);
});

test("a partial grant map only permits the named tool", () => {
  const grants = { shell: true };
  for (const tool of SUPPORTED_TOOLS) {
    assert.equal(
      isPermitted(grants, tool),
      tool === "shell",
      `${tool} permission should be scoped to its own grant`,
    );
  }
  assert.equal(isPermitted({ shell: true, http_fetch: false }, "http_fetch"), false);
  assert.equal(isPermitted({ http_fetch: true, shell: false }, "shell"), false);
});

test("with permission granted, code_exec runs inside the box-dind container with logs captured", async () => {
  const calls = [];
  const router = createToolRouter({
    dockerAvailable: () => true,
    runInSandbox: async (spec) => {
      calls.push(spec);
      return { exitCode: 0, output: "hello from container", logPath: "/tmp/run.log" };
    },
  });
  router.setGrants({ code_exec: true });
  const result = await router.invoke({ tool: "code_exec", input: { command: "echo hi" } });
  assert.equal(result.ok, true);
  assert.equal(result.executed, true);
  assert.equal(result.servedBy, "box-dind");
  assert.equal(result.image, SANDBOX_IMAGE);
  assert.equal(result.logPath, "/tmp/run.log");
  assert.equal(calls.length, 1);
  assert.equal(calls[0].image, SANDBOX_IMAGE);
  assert.equal(calls[0].command, "echo hi");
});

test("code_exec gracefully refuses when Docker is unavailable (never runs unsandboxed)", async () => {
  let ran = false;
  const router = createToolRouter({
    dockerAvailable: () => false,
    runInSandbox: async () => {
      ran = true;
      return { exitCode: 0, output: "" };
    },
  });
  router.setGrants({ all: true });
  const result = await router.invoke({ tool: "code_exec", input: { command: "echo hi" } });
  assert.equal(result.ok, false);
  assert.equal(result.status, "sandbox_unavailable");
  assert.equal(result.executed, false);
  assert.equal(ran, false, "code must not run when the sandbox is unavailable");
});

test("read_local_file is confined to the allowed root", async () => {
  const router = createToolRouter({
    readFile: async () => "secret",
    allowedReadRoot: "/repo",
    resolvePath: (value) => (value.startsWith("/") ? value : `/repo/${value}`),
  });
  router.setGrants({ read_local_file: true });
  const outside = await router.invoke({ tool: "read_local_file", input: { path: "/etc/passwd" } });
  assert.equal(outside.ok, false);
  assert.equal(outside.status, "forbidden");

  const inside = await router.invoke({ tool: "read_local_file", input: { path: "README.md" } });
  assert.equal(inside.ok, true);
  assert.equal(inside.body, "secret");
});

test("read_local_file rejects a sibling directory that shares the root prefix", async () => {
  // `/repo-evil` starts with the string `/repo` but is not inside it; a raw
  // prefix check would wrongly allow it, a containment check must refuse.
  const router = createToolRouter({
    readFile: async () => "secret",
    allowedReadRoot: "/repo",
    resolvePath: (value) => (value.startsWith("/") ? value : `/repo/${value}`),
  });
  router.setGrants({ read_local_file: true });
  const sibling = await router.invoke({
    tool: "read_local_file",
    input: { path: "/repo-evil/secret" },
  });
  assert.equal(sibling.ok, false);
  assert.equal(sibling.status, "forbidden");
});

test("unknown tools are rejected without executing", async () => {
  const router = createToolRouter();
  router.setGrants({ all: true });
  const result = await router.invoke({ tool: "rm_rf", input: {} });
  assert.equal(result.ok, false);
  assert.equal(result.status, "unknown_tool");
  assert.equal(result.executed, false);
});
