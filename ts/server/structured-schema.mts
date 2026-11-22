// Response schemas treat required as presence, including for null and empty strings.
import { jsonText } from '../agentic/plan.mjs';
import { isMap } from './protocol-values.mjs';
import { anchoredPattern } from './structured-pattern.mjs';

const TYPES = ['object', 'array', 'string', 'number', 'integer', 'boolean', 'null'];
const ANNOTATIONS = [
  'title', 'description', 'default', 'examples', '$id', '$comment', 'deprecated',
  'readOnly', 'writeOnly', 'contentMediaType', 'contentEncoding', 'format',
];
const DIALECTS = ['draft-07/schema', 'draft/2019-09/schema', 'draft/2020-12/schema'];
const COUNTS = ['minLength', 'maxLength', 'minItems', 'maxItems', 'minProperties', 'maxProperties'];
const LIMITS = ['minimum', 'maximum', 'exclusiveMinimum', 'exclusiveMaximum'];
const COMBINATIONS = ['anyOf', 'allOf', 'oneOf'];
const MAXIMUM_DEPTH = 32;

function referencePointer(reference) {
  if (typeof reference !== 'string' || !reference.startsWith('#')) return undefined;
  try {
    const pointer = decodeURIComponent(reference.slice(1));
    return pointer === '' || pointer.startsWith('/') ? pointer : undefined;
  } catch {
    return undefined;
  }
}

function atPointer(root, pointer) {
  if (pointer === '') return root;
  return pointer.slice(1).split('/').reduce((node, part) => {
    const key = part.replaceAll('~1', '/').replaceAll('~0', '~');
    return node && typeof node === 'object' && Object.hasOwn(node, key) ? node[key] : undefined;
  }, root);
}

export function schemaReference(root, reference) {
  const pointer = referencePointer(reference);
  return pointer === undefined ? undefined : atPointer(root, pointer);
}

function supportedDialect(value) {
  if (typeof value !== 'string') return false;
  const dialect = value.replace(/^https?:\/\/json-schema\.org\//u, '');
  return dialect !== value && DIALECTS.includes(dialect.endsWith('#') ? dialect.slice(0, -1) : dialect);
}

function childPointer(pointer, key) {
  return `${pointer}/${String(key).replaceAll('~', '~0').replaceAll('/', '~1')}`;
}

function safePattern(pattern) {
  if (typeof pattern !== 'string' || pattern.length > 128 || /[^ -~]|\[\^|&&|--|~~/u.test(pattern)) return null;
  if (/[.()|{}\\]/u.test(pattern) || (pattern.match(/[+*?]/gu) ?? []).length > 1) {
    return anchoredPattern(pattern);
  }
  try { return new RegExp(pattern, 'u'); } catch { return null; }
}

export function schemaIssue(root) {
  const visited = new WeakSet();
  function visit(schema, pointer, depth, keyword) {
    const locationDepth = (pointer.match(/\//gu) ?? []).length;
    if (depth > MAXIMUM_DEPTH || locationDepth > MAXIMUM_DEPTH) return { keyword, pointer };
    if (typeof schema === 'boolean') return null;
    if (!isMap(schema)) return { keyword, pointer };
    if (visited.has(schema)) return null;
    visited.add(schema);
    for (const [key, value] of Object.entries(schema)) {
      if (ANNOTATIONS.includes(key)) continue;
      const fieldDepth = locationDepth + 1;
      if (fieldDepth > MAXIMUM_DEPTH) return { keyword: key, pointer };
      const fieldPointer = childPointer(pointer, key);
      const nested = (part, at) => visit(part, at, depth + 1, key);
      let valid;
      if (key === '$ref') {
        const targetPointer = referencePointer(value);
        const target = targetPointer === undefined ? undefined : atPointer(root, targetPointer);
        if (target === undefined) return { keyword: key, pointer };
        const issue = nested(target, targetPointer);
        if (issue) return issue;
      } else if (['$defs', 'definitions', 'properties'].includes(key) || COMBINATIONS.includes(key)) {
        const isCombination = COMBINATIONS.includes(key);
        if (isCombination ? !Array.isArray(value) || value.length === 0 : !isMap(value)) {
          return { keyword: key, pointer };
        }
        for (const [name, part] of Object.entries(value)) {
          const issue = nested(part, childPointer(fieldPointer, name));
          if (issue) return issue;
        }
      } else if (['additionalProperties', 'items', 'not'].includes(key)) {
        const issue = nested(value, fieldPointer);
        if (issue) return issue;
      } else {
        if (key === '$schema') valid = supportedDialect(value);
        else if (COUNTS.includes(key)) valid = Number.isSafeInteger(value) && value >= 0;
        else if (LIMITS.includes(key)) valid = typeof value === 'number';
        else if (key === 'required') valid = Array.isArray(value) && value.every(item => typeof item === 'string');
        else if (key === 'enum') valid = Array.isArray(value) && value.length > 0;
        else if (key === 'type') {
          const types = Array.isArray(value) ? value : [value];
          valid = types.length > 0 && types.every(kind => TYPES.includes(kind));
        } else if (key === 'pattern') valid = safePattern(value) !== null;
        else if (key === 'const') valid = true;
        else return { keyword: key, pointer, unsupported: true };
        if (!valid || !portableValue(value, fieldDepth)) return { keyword: key, pointer };
      }
    }
    return null;
  }
  return isMap(root) ? visit(root, '', 0, 'schema') : { keyword: 'schema', pointer: '' };
}

export function validOutput(value, schema, root = schema) {
  let remaining = 8192;
  let exhausted = false;
  function valid(item, rule, depth) {
    if (--remaining < 0 || depth > MAXIMUM_DEPTH) {
      exhausted = true;
      return false;
    }
    if (rule === undefined || rule === true) return true;
    if (rule === false) return false;
    const nested = part => valid(item, part, depth + 1);
    if (rule.$ref !== undefined) {
      const target = schemaReference(root, rule.$ref);
      if (target === undefined || !nested(target)) return false;
    }
    if (rule.allOf && !rule.allOf.every(nested)) return false;
    if (rule.anyOf && !rule.anyOf.some(nested)) return false;
    if (rule.oneOf && rule.oneOf.filter(nested).length !== 1) return false;
    if (Object.hasOwn(rule, 'not') && nested(rule.not)) return false;
    const kind = item === null ? 'null' : Array.isArray(item) ? 'array' : typeof item;
    if (rule.type && ![rule.type].flat().some(type => type === kind
        || (type === 'integer' && Number.isInteger(item)))) return false;
    if (rule.enum && !rule.enum.some(member => jsonText(member) === jsonText(item))) return false;
    if (Object.hasOwn(rule, 'const') && jsonText(rule.const) !== jsonText(item)) return false;
    if (kind === 'number') {
      return item >= (rule.minimum ?? -Infinity) && item <= (rule.maximum ?? Infinity)
        && item > (rule.exclusiveMinimum ?? -Infinity) && item < (rule.exclusiveMaximum ?? Infinity);
    }
    if (kind === 'string') {
      const length = [...item].length;
      return length >= (rule.minLength ?? 0) && length <= (rule.maxLength ?? Infinity)
        && (rule.pattern === undefined || (length <= 16384 && safePattern(rule.pattern)?.test(item)));
    }
    if (kind === 'array') {
      return item.length >= (rule.minItems ?? 0) && item.length <= (rule.maxItems ?? Infinity)
        && item.every(child => valid(child, rule.items, depth + 1));
    }
    if (kind === 'object') {
      const entries = Object.entries(item);
      return entries.length >= (rule.minProperties ?? 0) && entries.length <= (rule.maxProperties ?? Infinity)
        && (rule.required ?? []).every(key => Object.hasOwn(item, key))
        && entries.every(([key, child]) => valid(child,
          rule.properties && Object.hasOwn(rule.properties, key) ? rule.properties[key] : rule.additionalProperties,
          depth + 1));
    }
    return true;
  }
  return Boolean(valid(value, schema, 0) && !exhausted);
}

export function portableValue(value, depth = 0) {
  if (depth > MAXIMUM_DEPTH) return false;
  if (typeof value === 'number') {
    return Number.isFinite(value) && (!Number.isInteger(value) || Number.isSafeInteger(value));
  }
  if (value && typeof value === 'object') return Object.values(value).every(child => portableValue(child, depth + 1));
  return true;
}
