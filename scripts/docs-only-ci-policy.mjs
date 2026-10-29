// Explicit documentation paths: unknown files and mixed changes still run CI.
import {readdirSync, readFileSync, writeFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
import YAML from 'yaml';
import {documentationPaths, lightweightWorkflows} from './lib/documentation-paths.mjs';
export {documentationPaths, lightweightWorkflows} from './lib/documentation-paths.mjs';

/** Edit only event filters; retained jobs, manual controls and schedules are untouched. */
export function filteredWorkflow(source) {
  const document = YAML.parseDocument(source);
  const edits = [];
  for (const event of ['push', 'pull_request']) {
    if (!document.hasIn(['on', event])) continue;
    const include = document.getIn(['on', event, 'paths']);
    const key = include ? 'paths' : 'paths-ignore';
    const sequence = document.getIn(['on', event, key]);
    const previous = sequence?.toJSON() ?? [];
    const additions = documentationPaths.map(path => include ? '!' + path : path).filter(path => !previous.includes(path));
    if (!additions.length) continue;
    if (sequence) {
      assert.ok(Array.isArray(previous) && previous.length, 'expected a nonempty path sequence');
      if (sequence.type === 'FLOW_SEQ' || sequence.flow) {
        const position = source.indexOf(']', sequence.items.at(-1).range[1]);
        assert.ok(position >= 0);
        edits.push([position, ', ' + additions.map(path => JSON.stringify(path)).join(', ')]);
      } else {
        const last = sequence.items.at(-1);
        const position = source.indexOf('\n', last.range[1] - 1) + 1;
        const lineStart = source.lastIndexOf('\n', sequence.range[0] - 1) + 1;
        const indent = source.slice(lineStart, sequence.range[0]);
        edits.push([position, additions.map(path => indent + '- ' + JSON.stringify(path) + '\n').join('')]);
      }
    } else {
      const eventNode = document.get('on', true).items.find(pair => pair.key.value === event);
      const position = source.indexOf('\n', eventNode.key.range[1]) + 1;
      assert.ok(position > 0 && !source.slice(eventNode.key.range[1],position).includes('{'), 'expected block-style event');
      edits.push([position, '    ' + key + ':\n' + additions.map(path => '      - ' + JSON.stringify(path) + '\n').join('')]);
    }
  }
  // Insertions preserve comments, formatting, manual controls and every retained job byte.
  for (const [position, text] of edits.sort((a,b) => b[0] - a[0])) source = source.slice(0,position) + text + source.slice(position);
  return source;
}

export function checkDocumentationPolicy({write = false} = {}) {
  let checked = 0;
  for (const file of readdirSync('.github/workflows').filter(file => /\.ya?ml$/u.test(file))) {
    if (lightweightWorkflows.has(file)) continue;
    const path = '.github/workflows/' + file;
    const source = readFileSync(path, 'utf8');
    const triggers = YAML.parse(source).on;
    if (!triggers || typeof triggers !== 'object' || !['push','pull_request'].some(event => Object.hasOwn(triggers,event))) continue;
    const expected = filteredWorkflow(source);
    if (write) writeFileSync(path, expected);
    else assert.equal(source, expected, path + ': run node scripts/docs-only-ci-policy.mjs --write');
    checked++;
  }
  return checked;
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  assert.ok(process.argv.slice(2).every(argument => argument === '--write'));
  console.log(JSON.stringify({documentationPolicy: 'explicit-paths/v1', checked: checkDocumentationPolicy({write: process.argv.includes('--write')})}));
}
