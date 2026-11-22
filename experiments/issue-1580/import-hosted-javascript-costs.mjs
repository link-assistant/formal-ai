// Re-import the complete hosted measurements and explicitly inherited scheduling bounds.
// This does not manufacture measurements for the three newly separated scenarios.
import {readFileSync, writeFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {mergeJavaScriptDurations} from '../../scripts/lib/ci-speed-durations.mjs';

const fixture = new URL('./fixtures/hosted-javascript-de79e6ec/', import.meta.url);
const observations = readFileSync(new URL('durations.jsonl', fixture), 'utf8').trim().split('\n').map(line => JSON.parse(line));
const identity = JSON.parse(readFileSync(new URL('identity.json', fixture), 'utf8'));
const partition = JSON.parse(readFileSync(new URL('scenario-partition.json', fixture), 'utf8'));
if (observations.length !== identity.completeObservations || observations.some(row => !row.complete)
  || new Set(observations.map(row => row.file)).size !== observations.length) throw Error('IncompleteHostedCostObservations');
const observed = new Map(observations.map(({file, seconds, complete}) => [file, {seconds, complete}]));
const parent = 'rust/tests/web/full-provenance-cli.test.mjs';
const bound = observed.get(parent).seconds;
if (bound !== partition.originalObservedFileSeconds || partition.sourceRun !== identity.run) throw Error('ParentScenarioObservationMismatch');
const path = fileURLToPath(new URL('../../data/meta/javascript-test-durations.lino', import.meta.url));
const before = readFileSync(path, 'utf8');
const options = {source: String(identity.run), recordedDate: '2026-11-22', raiseOnly: true};
let after = mergeJavaScriptDurations(before, observed, options);
for (const {path: child} of partition.scenarios) {
  const escaped = child.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&');
  const pattern = new RegExp('^  test "' + escaped + '"\\n((?:    [^\\n]*\\n)*)', 'mu');
  const previous = after.match(pattern);
  const previousSeconds = Number(previous?.[1].match(/^    seconds (\S+)$/mu)?.[1] ?? 0);
  const metadata = child === parent
    ? '    estimate-kind parent-scenario-aggregate-upper-bound\n    source-run ' + identity.run + '\n'
    : '    bound-from "' + parent + '"\n';
  const record = '  test "' + child + '"\n    seconds ' + Math.max(bound, previousSeconds) + '\n' + metadata;
  after = previous ? after.replace(pattern, () => record) : after + record;
}
after = mergeJavaScriptDurations(after, observed, options);
if (process.argv.includes('--write')) writeFileSync(path, after);
else if (before !== after) { console.error('Hosted JavaScript cost import is stale; run this script with --write.'); process.exitCode = 1; }
else console.log('Hosted costs and explicitly inherited parent bounds are current.');
