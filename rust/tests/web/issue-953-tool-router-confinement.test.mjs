// Issue #953 (architecture review 8.4): desktop tool-router confinement is
// spec data shared with the Rust core, not hand-rolled JavaScript. The
// adversarial cases here — symlinks, "..", UNC prefixes, reserved Windows
// segments, archive entries — must all be denied with the reason strings
// from desktop/lib/tool-router-confinement.lino, and the minified web
// bundles must not be tracked.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

import { REPO_ROOT } from "./support/browser-runtime.mjs";

const require = createRequire(path.join(REPO_ROOT, "desktop/main.cjs"));
const confinement = require("./lib/confinement.cjs");
const { createToolRouter } = require("./lib/tool-router.cjs");

const SPEC_PATH = path.join(REPO_ROOT, "desktop/lib/tool-router-confinement.lino");
const ROOT = path.join(REPO_ROOT, "desktop-fixture-root");
const WORKSPACE = path.join(ROOT, "computer-use");

function router() {
  return createToolRouter({
    allowedReadRoot: ROOT,
    computerUseRoot: WORKSPACE,
    resolvePath: (value) => path.resolve(ROOT, String(value || "")),
  });
}

test("the spec parses with rules, fields, and deny reasons", () => {
  const rules = confinement.loadSpec(readFileSync(SPEC_PATH, "utf8"));
  for (const name of [
    "read_root",
    "computer_use_workspace",
    "archive_entry",
    "windows_hardening",
    "dot_segments",
  ]) {
    assert.ok(rules[name], `rule ${name} exists`);
    assert.ok(Object.keys(rules[name].deny).length > 0, `rule ${name} carries denials`);
  }
  assert.equal(
    rules.read_root.deny.escapes_root,
    "path is outside the allowed root",
  );
  assert.equal(
    rules.windows_hardening.deny.unc_prefix,
    "UNC paths are not addressable inside the confinement root",
  );
});

test("read-root confinement denies traversal, absolutes, .., and UNC", () => {
  const spec = confinement.loadSpec(readFileSync(SPEC_PATH, "utf8"));
  const inside = path.join(ROOT, "notes", "a.txt");
  assert.equal(confinement.confineToRoot(spec, ROOT, inside).requested, inside);

  assert.match(
    confinement.confineToRoot(spec, ROOT, path.join(ROOT, "..", "escape.txt")).error,
    /outside the allowed root|\.\. segments/,
  );
  assert.match(
    confinement.confineToRoot(spec, ROOT, "/etc/passwd").error,
    /outside the allowed root/,
  );
  assert.match(
    confinement.confineToRoot(spec, ROOT, "\\\\server\\share\\escape.txt").error,
    /UNC paths are not addressable/,
  );
  assert.match(
    confinement.confineToRoot(spec, ROOT, path.join(ROOT, "trailing.", "file.txt")).error,
    /dots or spaces/,
  );
});

test("a symlink pointing outside the root is a sandbox-escape denial", async () => {
  const spec = confinement.loadSpec(readFileSync(SPEC_PATH, "utf8"));
  const outsideRoot = path.join(path.dirname(ROOT), "outside-target.txt");
  const checked = await confinement.confineToRootResolved(
    spec,
    ROOT,
    path.join(ROOT, "innocent.txt"),
    async () => outsideRoot, // realpath resolves the symlinked target
  );
  assert.match(checked.error, /symlink .*resolves outside|symlink_escape/);
});

test("plan workspaces confine: plan_id grammar, relative paths, escapes", () => {
  const spec = confinement.loadSpec(readFileSync(SPEC_PATH, "utf8"));
  const root = path.join(ROOT, "computer-use");
  const ok = confinement.confineToWorkspace(spec, root, { plan_id: "plan-01" }, "out.txt");
  assert.ok(ok.requested);

  for (const planId of ["..", ".", "a b", "../plan-01", "x".repeat(129), ""]) {
    const checked = confinement.confineToWorkspace(spec, root, { plan_id: planId }, "out.txt");
    assert.ok(checked.error, `plan_id ${JSON.stringify(planId)} must be refused`);
  }
  for (const value of ["", "/etc/passwd", "../escape.txt", "a/../../escape.txt"]) {
    const checked = confinement.confineToWorkspace(spec, root, { plan_id: "plan-01" }, value);
    assert.ok(checked.error, `path ${JSON.stringify(value)} must be refused`);
  }
});

test("archive entries cannot escape the extraction directory", () => {
  const spec = confinement.loadSpec(readFileSync(SPEC_PATH, "utf8"));
  const root = path.join(ROOT, "computer-use");
  const ok = confinement.confineArchiveEntry(
    spec, root, { plan_id: "plan-01" }, "out", "member.txt",
  );
  assert.ok(ok.requested);

  for (const entry of ["../member.txt", "/abs/member.txt", "a/../../member.txt"]) {
    const checked = confinement.confineArchiveEntry(
      spec, root, { plan_id: "plan-01" }, "out", entry,
    );
    assert.ok(checked.error, `entry ${JSON.stringify(entry)} must be refused`);
  }
});

test("the tool router applies the same spec through its own helpers", () => {
  const tools = router();
  assert.match(
    tools.safePath("../escape.txt").error,
    /outside the allowed root|\.\. segments/,
  );
  assert.match(
    tools.computerSafePath({ plan_id: "../plan" }, "x.txt").error,
    /plan_id is not safe/,
  );
  assert.ok(tools.computerSafePath({ plan_id: "plan-01" }, "x.txt").requested);
});

test("the engine authorizer is authoritative when it answers", async () => {
  const decisions = [];
  const tools = createToolRouter({
    allowedReadRoot: ROOT,
    computerUseRoot: WORKSPACE,
    resolvePath: (value) => path.resolve(ROOT, String(value || "")),
    readFile: () => Promise.resolve(""),
    engineAuthorize: async (payload) => {
      decisions.push(payload);
      return { available: true, decision: "refuse", reason: "denied by the Rust core" };
    },
  });
  tools.setGrants({ write_file: true });
  const result = await tools.invoke({ tool: "write_file", input: { path: "ok.txt", body: "x" } });
  assert.equal(result.status, "refused");
  assert.match(result.reason, /denied by the Rust core/);
  assert.equal(decisions.length, 1);
  assert.equal(decisions[0].canonicalTool, "write_file");
  assert.equal(decisions[0].grants.write_file, true);
});

test("minified web bundles are build output, not tracked sources", () => {
  const listed = spawnSync(
    "git",
    ["ls-files", "js/app.js", "js/vendor.bundle.js", "js/ocr.bundle.js", "js/web-search-component.bundle.js"],
    { cwd: REPO_ROOT, encoding: "utf8" },
  );
  assert.equal(listed.status, 0, listed.stderr);
  assert.equal(listed.stdout.trim(), "", "no minified bundle may remain tracked");
  const ignored = readFileSync(path.join(REPO_ROOT, ".gitignore"), "utf8");
  for (const bundle of ["js/app.js", "js/vendor.bundle.js", "js/ocr.bundle.js", "js/web-search-component.bundle.js"]) {
    assert.ok(ignored.includes(bundle), `${bundle} is gitignored`);
  }
});
