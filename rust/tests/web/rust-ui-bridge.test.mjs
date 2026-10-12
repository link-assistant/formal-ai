// Issue #951: the UI migration boundary never substitutes a JavaScript
// renderer -- a missing transport, an unknown operation, or an operation the
// compiled core does not expose all refuse instead of guessing.
import assert from "node:assert/strict";
import { test } from "node:test";

import { createRustUiBridge, RUST_UI_OPERATIONS } from "../../../js/app/rust-ui-bridge.js";

test("a bridge without a compiled transport is refused", () => {
  assert.throws(() => createRustUiBridge(null), /transport required/);
  assert.throws(() => createRustUiBridge({}), /transport required/);
});

test("known operations reach the transport; unknown or unsupported ones are refused", async () => {
  const seen = [];
  const bridge = createRustUiBridge({
    supports: (operation) => operation === RUST_UI_OPERATIONS.evidenceSlug,
    call: async (operation, facts) => {
      seen.push([operation, facts]);
      return "slug";
    },
  });
  assert.equal(await bridge.call(RUST_UI_OPERATIONS.evidenceSlug, { title: "x" }), "slug");
  assert.deepEqual(seen, [[RUST_UI_OPERATIONS.evidenceSlug, { title: "x" }]]);
  await assert.rejects(bridge.call("ui.render_anything", {}), /Unknown UI core operation/);
  await assert.rejects(bridge.call(RUST_UI_OPERATIONS.issueReport, {}), /does not expose ui\.render_issue_report/);
  const noSupports = createRustUiBridge({ call: async () => "x" });
  await assert.rejects(noSupports.call(RUST_UI_OPERATIONS.issueReport, {}), /does not expose/);
});
