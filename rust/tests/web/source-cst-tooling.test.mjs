import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import { sourceCstFrontends } from '../../../scripts/lib/source-cst-frontends.mjs';
import { createSourceCstPacket, restoreSourceCstPacket } from '../../../scripts/lib/source-cst-packets.mjs';
import { main, ownedProductionSources } from '../../../scripts/check-source-networks.mjs';
import {workflowPin} from '../../../scripts/translate-js-rust.mjs';
test('complete grammar accepts CommonJS return and typed CTS without discarding source layout', async () => {
  const runtime = await sourceCstFrontends();
  try {
    for (const [file, language, source] of [['packages/example/index.cjs', 'JavaScript', '// unchanged CommonJS wrapper\nreturn require("node:path");\n'], ['packages/example/index.cts', 'TypeScript', '/* typed CommonJS */\nconst value: number = 1; module.exports = value;\n']]) {
      const packet = createSourceCstPacket({
        path: file,
        language,
        source
      }, runtime.forSource(file, language));
      assert.equal(restoreSourceCstPacket(packet.receipt, packet.compressed).source, source);
    }
  } finally {
    runtime.close();
  }
});
test('actual source producer covers tracked tooling and packages, runs without SDK, and marks changed tooling uncommitted', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'owned-cst-tooling-')),
    previous = process.env.FORMAL_AI_META_LANGUAGE;
  const git = (...args) => execFileSync('git', ['-C', root, ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe']
  }).trim();
  const put = (file, source) => {
    fs.mkdirSync(path.dirname(path.join(root, file)), {
      recursive: true
    });
    fs.writeFileSync(path.join(root, file), source);
  };
  try {
    git('init', '-q');
    git('config', 'user.name', 'Fixture');
    git('config', 'user.email', 'fixture@example.invalid');
    const programs = new Map([['js/example.mjs', 'export const x = 1;\n'], ['ts/example.ts', 'export const x: number = 1;\n'], ['rust/src/lib.rs', 'pub fn value() -> i32 { 1 }\n'], ['rust/build.rs', 'fn main() {}\n'], ['scripts/tool.mjs', '// tool comment\nexport const operation = "derive";\n'], ['packages/example/index.cjs', '/* package comment */\nreturn 1;\n'], ['packages/example/index.cts', 'const value: number = 1; module.exports = value;\n']]);
    for (const [file, source] of programs) put(file, source);
    put('js/vendor/external.js', 'invalid vendor payload');
    put('rust/tests/fixture.rs', 'deliberately invalid fixture');
    git('add', '.');
    git('commit', '-qm', 'Owned source fixture');
    const head = git('rev-parse', 'HEAD');
    put('scripts/untracked.mjs', 'untracked source is not owned');
    assert.deepEqual(ownedProductionSources(root).sort(), [...programs.keys()].sort());
    process.env.FORMAL_AI_META_LANGUAGE = path.join(root, 'absent-sdk');
    const run = output => main(['--source-root', root, '--upstream-commit', workflowPin(fs.readFileSync(new URL('../../../.github/workflows/layered-ci.yml', import.meta.url), 'utf8')), '--shard-count', '1', '--shard-index', '0', '--write', '--output', path.join(root, output)]);
    const report = await run('clean-cst');
    assert.equal(report.sourceHead, head);
    assert.equal(report.inputState, 'committed-head');
    assert.equal(report.checkedSources, programs.size);
    for (const receipt of report.sources) {
      const packet = fs.readFileSync(path.join(root, 'clean-cst', receipt.packet));
      assert.equal(restoreSourceCstPacket(receipt, packet).source, programs.get(receipt.path));
    }
    assert.equal(fs.existsSync(process.env.FORMAL_AI_META_LANGUAGE), false, 'producer must not provision an unused SDK');
    put('scripts/tool.mjs', programs.get('scripts/tool.mjs') + '// changed tool\n');
    const dirty = await run('changed-tool-cst');
    assert.equal(dirty.sourceHead, null);
    assert.equal(dirty.inputState, 'working-tree');
    assert.equal(dirty.baseHead, head);
  } finally {
    if (previous === undefined) delete process.env.FORMAL_AI_META_LANGUAGE;else process.env.FORMAL_AI_META_LANGUAGE = previous;
    fs.rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
