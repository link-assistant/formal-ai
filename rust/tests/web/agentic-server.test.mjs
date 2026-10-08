// The agentic planner on the JavaScript server (R1015): every agent-mode
// request of the server parity corpus (rust/tests/fixtures/server-parity/
// requests.lino, `agent_mode true`) answers on a real socket with the tool
// call or final answer the Rust server gives. The expectations are the ones
// the Rust suites assert - rust/tests/integration/issue_749_shell_routing.rs,
// rust/tests/unit/agentic_surfaces.rs, rust/src/protocol.rs's gates - and,
// where those suites do not pin a value, the Rust server's own answer.

import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, before, test } from "node:test";

import { startServer } from "../../../js/server/main.mjs";
import { loadCorpus } from "../../../scripts/check-server-parity.mjs";

const REPO = path.resolve(import.meta.dirname, "../../..");
const corpus = loadCorpus(readFileSync(path.join(REPO, "rust/tests/fixtures/server-parity/requests.lino"), "utf8"));
const requests = corpus.requests.filter((request) => request.agentMode);

const shell = (command) => ({ tool: "run_shell_command", args: { command, is_background: false } });
const cmd = (value) => ({ tool: "exec_command", args: { cmd: value } });

/** Expected first tool call (`tool`, parsed `args`) or final answer prefix (`final`). */
const EXPECTED = {
  agent_shell_show_current_directory: shell("pwd"),
  agent_shell_show_environment: shell("env"),
  agent_shell_copy_file: shell("test -e a.txt"),
  agent_shell_git_changes: shell("git diff"),
  // A content search is the workspace-search arm's grep, which lists file:line
  // hits (PR #1188 T90).
  agent_shell_search_todo: shell("grep -rnHw --exclude-dir=.git -- 'TODO' '.'"),
  agent_shell_delete_en: shell("rm old.txt"),
  agent_shell_delete_ru: shell("rm old.txt"),
  agent_shell_delete_hi: shell("rm old.txt"),
  agent_shell_delete_zh: shell("rm old.txt"),
  agent_shell_opencode_quotes: shell("echo ISSUE749_OPENCODE_TWO_WORDS SECOND_ARGUMENT"),
  agent_shell_delete_step_two: { final: "The command completed successfully without output." },
  agent_shell_pwd_done: { final: "The `pwd` command completed. Output:" },
  agent_responses_cmd_date: cmd("date"),
  agent_responses_cmd_echo: cmd("echo hi there"),
  agent_responses_cmd_sort: cmd("sort names.txt"),
  agent_responses_cmd_bash_c: cmd("bash -c 'printf hello'"),
  agent_responses_cmd_powershell: cmd("powershell Get-ChildItem"),
  agent_responses_shell_cmd: { tool: "shell", args: { cmd: "ls" } },
  agent_responses_bash_command: { tool: "bash", args: { command: "ls" } },
  agent_responses_formalize: { tool: "web_search", args: { query: "Пушкин Сказка о рыбаке и рыбке полный текст" } },
  agent_responses_formalize_fetch: {
    tool: "web_fetch",
    args: { format: "text", url: "https://ru.wikisource.org/wiki/Сказка_о_рыбаке_и_рыбке_(Пушкин)" },
  },
  agent_responses_formalize_final: { final: "Formalized «Сказка о рыбаке и рыбке» into a Links Notation knowledge base" },
  agent_messages_formalize: { tool: "web_search", args: { query: "Пушкин Сказка о рыбаке и рыбке полный текст" } },
  agent_messages_shell: { tool: "bash", args: { command: "pwd" } },
  agent_gemini_shell: { tool: "run_shell_command", args: { command: "pwd" } },
  agent_tool_choice_without_tools: { final: "Tool calls are not allowed for `tool:*`: no installed associative package grants tool:*." },
  agent_non_agentic_task: { final: "2 + 2 = 4" },
  agent_no_tools_chat: { final: "Hi, how may I help you?" },
  agent_opencode_read: { tool: "read", args: { filePath: "alpha.txt" } },
  agent_opencode_search: { tool: "websearch", args: { query: "the web for the latest rust release" } },
  agent_opencode_repeated: { final: "The `pwd` command completed. Output:" },
};

/** The first tool call or the final text of any protocol's answer. */
function decision(body) {
  if (body.choices) {
    const message = body.choices[0].message;
    const call = message.tool_calls?.[0];
    return call ? { tool: call.function.name, args: JSON.parse(call.function.arguments) } : { final: message.content };
  }
  if (body.output) {
    const call = body.output.find((item) => item.type !== "message");
    if (call) return { tool: call.name, args: JSON.parse(call.arguments) };
    return { final: body.output[0].content[0].text };
  }
  if (body.content) {
    const use = body.content.find((block) => block.type === "tool_use");
    return use ? { tool: use.name, args: use.input } : { final: body.content.map((block) => block.text).join("") };
  }
  const parts = body.candidates[0].content.parts;
  const call = parts.find((part) => part.functionCall);
  return call ? { tool: call.functionCall.name, args: call.functionCall.args } : { final: parts.map((part) => part.text).join("") };
}

let server;
let base;
let home;

before(async () => {
  home = mkdtempSync(path.join(os.tmpdir(), "formal-ai-agentic-server-"));
  const env = {
    ...process.env,
    HOME: home,
    FORMAL_AI_API_BEARER_TOKEN: corpus.token,
    FORMAL_AI_MEMORY_PATH: path.join(home, "memory.lino"),
    FORMAL_AI_DIALOG_LOG_DIR: path.join(home, "dialogs"),
    FORMAL_AI_RECORD_CHAT: "0",
    FORMAL_AI_DREAMING: "0",
  };
  ({ server, url: base } = await startServer({ port: 0, agentMode: true, env }));
});

after(() => {
  server?.close();
  rmSync(home, { recursive: true, force: true });
});

test("the corpus carries agent-mode requests", () => {
  assert.ok(requests.length >= 30, `${requests.length} agent-mode requests`);
});

for (const request of requests) {
  const expected = EXPECTED[request.id];
  if (!expected) continue;
  test(`agent mode: ${request.id}`, async () => {
    const response = await fetch(base + request.path, {
      method: request.method,
      headers: { authorization: `Bearer ${corpus.token}`, "content-type": "application/json" },
      body: request.body,
    });
    assert.equal(response.status, 200);
    const got = decision(await response.json());
    if (expected.final !== undefined) {
      assert.equal(got.final?.startsWith(expected.final), true, `${request.id}: ${JSON.stringify(got)}`);
    } else {
      assert.deepEqual(got, expected, request.id);
    }
  });
}
