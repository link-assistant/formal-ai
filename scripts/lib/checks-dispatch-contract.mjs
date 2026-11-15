// Closed workflow-dispatch admission, not a general YAML interpreter.
// It reads only the finite block/choice input shapes this action supports.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {unquote} from './ci-speed-workflows.mjs';

const identifier = /^[A-Za-z][A-Za-z0-9_-]*$/u;
const ignored = line => /^\s*(?:#.*)?$/u.test(line);
const indentation = line => /^ */u.exec(line)[0].length;
function scalar(raw) {
  const value = raw.trim();
  if (/^'(?:[^']*)'(?:\s+#.*)?$/u.test(value)
    || /^"(?:[^"\\]*)"(?:\s+#.*)?$/u.test(value)) return unquote(value);
  const plain = value.replace(/\s+#.*$/u, '').trim();
  return plain !== '' && !/[\[\]{}&*!|>'"\\]/u.test(plain) ? plain : null;
}
function fields(lines, indent) {
  const result = new Map();
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];
    if (ignored(line)) continue;
    if (indentation(line) !== indent) throw Error('UnsupportedWorkflowIndentation');
    const match = /^(?:([A-Za-z][A-Za-z0-9_-]*)|'([A-Za-z][A-Za-z0-9_-]*)'|"([A-Za-z][A-Za-z0-9_-]*)"):\s*(.*)$/u.exec(line.slice(indent));
    if (!match) throw Error('UnsupportedWorkflowMapping');
    const key = match[1] ?? match[2] ?? match[3];
    if (result.has(key)) throw Error('DuplicateWorkflowKey');
    let end = index + 1;
    while (end < lines.length && (ignored(lines[end]) || indentation(lines[end]) > indent)) end++;
    result.set(key, {value: match[4].replace(/(?:^|\s)#.*$/u, '').trim(), body: lines.slice(index + 1, end)});
    index = end - 1;
  }
  return result;
}
function mapping(field, indent) {
  if (!field || field.value !== '') throw Error('ExplicitWorkflowMappingRequired');
  return fields(field.body, indent);
}
function choiceOptions(field) {
  if (!field) return null;
  let values;
  if (field.value.startsWith('[') && field.value.endsWith(']') && field.body.every(ignored)) {
    values = field.value.slice(1, -1).split(',').map(scalar);
  } else if (field.value === '') {
    values = field.body.filter(line => !ignored(line)).map(line => {
      const item = /^ {10}-\s+(.+)$/u.exec(line);
      return item ? scalar(item[1]) : null;
    });
  } else return null;
  return values.length > 0 && values.every(value => typeof value === 'string' && identifier.test(value))
    && new Set(values).size === values.length ? values : null;
}
function inputContract(field) {
  const config = mapping(field, 8);
  if ([...config.keys()].some(key => !['description', 'required', 'type', 'default', 'options'].includes(key))) return null;
  for (const key of ['type', 'required', 'default']) {
    if (config.has(key) && !config.get(key).body.every(ignored)) return null;
  }
  const description = config.get('description');
  if (description && !/^[|>][+-]?$/u.test(description.value)
    && (!description.body.every(ignored) || scalar(description.value) === null)) return null;
  const requiredRaw = config.get('required')?.value;
  if (requiredRaw !== undefined && !['true', 'false'].includes(requiredRaw)) return null;
  const type = config.has('type') ? scalar(config.get('type').value) : 'string';
  if (!['string', 'boolean', 'number', 'choice'].includes(type)) return null;
  const options = type === 'choice' ? choiceOptions(config.get('options')) : null;
  if (type === 'choice' && options === null || type !== 'choice' && config.has('options')) return null;
  const defaultField = config.get('default');
  let defaultValue;
  if (defaultField) {
    if (!defaultField.body.every(ignored)) return null;
    const raw = defaultField.value;
    if (type === 'boolean') {
      if (!['true', 'false'].includes(raw)) return null;
      defaultValue = raw;
    } else if (type === 'number') {
      if (!/^-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?$/u.test(raw) || !Number.isFinite(Number(raw))) return null;
      defaultValue = raw;
    } else {
      defaultValue = scalar(raw);
      if (defaultValue === null || type === 'choice' && !options.includes(defaultValue)) return null;
    }
  }
  return {type, options, required: requiredRaw === 'true', hasDefault: Boolean(defaultField), defaultValue};
}

/** Admit only an explicit checks-mode contract; other choices are never selected. */
export function checksDispatchContract(source) {
  try {
    if (typeof source !== 'string' || Buffer.byteLength(source) > 1048576) return false;
    const lines = source.replace(/\r\n/gu, '\n').split('\n');
    const roots = fields(lines, 0);
    if ([...roots.values()].some(field => /^[\x27\x22]/u.test(field.value) && scalar(field.value) === null)) return false;
    const trigger = roots.get('on');
    if (!trigger || trigger.value !== '') return false;
    if (trigger.body.filter(line => !ignored(line)).some(line => /\t|(?:^|\s)[&*!][A-Za-z0-9_-]|<<\s*:/u.test(line))) return false;
    const events = mapping(trigger, 2);
    if (!events.has('pull_request') || events.get('pull_request').value !== '') return false;
    const dispatch = mapping(events.get('workflow_dispatch'), 4);
    if ([...dispatch.keys()].some(key => key !== 'inputs')) return false;
    const inputs = mapping(dispatch.get('inputs'), 6);
    const contracts = new Map([...inputs].map(([name, field]) => [name, inputContract(field)]));
    if ([...contracts.values()].some(value => value === null)) return false;
    const mode = contracts.get('mode'), pullRequest = contracts.get('pull-request');
    if (mode?.type !== 'choice' || !mode.options.includes('checks')
      || (mode.hasDefault ? mode.defaultValue : mode.options[0]) !== 'checks'
      || pullRequest?.type !== 'string') return false;
    for (const [name, input] of contracts) {
      if (!['mode', 'pull-request'].includes(name) && input.required && !input.hasDefault) return false;
    }
    return true;
  } catch { return false; }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const file = process.argv[2];
  if (!/^\.github\/workflows\/[A-Za-z0-9_.-]+\.ya?ml$/u.test(file ?? '') || process.argv.length !== 3) {
    process.exitCode = 1;
  } else {
    try { process.exitCode = checksDispatchContract(readFileSync(file, 'utf8')) ? 0 : 1; }
    catch { process.exitCode = 1; }
  }
}
