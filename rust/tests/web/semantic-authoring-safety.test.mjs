import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { before, test } from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { installNodeHost } from "../../../js/agentic/node-host.mjs";
import { composeGeneralChangePlan } from "../../../js/agentic/general_planner.mjs";
import {
  planChatStepResolved,
  planChatStep,
} from "../../../js/agentic/planner.mjs";
import { finalResult } from "../../../js/agentic/final_result.mjs";
import { isSourceLinksTask } from "../../../js/agentic/source_links.mjs";
import { drive } from "../../../experiments/js_dogfood/drive.mjs";
let realm;
before(async () => {
  realm = await installNodeHost(new WorkerHost());
});
for (const request of [
  "Implement a resolver in output.mjs with meaningful tests.",
  "Implement a resolver in output.mjs with meaningful tests. Note «meaningful tests.» elsewhere.",
  "Implement a helper in folder/方法-α.mjs with source-backed symlinks initially.",
  "Implement a helper in output.mjs. Do not invent source links.",
])
  test(
    "semantic goals stay unsupported without overwriting prose " + request,
    async () => {
      assert.equal(composeGeneralChangePlan(request), null);
      assert.equal(realm.obligationPlanMode(request), null);
      const plan = await planChatStepResolved(
        [{ role: "user", content: request }],
        ["read", "write", "bash"],
      );
      assert.equal(plan.kind, "final");
      const result = finalResult(plan);
      assert.equal(result.disposition, "gap");
      assert.equal(result.origin, "semantic-authoring-missing-contract");
      assert.equal(result.discovery.authored, false);
      assert.equal(result.discovery.verified, false);
      assert.equal(result.discovery.goal, request);
    },
  );
const literals = [
  ["Create output.mjs with exactly this content: hello", "hello"],
  ["Implement a resolver in output.mjs with «const x = 1;».", "const x = 1;"],
  [
    "Implement a resolver in output.mjs with exactly this content: hello",
    "hello",
  ],
  ["Note İK𐐷 😀 café.\nCreate file x.txt containing «hello».", "hello"],
  [
    "Create rows.txt with content «  seed\n    row true\n».",
    "  seed\n    row true\n",
  ],
  ["Create punctuation.txt containing exactly: symbols #!?", "symbols #!?"],
];
for (const [request, expected] of literals)
  test("worker/root literal bytes agree " + request, () => {
    const plan = composeGeneralChangePlan(request);
    assert.equal(plan.mode, "literal_file");
    assert.equal(plan.content, expected);
    assert.equal(realm.obligationPlanMode(request), plan.mode);
  });
const positive = [
  "Translate the entire source code of our system to the links / meta language and back to source, and record the whole-repository source-to-links projection in Links Notation so we can recompile ourselves.",
  "recompile yourself: project the whole source graph to links",
  "translate the entire source of the system to links and back",
  "source links",
  "Create a source graph report.",
  "Translate all source to links and back without overwriting source.",
  "Пересобери исходный код.",
  "पुनः संकलित करो स्रोत।",
  "重新编译源码。",
  "Recompila el código fuente.",
];
for (const request of positive)
  test("positive owned recipe survives " + request, () =>
    assert.equal(isSourceLinksTask(request), true),
  );
const negative = [
  "Initially inspect source-backed symlinks.",
  "Inspect callback links in all-source.mjs.",
  "Read source-links.mjs.",
  "Read tools/source-links/manifest.txt.",
  "Implement a resolver in output.mjs using source links.",
  "Implement source links support in output.mjs.",
  "Create file x.txt containing «source links».",
  "Create file x.txt containing «Translate all source to links and back».",
  "Translate all source. Links and back are discussed separately.",
  "Do not invent fetch timestamps, source links or topic-specific records.",
  "Never recompile the source.",
  "Не создавай source links.",
  "不要生成 source links。",
  "No crees source links.",
  "source linksHelper",
  "RecompileHelper reads a source graph.",
  "Explain what recompile means.",
];
for (const request of negative)
  test(
    "unowned quote/path/negative/identifier cue refuses recipe " + request,
    () => assert.equal(isSourceLinksTask(request), false),
  );
test("negation elsewhere does not erase real positive instruction", () =>
  assert.equal(
    isSourceLinksTask(
      "Do not modify data. Translate all source to links and back.",
    ),
    true,
  ));
test("a source read is independent of semantic implementation", async () => {
  const plan = await planChatStepResolved(
    [{ role: "user", content: "Read proposal.json" }],
    ["read", "write", "bash"],
  );
  assert.equal(plan.kind, "tool_calls");
  assert.ok(plan.calls.every((call) => !["write", "edit"].includes(call.tool)));
});
test("raw literal source physically preserves semicolon and quoted cues", async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "semantic-literal-"));
  try {
    const content = "const x = 1;\n// source links; Task: Repair something\n";
    const out = await drive(
      planChatStep,
      directory,
      "Create output.mjs with exactly this content «" + content + "».",
      { steps: 16 },
    );
    assert.equal(
      fs.readFileSync(path.join(directory, "output.mjs"), "utf8"),
      content,
    );
    assert.equal(out.stop, "final");
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
