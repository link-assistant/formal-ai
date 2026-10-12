// R1188-U2: dependencies point inward (scripts/check-dependency-direction.mjs).
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";

import {
  isSurface,
  numberOf,
  outwardReferences,
  portBypasses,
  rustModuleOf,
  valuesOf,
} from "../../../scripts/check-dependency-direction.mjs";

const SURFACES = ["cli_", "server", "telegram"];

test("a prefix ending in _ names every module it starts, a plain name only itself", () => {
  assert.equal(isSurface("cli_env", SURFACES), true);
  assert.equal(isSurface("server", SURFACES), true);
  assert.equal(isSurface("server_parity", SURFACES), false);
  assert.equal(isSurface("solver", SURFACES), false);
});

test("the top-level module of a path is its first segment", () => {
  assert.equal(rustModuleOf("rust/src/server/routes.rs"), "server");
  assert.equal(rustModuleOf("rust/src/cli_env.rs"), "cli_env");
});

test("solver code naming a surface reaches outward; a surface naming itself and comments do not", () => {
  const found = outwardReferences(
    [
      { path: "rust/src/solver.rs", text: "use crate::server::Api;\n// crate::cli_env::flag\nlet x = crate::seed::lexicon();" },
      { path: "rust/src/server/routes.rs", text: "use crate::server::Api;" },
      { path: "rust/src/agent.rs", text: "if crate::cli_env::flag_enabled(\"X\") {}" },
    ],
    SURFACES,
  );
  assert.deepEqual(found, [
    { path: "rust/src/solver.rs", line: 1, module: "server" },
    { path: "rust/src/agent.rs", line: 1, module: "cli_env" },
  ]);
});

test("only a listed adapter may import node: built-ins or the Node server", () => {
  const found = portBypasses(
    [
      { path: "js/agentic/node-host.mjs", text: "import fs from 'node:fs';" },
      { path: "js/agentic/planner.mjs", text: "import fs from 'node:fs';\nimport { x } from './host.mjs';" },
      { path: "js/agentic/solve.mjs", text: "import { solve } from '../server/solve.mjs';" },
    ],
    ["js/agentic/node-host.mjs"],
  );
  assert.deepEqual(found, [
    { path: "js/agentic/planner.mjs", line: 1, specifier: "node:fs" },
    { path: "js/agentic/solve.mjs", line: 1, specifier: "../server/solve.mjs" },
  ]);
});

test("the rules file is read by key", () => {
  const rules = 'x\n  surface-module "cli_"\n  surface-module "server"\n  outward-references-ceiling 3\n';
  assert.deepEqual(valuesOf(rules, "surface-module"), ["cli_", "server"]);
  assert.equal(numberOf(rules, "outward-references-ceiling"), 3);
});

test("the repository meets its ceilings", () => {
  execFileSync(process.execPath, ["scripts/check-dependency-direction.mjs"], { stdio: "pipe" });
});
