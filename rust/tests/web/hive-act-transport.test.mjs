import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {stripVTControlCharacters} from 'node:util';
import {decodeActWorkflowLog, requireExactProgramOutput} from '../../../scripts/lib/hive-delivery-evidence.mjs';
import {adaptHiveEntrypoint} from '../../../scripts/prepare-hive-original-delivery.mjs';

const entrypoint = fs.readFileSync(new URL('../fixtures/hive-act-transport/entrypoint-8f22b87c.mjs', import.meta.url), 'utf8');
const oracleSource = fs.readFileSync(new URL('../fixtures/hive-act-transport/oracle-8f22b87c.mjs', import.meta.url), 'utf8');
const oracleStart = oracleSource.indexOf('export function logPrintsHelloWorld(');
const oracleEnd = oracleSource.indexOf('\n}\n', oracleStart) + 2;
assert.ok(oracleStart > 0 && oracleEnd > oracleStart);
const oracleBody = oracleSource.slice(oracleStart, oracleEnd).replace(/^export /u, '');
const expected = /export const HELLO_WORLD_OUTPUT = '([^']*)';/u.exec(oracleSource)[1];
const originalOracle = Function('stripVTControlCharacters', 'HELLO_WORLD_OUTPUT', 'return (' + oracleBody + ');')(stripVTControlCharacters, expected);
const event = msg => JSON.stringify({dryrun: false, raw_output: true, jobID: 'run', step: 'sh tests/verify-output.sh', msg});

test('authentic newline-bearing act messages pass the exact unchanged upstream line oracle', () => {
  const raw = event(expected + '\n') + '\n';
  assert.equal(originalOracle(raw), false, 'original transport mismatch reproduced');
  const decoded = decodeActWorkflowLog(raw);
  assert.equal(decoded, expected + '\n');
  assert.equal(originalOracle(decoded), true);
  requireExactProgramOutput({exitCode: 0, signal: null, stdoutBase64: Buffer.from(decoded).toString('base64'), stderrBase64: ''}, expected + '\n');
});
test('arbitrary output chunks retain exact Unicode and newline bytes', () => {
  const chunks = ['English\nРусский\n', 'हिन्दी\n中文\nEspañol\n'];
  assert.equal(decodeActWorkflowLog(chunks.map(event).join('\n')), chunks.join(''));
});
test('control messages, dry runs, wrong types and unframed text cannot become program output', () => {
  for (const changed of [{dryrun: false, msg: expected}, {dryrun: true, raw_output: true, msg: expected},
    {dryrun: false, raw_output: 'true', msg: expected}, {dryrun: false, raw_output: true, msg: {text: expected}}]) {
    const decoded = decodeActWorkflowLog(JSON.stringify(changed) + '\n' + expected + '\n{malformed');
    assert.equal(decoded, '');
    assert.equal(originalOracle(decoded), false);
  }
});
test('the reviewed adapter changes only its import and workflow-log transport binding', () => {
  const adapted = adaptHiveEntrypoint(entrypoint);
  const imported = "import { decodeActWorkflowLog } from './formal-ai-hive-delivery-evidence.mjs';\n";
  const reversed = adapted.replace(imported, '').replace('stdout: decodeActWorkflowLog(results.map(result => result.stdout).join(', 'stdout: results.map(result => result.stdout).join(')
    .replace(".join('\\n')) };", ".join('\\n') };");
  assert.equal(reversed, entrypoint, 'all original task/oracle wiring is conserved');
  assert.throws(() => adaptHiveEntrypoint(entrypoint + '\n'), /unreviewed Hive entrypoint bytes/u);
});
