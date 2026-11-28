import { cached } from '../agentic/host.mjs';
import { agenticMessage } from '../agentic/messages.mjs';
import { schemaReference, validOutput } from './structured-schema.mjs';
import { assessCriterion, criterionReference, mentionsSubject } from './structured-verdict.mjs';

function pattern(rule, flags = 'iu') {
  return cached('structured-quote-' + rule, () =>
    new RegExp(agenticMessage('structured-quote-' + rule), flags));
}

function spans(text) {
  const quotes = [];
  let start = 0;
  for (const boundary of text.matchAll(pattern('boundary', 'gu'))) {
    quotes.push(text.slice(start, boundary.index + (boundary[1]?.length ?? 0)).trim());
    start = boundary.index + boundary[0].length;
  }
  return [...quotes, text.slice(start).trim()].filter(Boolean);
}

function criteriaFrom(header, normalizeKey) {
  const marker = pattern('criteria').exec(header);
  if (!marker) return { definitions: new Map() };
  const definitions = new Map();
  const tail = header.slice(marker.index + marker[0].length);
  const stop = Array.from(tail.matchAll(pattern('boundary', 'gu'))).find(boundary => boundary[1]);
  const contents = tail.slice(0, stop?.index).trim();
  const assignments = Array.from(contents.matchAll(pattern('assignment', 'giu')));
  for (const [index, assignment] of assignments.entries()) {
    const key = normalizeKey(assignment[1]);
    const definition = contents.slice(assignment.index + assignment[0].length,
      assignments[index + 1]?.index).trim();
    definitions.set(key, definitions.has(key) ? '' : definition);
  }
  return { definitions, single: assignments.length ? undefined : contents };
}

function sourceParts(header, request, text, normalizeKey) {
  const criteria = request ? header.slice(0, request.index) : header;
  return { ...criteriaFrom(criteria, normalizeKey), reference: criterionReference(criteria), text, quotes: spans(text),
    instruction: request ? header.slice(request.index) : '' };
}

function quoteSource(source, normalizeKey, requireInstruction = false) {
  if (Buffer.byteLength(source) > 8192) return undefined;
  const marker = pattern('source').exec(source);
  if (marker) {
    const header = source.slice(0, marker.index);
    const instruction = pattern('instruction').exec(header);
    if (requireInstruction && !instruction) return undefined;
    const text = source.slice(marker.index + marker[0].length);
    return sourceParts(header, instruction, text, normalizeKey);
  }
  const request = pattern('instruction').exec(source);
  if (request) {
    const start = request.index + request[0].length - 1;
    const separator = /[:：\r\n]/u.exec(source.slice(start));
    if (!separator) return undefined;
    const end = start + separator.index;
    const text = source.slice(end + 1);
    return sourceParts(source.slice(0, end), request, text, normalizeKey);
  }
  if (requireInstruction || pattern('criteria').test(source)) return undefined;
  return { text: source, quotes: spans(source), instruction: '', definitions: new Map() };
}

function targetFrom(instruction) {
  const cue = pattern('target').exec(instruction);
  if (!cue) return undefined;
  return spans(instruction.slice(cue.index + cue[0].length))[0]?.split(',')[0]
    .trim().replace(/^["'“«]+|[.!?"'”»]+$/gu, '').trim();
}

function support(source, criterion) {
  if (!criterion || assessCriterion(criterion, source.text, source.reference)?.result !== true) return undefined;
  return source.quotes.find(quote => assessCriterion(criterion, quote, source.reference)?.result === true);
}

function singleQuote(source, description = '') {
  if (!source) return undefined;
  const target = targetFrom(source.instruction) ?? targetFrom(description);
  if (target !== undefined) return source.quotes.find(quote => mentionsSubject(quote, target));
  const criteria = [...source.definitions.values()];
  if (source.single !== undefined || criteria.length === 1) return support(source, source.single ?? criteria[0]);
  if (!criteria.length && source.quotes.length === 1) return source.quotes[0];
  return undefined;
}

export function requestedQuote(source, normalizeKey) {
  const input = quoteSource(source, normalizeKey, true);
  return input?.instruction && targetFrom(input.instruction) ? singleQuote(input) : undefined;
}

function resolve(rule, root) {
  return schemaReference(root, rule?.$ref) ?? rule;
}

function description(field, root) {
  const text = typeof field?.description === 'string' ? field.description : resolve(field, root)?.description;
  return typeof text === 'string' ? text : '';
}

function stringShape(field, root, visited = new Set()) {
  const rule = resolve(field, root);
  if (!rule || visited.has(rule)) return false;
  visited.add(rule);
  return [rule?.type].flat().includes('string') || rule?.enum?.some(value => typeof value === 'string')
    || typeof rule?.const === 'string' || ['allOf', 'anyOf', 'oneOf'].some(key =>
      rule?.[key]?.some(member => stringShape(member, root, visited)));
}

function quoteField(name, field, root, normalizeKey) {
  return stringShape(field, root) &&
    (pattern('cue').test(normalizeKey(name)) || pattern('cue').test(description(field, root)));
}

function quoteItems(field, root, normalizeKey) {
  const rule = resolve(field, root);
  if (rule?.type && ![rule.type].flat().includes('array')) return undefined;
  const entries = Object.entries(resolve(rule?.items, root)?.properties ?? {});
  const quotes = entries.filter(([name, property]) => quoteField(name, property, root, normalizeKey));
  if (quotes.length !== 1) return undefined;
  const [quoteName] = quotes[0];
  const selectors = entries.flatMap(([name, property]) => {
    const labels = resolve(property, root)?.enum;
    return name !== quoteName && labels?.every(label => typeof label === 'string') ? [{ name, labels }] : [];
  });
  const selector = selectors.length === 1 ? selectors[0] : undefined;
  return selector && { ...selector, quoteName };
}

function criterionQuotes(source, items, normalizeKey) {
  if (!source) return [];
  const { name, labels, quoteName } = items;
  return [...new Set(labels)].flatMap(label => {
    const criterion = source.definitions.size ? source.definitions.get(normalizeKey(label))
      : source.single !== undefined ? (labels.length === 1 ? source.single : undefined) : label;
    const quote = support(source, criterion);
    return quote === undefined ? [] : [{ [name]: label, [quoteName]: quote }];
  });
}

// Every recognized field replaces a model-produced value, including when the
// source cannot ground a quote. The complete output still undergoes schema validation.
export function quoteFields(root, fields, source, normalizeKey) {
  const values = new Map();
  const missing = [];
  let input;
  for (const [name, field] of Object.entries(fields.properties ?? {})) {
    const items = quoteItems(field, root, normalizeKey);
    if (!items && !quoteField(name, field, root, normalizeKey)) continue;
    input ??= quoteSource(source, normalizeKey) ?? null;
    const value = items && Buffer.byteLength(source) <= 8192 ? criterionQuotes(input, items, normalizeKey)
      : singleQuote(input, description(field, root));
    if (items && value?.length === 0 && validOutput(value, field, root)) missing.push(name);
    values.set(name, value);
  }
  return { values, missing };
}
