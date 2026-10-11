import assert from 'node:assert/strict';
import { test } from 'node:test';
import { portBypasses } from '../../../scripts/check-dependency-direction.mjs';

const sourcePath = 'js/agentic/arbitrary.mjs';
const scan = text => portBypasses([{ path: sourcePath, text }], []);

for (const [name, text, expected] of [
  ['named import', "import fs from 'node:fs';", ['node:fs']],
  ['side effect import', "import 'node:fs';", ['node:fs']],
  ['require call', "require('node:child_process');", ['node:child_process']],
  ['dynamic import', "import('node:path');", ['node:path']],
  ['multiline call', "require(\n 'node:fs'\n);", ['node:fs']],
  ['two operations', "require('node:fs'); import('node:path');", ['node:fs', 'node:path']],
  ['server import', "import { x } from '../server/main.mjs';", ['../server/main.mjs']],
  ['double quoted program', '"require(\'node:child_process\');"', []],
  ['single quoted program', "'require(\"node:fs\");'", []],
  ['template program', '`require("node:fs");`', []],
  ['line comment', '// require("node:fs");', []],
  ['block comment', '/* import("node:path"); */', []],
  ['template interpolation', '`data ${require("node:fs")} data`', ['node:fs']],
  ['nested template interpolation', '`data ${`nested ${import("node:path")}`}`', ['node:path']],
  ['quoted generated program and real call', '"require(\'node:fs\')";\nimport("node:path")', ['node:path']],
]) {
  test('structural dependency scanner: ' + name, () => {
    assert.deepEqual(scan(text).map(result => result.specifier), expected);
  });
}

test('source locations remain physical across multiline payloads', () => {
  const text = '`payload\nrequire("node:fs")`;\nrequire("node:path");';
  assert.deepEqual(scan(text), [{ path: sourcePath, line: 3, specifier: 'node:path' }]);
});

test('existing declared adapters retain exactly their existing treatment', () => {
  assert.deepEqual(portBypasses([{ path: sourcePath, text: 'require("node:fs")' }], [sourcePath]), []);
});
