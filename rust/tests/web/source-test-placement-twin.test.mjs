// Issue #1191: main's `src_has_no_inline_unit_tests`
// (rust/tests/unit/ci-cd/source_test_placement.rs) in the fast PR lane, through
// scripts/check-source-test-placement.mjs, without compiling Rust.
//
// Recorded regression: PR #1483 left an inline `#[cfg(test)] mod tests` in
// rust/src/agentic_coding/work_item_leads.rs; its pull request was green and
// main then failed that test. The replay rebuilds that file shape in a scratch
// crate and requires the same five lines the native test reported.

import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  checkCrate, externalTestRegistrations, inlineTestViolations, isInlineTestMarker,
} from '../../../scripts/check-source-test-placement.mjs';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const SCRIPT = join(ROOT, 'scripts/check-source-test-placement.mjs');

function withCrate(body) {
  const root = mkdtempSync(join(tmpdir(), 'source-test-placement-'));
  try {
    mkdirSync(join(root, 'src/nested'), { recursive: true });
    mkdirSync(join(root, 'tests'), { recursive: true });
    return body(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test('the crate under rust/src carries no inline unit tests', () => {
  const result = spawnSync(process.execPath, [SCRIPT], { encoding: 'utf8', timeout: 120_000 });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /^source test placement: \d+ files under src\/ carry no inline unit tests\n$/u);
});

test('replay of the main-only failure: an inline mod tests in work_item_leads.rs fails', () => {
  withCrate(root => {
    mkdirSync(join(root, 'src/agentic_coding'));
    const lines = Array.from({ length: 310 }, (_, index) => `const LINE_${index}: u8 = 0;`);
    lines.push('#[cfg(test)]', 'mod tests {', '    use super::*;', '');
    for (const name of ['leads', 'symbols', 'matches']) lines.push('    #[test]', `    fn ${name}() {}`, '');
    lines.push('}', '');
    writeFileSync(join(root, 'src/agentic_coding/work_item_leads.rs'), lines.join('\n'));
    assert.deepEqual(checkCrate(root).violations.map(violation => violation.split(': ')[0]), [
      'src/agentic_coding/work_item_leads.rs:311',
      'src/agentic_coding/work_item_leads.rs:312',
      'src/agentic_coding/work_item_leads.rs:315',
      'src/agentic_coding/work_item_leads.rs:318',
      'src/agentic_coding/work_item_leads.rs:321',
    ]);
    const result = spawnSync(process.execPath, [SCRIPT, root], { encoding: 'utf8', timeout: 60_000 });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /must not contain inline tests/u);
    // The fix: the same tests behind an external registration pass.
    writeFileSync(join(root, 'tests/work_item_leads.rs'), '#[test]\nfn leads() {}\n');
    writeFileSync(join(root, 'src/agentic_coding/work_item_leads.rs'),
      'pub fn f() {}\n\n#[cfg(test)]\n#[path = "../../tests/work_item_leads.rs"]\nmod tests;\n');
    assert.deepEqual(checkCrate(root).violations, []);
  });
});

test('the moved work_item_leads tests are registered from the module they test', () => {
  const source = readFileSync(join(ROOT, 'rust/src/agentic_coding/work_item_leads.rs'), 'utf8');
  const registrations = externalTestRegistrations(join(ROOT, 'rust'), join(ROOT, 'rust/src/agentic_coding/work_item_leads.rs'), source);
  assert.deepEqual(registrations.map(({ path }) => path.slice(path.indexOf('/rust/') + 1)),
    ['rust/tests/unit/agentic-coding/work_item_leads.rs']);
});

test('the attribute detector matches the Rust is_inline_test_marker', () => {
  for (const line of ['#[cfg(test)]', '#[cfg(test)] fn f() {}', 'mod tests {', 'mod tests;', '#[test]',
    '#[ test]', '#[test(skip)]', '#[tokio::test]', '#[tokio::test(flavor = "multi_thread")]']) {
    assert.equal(isInlineTestMarker(line), true, line);
  }
  // `#[cfg(test, ...)]` is not a marker in the Rust detector either (it needs `#[cfg(test)`).
  for (const line of ['#[cfg(test, feature = "x")]', '"#[test]\\n"', '// #[test]', '#[cfg(not(test))]', 'mod testing;', '#[testing]', '#[derive(Debug)]']) {
    assert.equal(isInlineTestMarker(line), false, line);
  }
});

// Ported case for case from external_registration_is_structural_and_refuses_unproven_paths_or_bodies.
test('external registration is structural and refuses unproven paths or bodies', () => {
  withCrate(root => {
    const sourcePath = join(root, 'src/nested/module.rs');
    const fixture = '#[test]\nfn physical_assertion() { assert_eq!(2 + 2, 4); }\n';
    for (const name of ['renamed-alpha.rs', 'renamed-beta.rs']) {
      writeFileSync(join(root, 'tests', name), fixture);
      const declaration = `#[cfg(test)]\n#[path = "../../tests/${name}"]\nmod arbitrary_name;`;
      const registrations = externalTestRegistrations(root, sourcePath, declaration);
      assert.equal(registrations.length, 1, 'renamed physical registration');
      assert.equal(readFileSync(registrations[0].path, 'utf8'), fixture);
      assert.deepEqual(inlineTestViolations(root, sourcePath, declaration), []);
      const contextual = `const TEXT: &str = r###"{ #[cfg(test)] }"###;\n/* { nested /* } */ } */\n${declaration}`;
      assert.equal(externalTestRegistrations(root, sourcePath, contextual).length, 1);
    }
    symlinkSync(join(root, 'tests/renamed-alpha.rs'), join(root, 'tests/linked.rs'));
    for (const source of [
      '#[cfg(test)]\nfn inline() {}',
      '#[cfg(test)]\n#[path = "../../tests/renamed-alpha.rs"]\nmod inline {}',
      '#[unknown]\n#[cfg(test)]\n#[path = "../../tests/renamed-alpha.rs"]\nmod hidden;',
      '#[cfg(test)]\n#[unknown]\n#[path = "../../tests/renamed-alpha.rs"]\nmod hidden;',
      'mod nested {\n#[cfg(test)]\n#[path = "../../tests/renamed-alpha.rs"]\nmod hidden;\n}',
      '#[cfg(test)]\n#[path = "../../tests/missing.rs"]\nmod absent;',
      '#[cfg(test)]\n#[path = "../../../escaped.rs"]\nmod escaped;',
      '#[cfg(test)]\n#[path = "../../tests/../tests/renamed-alpha.rs"]\nmod redundant;',
      '#[cfg(test)]\n#[path = "../../tests/./renamed-alpha.rs"]\nmod redundant;',
      '#[cfg(test)]\n#[path = "../../tests/renamed-alpha.rs"]\nmod incomplete',
      '#[cfg(test)]\n#[path = "../../tests/renamed-alpha.rs"]\nmod hidden;\n}',
      '#[cfg(test)]\n#[path = "../../tests/linked.rs"]\nmod linked;',
      '#[test]\nfn actual_inline_test() {}',
      '#[tokio::test]\nasync fn actual_inline_test() {}',
      'mod tests { fn body() {} }',
    ]) {
      assert.deepEqual(externalTestRegistrations(root, sourcePath, source), [], `must refuse ${source}`);
      assert.notDeepEqual(inlineTestViolations(root, sourcePath, source), [], `original detector must reject ${source}`);
    }
  });
});
