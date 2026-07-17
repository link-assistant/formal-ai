import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";

const require = createRequire(import.meta.url);
const {
  createToolRouter,
  isPermitted,
  SANDBOX_IMAGE,
  SUPPORTED_TOOLS,
  READ_ONLY_TOOLS,
} = require("../lib/tool-router.cjs");

function requestForTool(tool) {
  if (tool === "web_search") {
    return { tool, input: { query: "formal ai" } };
  }
  if (tool === "web_fetch") {
    return { tool, input: { url: "https://example.com/app" } };
  }
  if (tool === "http_fetch" || tool === "url_navigate") {
    return { tool, input: { url: "https://example.com" } };
  }
  if (tool === "read_local_file") {
    return { tool, input: { path: "README.md" } };
  }
  return { tool, input: { command: "echo hi" } };
}

test("default-deny: an ungranted write-capable tool is refused and nothing executes", async () => {
  let ran = false;
  const router = createToolRouter({
    runOnHost: async () => {
      ran = true;
      return { exitCode: 0, output: "body" };
    },
  });
  const result = await router.invoke({ tool: "shell", input: { command: "echo hi" } });
  assert.equal(result.ok, false);
  assert.equal(result.status, "refused");
  assert.equal(result.executed, false);
  assert.equal(ran, false, "shell must not run when the tool is denied");
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
    runOnHost: async () => {
      effects += 1;
      return { exitCode: 0, output: "body" };
    },
    runInSandbox: async () => {
      effects += 1;
      return { exitCode: 0, output: "body" };
    },
  });
  router.setGrants({});

  for (const tool of SUPPORTED_TOOLS.filter((name) => !router.isReadOnly(name))) {
    const result = await router.invoke(requestForTool(tool));
    assert.equal(result.ok, false, `${tool} must be refused`);
    assert.equal(result.status, "refused", `${tool} status`);
    assert.equal(result.executed, false, `${tool} executed flag`);
  }
  assert.equal(effects, 0, "no fetch, file, docker, or sandbox effect may run");
});

test("specialized common file tools provide read-only inspection and granted edits", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "formal-ai-router-"));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  await fs.mkdir(path.join(root, "src"));
  await fs.writeFile(path.join(root, "src", "one.txt"), "alpha\nbeta\n", "utf8");
  await fs.writeFile(path.join(root, "src", "two.js"), "const alpha = 1;\n", "utf8");
  const router = createToolRouter({
    readFile: (file) => fs.readFile(file, "utf8"),
    writeFile: (file, body) => fs.writeFile(file, body, "utf8"),
    readDirectory: (directory) => fs.readdir(directory, { withFileTypes: true }),
    allowedReadRoot: root,
    resolvePath: (value) => path.resolve(root, value),
  });

  assert.match((await router.invoke({ tool: "read_file", input: { path: "src/one.txt" } })).body, /alpha/);
  assert.match((await router.invoke({ tool: "grep", input: { path: "src", pattern: "alpha" } })).body, /one\.txt:1/);
  assert.deepEqual((await router.invoke({ tool: "glob", input: { path: "src", pattern: "*.js" } })).matches, ["two.js"]);
  assert.equal((await router.invoke({ tool: "write_file", input: { path: "new.txt", content: "new" } })).status, "refused");

  router.setGrants({ write_file: true, edit_file: true, multi_edit: true });
  assert.equal((await router.invoke({ tool: "write_file", input: { path: "new.txt", content: "new" } })).ok, true);
  assert.equal((await router.invoke({ tool: "edit_file", input: { path: "new.txt", old_string: "new", new_string: "edited" } })).body, "edited");
  assert.equal((await router.invoke({
    tool: "multi_edit",
    input: { path: "new.txt", edits: [{ old_string: "edit", new_string: "test" }, { old_string: "ed", new_string: "ing" }] },
  })).body, "testing");
});

test("web search and rendered fetch run with agent permission off", async () => {
  const calls = [];
  const router = createToolRouter({
    webSearch: async (input) => {
      calls.push(["search", input]);
      return { body: "search results", results: [{ title: "Result" }] };
    },
    webFetch: async (input) => {
      calls.push(["fetch", input]);
      return { body: "rendered body", url: input.url };
    },
  });
  router.setGrants({ all: false, web_search: false, web_fetch: false });

  const search = await router.invoke({ tool: "web_search", input: { query: "formal ai" } });
  const fetched = await router.invoke({
    tool: "web_fetch",
    input: { url: "https://example.com/app" },
  });

  assert.equal(search.ok, true);
  assert.equal(search.executed, true);
  assert.equal(search.servedBy, "web-capture");
  assert.equal(search.body, "search results");
  assert.equal(fetched.ok, true);
  assert.equal(fetched.body, "rendered body");
  assert.deepEqual(calls, [
    ["search", { query: "formal ai" }],
    ["fetch", { url: "https://example.com/app" }],
  ]);
});

test("common frontend aliases route by capability instead of tool name", async () => {
  const router = createToolRouter({
    webSearch: async () => ({ body: "aliased search" }),
    runOnHost: async () => ({ exitCode: 0, output: "aliased shell" }),
  });

  assert.equal((await router.invoke({ tool: "SEARCH_WEB", input: { query: "formal ai" } })).body, "aliased search");
  assert.equal((await router.invoke({ tool: "exec_command", input: { command: "pwd" } })).status, "refused");
  router.setGrants({ shell: true });
  assert.equal((await router.invoke({ tool: "exec_command", input: { command: "pwd" } })).body, "aliased shell");
});

test("permission-free http_fetch is served by the local process", async () => {
  const router = createToolRouter({
    fetchImpl: async (url) => ({ status: 200, text: async () => `fetched ${url}` }),
  });
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
      ["shell", "bash", "exec_command"].includes(tool),
      `${tool} permission should be scoped to its own grant`,
    );
  }
  assert.equal(isPermitted({ shell: true, http_fetch: false }, "http_fetch"), false);
  assert.equal(isPermitted({ http_fetch: true, shell: false }, "shell"), false);
});

test("with permission granted, shell runs on the host by default", async () => {
  const calls = [];
  const router = createToolRouter({
    dockerAvailable: () => {
      throw new Error("host shell must not probe Docker by default");
    },
    runOnHost: async (spec) => {
      calls.push(spec);
      return {
        exitCode: 0,
        output: "Desktop\nDocuments\n",
        stdout: "Desktop\nDocuments\n",
        stderr: "",
        logPath: "/tmp/host-shell.log",
      };
    },
    runInSandbox: async () => {
      throw new Error("host shell must not run in Docker by default");
    },
  });
  router.setGrants({ shell: true });

  const result = await router.invoke({ tool: "shell", input: { command: "ls ~" } });

  assert.equal(result.ok, true);
  assert.equal(result.executed, true);
  assert.equal(result.servedBy, "host-shell");
  assert.equal(result.isolation, "host");
  assert.equal(result.exitCode, 0);
  assert.equal(result.logPath, "/tmp/host-shell.log");
  assert.equal(result.stdout, "Desktop\nDocuments\n");
  assert.equal(result.stderr, "");
  assert.equal(result.body, "Desktop\nDocuments\n");
  assert.deepEqual(calls, [{ tool: "shell", command: "ls ~" }]);
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

test("shell can still opt into Docker isolation", async () => {
  const calls = [];
  let hostRan = false;
  const router = createToolRouter({
    dockerAvailable: () => true,
    runOnHost: async () => {
      hostRan = true;
      return { exitCode: 0, output: "" };
    },
    runInSandbox: async (spec) => {
      calls.push(spec);
      return { exitCode: 0, output: "container home\n", logPath: "/tmp/docker-shell.log" };
    },
  });
  router.setGrants({ shell: true });

  const result = await router.invoke({
    tool: "shell",
    input: { command: "ls ~", isolation: "docker" },
  });

  assert.equal(result.ok, true);
  assert.equal(result.executed, true);
  assert.equal(result.servedBy, "box-dind");
  assert.equal(result.isolation, "docker");
  assert.equal(result.image, SANDBOX_IMAGE);
  assert.equal(result.logPath, "/tmp/docker-shell.log");
  assert.equal(result.body, "container home\n");
  assert.equal(hostRan, false);
  assert.equal(calls.length, 1);
  assert.equal(calls[0].tool, "shell");
  assert.equal(calls[0].command, "ls ~");
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
