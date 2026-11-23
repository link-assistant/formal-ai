// Finite pure scheduling replay: no functional tests, builds or subprocesses.
import {readFileSync, readdirSync, writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import {join} from 'node:path';
import YAML from 'yaml';
import {parseTestDurations, mergeJavaScriptDurations} from '../../scripts/lib/ci-speed-durations.mjs';
import {durationLookup, planShards} from '../../scripts/lib/ci-speed-shards.mjs';

const [output] = process.argv.slice(2);
assert(output, 'provide a result directory');
const root = 'experiments/issue-1580/fixtures/';
const current = root + 'hosted-javascript-e46254ac/';
const prior = readFileSync(current + 'prior-scheduling-floor.lino', 'utf8');
const rows = readFileSync(current + 'durations.jsonl', 'utf8').trim().split('\n').map(JSON.parse);
const files = readdirSync('rust/tests/web/').filter(file => file.endsWith('.test.mjs')).map(file => 'rust/tests/web/' + file);
const count = YAML.parse(readFileSync('.github/workflows/layered-ci.yml', 'utf8')).jobs.js.strategy.matrix.shard.length;
const cohorts = readdirSync(root).filter(name => name.startsWith('hosted-javascript-')).map(name => {
  const identityPath = join(root, name, 'identity.json');
  let identity;
  try { identity = JSON.parse(readFileSync(identityPath, 'utf8')); }
  catch (error) { if (error.code !== 'ENOENT') throw error; identity = {}; }
  const observations = readFileSync(join(root, name, 'durations.jsonl'), 'utf8').trim().split('\n').map(JSON.parse);
  const measured = new Map(observations.map(row => [row.file, row.seconds]));
  if (name.endsWith('de79e6ec')) measured.delete('rust/tests/web/full-provenance-cli.test.mjs');
  return {name, identity, measured};
});
const evaluate = text => {
  const fallback = Number(text.match(/^ {2}default-seconds (\S+)$/mu)[1]);
  const {shards} = planShards(files, count, durationLookup(parseTestDurations(text), fallback));
  return cohorts.map(({name, identity, measured}) => {
    const cost = durationLookup(measured, identity.unobservedFallbackSeconds ?? 0.34);
    const loads = shards.map(shard => {
      const workers = Array(identity.modeledWorkers ?? 4).fill(0);
      for (const file of shard) {
        if (name.endsWith('de79e6ec') && !measured.has(file)) continue;
        workers[workers.indexOf(Math.min(...workers))] += cost(file);
      }
      return Math.max(...workers) + (identity.runnerOverheadAllowanceSeconds ?? 0);
    });
    return {name, maximumSeconds: Math.max(...loads), loads};
  });
};
const candidates = [];
const dialogueOwners = ['cnl-solver-dialogue', 'cnl-solver-dialogue-b', 'cnl-solver-dialogue-c']
  .map(name => 'rust/tests/web/' + name + '.test.mjs');
const dialogueAggregate = rows.filter(row => dialogueOwners.includes(row.file)).reduce((sum, row) => sum + row.seconds, 0);
assert.equal(rows.filter(row => dialogueOwners.includes(row.file)).length, dialogueOwners.length);
let selected = null;
for (let index = 0; index < 13; index++) {
  const factor = Number((1.65 + index * 0.05).toFixed(2));
  const reserved = new Map(rows.map(row => [row.file, {seconds: Math.ceil(row.seconds * factor * 10) / 10, complete: true}]));
  // Each split owner retains a conservative family bound so all three start
  // independently. This scheduling estimate is not an observed child duration.
  const dialogueBound = Math.ceil(dialogueAggregate * factor * 10) / 10;
  for (const file of dialogueOwners) reserved.set(file, {seconds: dialogueBound, complete: true});
  let text = mergeJavaScriptDurations(prior, reserved,
    {source: 45240466225, recordedDate: '2026-11-23', raiseOnly: true, floor: 0.401});
  for (const file of dialogueOwners) {
    const marker = '  test "' + file + '"\n    seconds ' + dialogueBound + '\n';
    const extra = file === dialogueOwners[0]
      ? '    estimate-kind observed-split-owner-aggregate-reserve\n    source-run 45240466225\n'
      : '    bound-from "' + dialogueOwners[0] + '"\n';
    assert(text.includes(marker));
    text = text.replace(marker, marker + extra);
  }
  const results = evaluate(text);
  const weights = parseTestDurations(text);
  const fallback = Number(text.match(/^ {2}default-seconds (\S+)$/mu)[1]);
  const {shards} = planShards(files, count, durationLookup(weights, fallback));
  const owners = ['cnl-solver-dialogue', 'cnl-solver-dialogue-b', 'cnl-solver-dialogue-c']
    .map(name => 'rust/tests/web/' + name + '.test.mjs');
  const ownerShards = owners.map(file => shards.findIndex(shard => shard.includes(file)) + 1);
  const distinctDialogueOwners = new Set(ownerShards).size === owners.length;
  const fits = distinctDialogueOwners && results.every(row => row.maximumSeconds <= 270);
  candidates.push({factor, fits, distinctDialogueOwners, ownerShards, results});
  if (fits && !selected) selected = {factor, text};
}
const report = {candidateLimit: 13, candidates, baseline: evaluate(prior), selectedFactor: selected?.factor ?? null};
writeFileSync(join(output, 'reserve-probe.json'), JSON.stringify(report, null, 2) + '\n');
assert(selected, 'no bounded candidate fits every retained cohort');
writeFileSync(join(output, 'proposed-scheduling-floor.lino'),
  '# Scheduling weights retain prior floors and reserve ' + Math.round((selected.factor - 1) * 100)
  + '% over run 45240466225.\n# Complete nine-job observations and the unchanged runner allowance are retained in\n'
  + '# experiments/issue-1580/fixtures/hosted-javascript-e46254ac/.\n' + selected.text);
console.log(JSON.stringify({selectedFactor: selected.factor,
  candidates: candidates.map(row => ({factor: row.factor, fits: row.fits, maximumSeconds: Math.max(...row.results.map(value => value.maximumSeconds))})),
  baseline: report.baseline.find(row => row.name.endsWith('e46254ac'))}));
