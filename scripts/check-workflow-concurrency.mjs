#!/usr/bin/env node
// Qualify the concurrency.queue key that actionlint 1.7.12 cannot model.
// Keep its narrow unknown-key exception and all other schema/shell checks.
// Expressions are observations requiring runtime evaluation, not invalid literals.
import {readFileSync, readdirSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';
import YAML from 'yaml';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const expression = value => typeof value === 'string' && /\$\{\{\s*\S[\s\S]*?\}\}/u.test(value);
const mapping = value => value !== null && typeof value === 'object' && !Array.isArray(value);

export function checkSource(file, source) {
  const observations = [], problems = [];
  let workflow;
  try {
    const document = YAML.parseDocument(source);
    if (document.errors.length) throw new Error(document.errors.map(error => error.message).join('; '));
    workflow = document.toJS({maxAliasCount: 100});
    if (!mapping(workflow)) throw new Error('workflow must be a mapping');
  } catch (error) {
    return {observations, problems: [{file, scope: 'workflow', message: error.message}]};
  }
  const scopes = [['workflow', workflow.concurrency]];
  for (const [name, job] of Object.entries(mapping(workflow.jobs) ? workflow.jobs : {}))
    scopes.push(['job ' + name, job?.concurrency]);
  for (const [scope, concurrency] of scopes) {
    if (!mapping(concurrency) || !Object.hasOwn(concurrency, 'queue')) continue;
    const queue = concurrency.queue;
    const dynamicQueue = expression(queue);
    const cancellation = concurrency['cancel-in-progress'];
    const dynamicCancellation = expression(cancellation);
    const problem = message => problems.push({file, scope, message});
    if (typeof queue !== 'string' || (!dynamicQueue && !['max', 'single'].includes(queue))) {
      problem('concurrency.queue must be max, single, or an expression');
      continue;
    }
    if (queue === 'max' && (cancellation === true || cancellation === 'true'))
      problem('concurrency.queue max cannot be combined with cancel-in-progress true');
    observations.push({file, scope, queue,
      qualification: dynamicQueue || dynamicCancellation ? 'requires-runtime-evaluation' : 'literal'});
  }
  return {observations, problems};
}

export function checkRepository(root = ROOT) {
  const directory = join(root, '.github/workflows');
  const files = readdirSync(directory, {withFileTypes: true})
    .filter(entry => entry.isFile() && /\.ya?ml$/u.test(entry.name))
    .map(entry => '.github/workflows/' + entry.name).sort();
  const observations = [], problems = [];
  for (const file of files) {
    const result = checkSource(file, readFileSync(join(root, file), 'utf8'));
    observations.push(...result.observations);
    problems.push(...result.problems);
  }
  if (!files.length) problems.push({file: directory, scope: 'repository', message: 'no workflow sources checked'});
  return {files, observations, problems};
}

function main(arguments_) {
  if (arguments_.length > 1) {
    console.error('usage: node scripts/check-workflow-concurrency.mjs [ROOT]');
    return 2;
  }
  try {
    const result = checkRepository(arguments_.length ? resolve(arguments_[0]) : ROOT);
    for (const {file, scope, message} of result.problems)
      console.error(file + ': ' + scope + ': ' + message);
    if (result.problems.length) return 1;
    const unresolved = result.observations.filter(row => row.qualification === 'requires-runtime-evaluation').length;
    console.log('Workflow concurrency: ' + result.files.length + ' files, ' + result.observations.length +
      ' queue policies; ' + unresolved + ' require runtime evaluation.');
    return 0;
  } catch (error) {
    console.error('Workflow concurrency: ' + error.message);
    return 1;
  }
}
if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url)
  process.exitCode = main(process.argv.slice(2));
