import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, readFileSync, writeFileSync, existsSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import YAML from 'yaml';
import {boundedCommand} from '../../../js/server/bounded-command.mjs';

test('the actual RML install and launcher work without executing package lifecycle scripts', {timeout: 30000}, async t => {
  const workflow = YAML.parse(readFileSync(process.env.FORMAL_AI_RML_WORKFLOW ?? '.github/workflows/layered-ci.yml', 'utf8'));
  const step = Object.values(workflow.jobs).flatMap(job => job.steps ?? [])
    .find(row => row.name === 'Install rml and name it in RML_BIN (R1186-6)');
  assert(step);
  const dir = mkdtempSync(join(tmpdir(), 'formal-ai-rml-policy-'));
  t.after(() => rmSync(dir, {recursive: true, force: true}));
  const prefix = join(dir, '.rml/js');
  mkdirSync(join(prefix, 'src'), {recursive: true});
  const metadata = {name: 'offline-rml-policy-fixture', version: '1.0.0', private: true,
    scripts: {postinstall: 'node install-sentinel.cjs'}};
  // No registry dependencies or network access: only the installer policy is under test.
  writeFileSync(join(prefix, 'package.json'), JSON.stringify(metadata));
  writeFileSync(join(prefix, 'install-sentinel.cjs'), "require('fs').writeFileSync('lifecycle-ran', 'yes');");
  writeFileSync(join(prefix, 'package-lock.json'), JSON.stringify({name: metadata.name, version: metadata.version,
    lockfileVersion: 3, requires: true, packages: {'': {...metadata, hasInstallScript: true}}}));
  writeFileSync(join(prefix, 'src/rml-links.mjs'), "console.log('RML launcher reached with ' + process.argv[2]);\n");
  const environment = join(dir, 'github-env'); writeFileSync(environment, '');
  const result = await boundedCommand('bash', ['-c', step.run], {
    cwd: dir, timeout: 20000, maxBuffer: 128 * 1024,
    env: {...process.env, RUNNER_TEMP: dir, GITHUB_ENV: environment,
      npm_config_offline: 'true', npm_config_cache: join(dir, 'npm-cache'), NODE_OPTIONS: ''},
  });
  assert.equal(result.error, undefined, result.stderr?.toString());
  assert.equal(result.status, 0, result.stderr?.toString());
  assert.equal(result.cleanup_complete, true);
  assert.equal(existsSync(join(prefix, 'lifecycle-ran')), false, 'no install-time code is needed for the source exporter');
  assert.doesNotMatch(result.stderr.toString(), /install-scripts|allowScripts/u);
  assert.equal(readFileSync(environment, 'utf8'), 'RML_BIN=' + join(dir, 'rml') + '\n');
  const launched = await boundedCommand(join(dir, 'rml'), ['export-probe'], {cwd: dir, timeout: 10000, maxBuffer: 16384});
  assert.equal(launched.status, 0, launched.stderr?.toString());
  assert.equal(launched.stdout.toString(), 'RML launcher reached with export-probe\n');
});
