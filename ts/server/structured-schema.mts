// Response schemas treat required as presence, including for null and empty strings.
import { jsonText } from '../agentic/plan.mjs';
import { isMap } from './protocol-values.mjs';

const TYPES = ['object', 'array', 'string', 'number', 'integer', 'boolean', 'null'];
const ANNOTATIONS = ['title', 'description', 'default', 'examples', 'const'];
const COUNTS = ['minLength', 'maxLength', 'minItems', 'maxItems', 'minProperties', 'maxProperties'];
const LIMITS = ['minimum', 'maximum', 'exclusiveMinimum', 'exclusiveMaximum'];
const COMBINATIONS = ['anyOf', 'allOf', 'oneOf'];
const MAXIMUM_DEPTH = 32;

export function schemaReference(root, reference) {
  if (reference === '#') return root;
  if (typeof reference !== 'string' || !reference.startsWith('#/')) return undefined;
  try {
    return decodeURIComponent(reference.slice(2)).split('/').reduce((node, part) => {
      const key = part.replaceAll('~1', '/').replaceAll('~0', '~');
      return node && Object.hasOwn(node, key) ? node[key] : undefined;
    }, root);
  } catch {
    return undefined;
  }
}

function safePattern(pattern) {
  // Restrict untrusted expressions to simple ASCII classes and at most one quantifier.
  if (typeof pattern !== 'string' || pattern.length > 128
      || /[^ -~]|[.()|{}\\]|\[\^|&&|--|~~/u.test(pattern)
      || (pattern.match(/[+*?]/gu) ?? []).length > 1) return null;
  try { return new RegExp(pattern, 'u'); } catch { return null; }
}

export function supportedSchema(root) {
  const visited = new WeakSet();
  function visit(schema, depth) {
    if (depth > MAXIMUM_DEPTH) return false;
    if (typeof schema === 'boolean') return true;
    if (!isMap(schema)) return false;
    if (visited.has(schema)) return true;
    visited.add(schema);
    const nested = value => visit(value, depth + 1);
    return Object.entries(schema).every(([key, value]) => {
      if (ANNOTATIONS.includes(key)) return true;
      if (COUNTS.includes(key)) return Number.isSafeInteger(value) && value >= 0;
      if (LIMITS.includes(key)) return Number.isFinite(value);
      if (COMBINATIONS.includes(key)) return Array.isArray(value) && value.length > 0 && value.every(nested);
      switch (key) {
        case '$schema': return value === 'https://json-schema.org/draft/2020-12/schema';
        case '$ref': return nested(schemaReference(root, value));
        case '$defs':
        case 'definitions':
        case 'properties': return isMap(value) && Object.values(value).every(nested);
        case 'additionalProperties':
        case 'items':
        case 'not': return nested(value);
        case 'required': return Array.isArray(value) && value.every(item => typeof item === 'string');
        case 'enum': return Array.isArray(value) && value.length > 0;
        case 'type': {
          const types = Array.isArray(value) ? value : [value];
          return types.length > 0 && types.every(kind => TYPES.includes(kind));
        }
        case 'pattern': return safePattern(value) !== null;
        default: return false;
      }
    });
  }
  return isMap(root) && visit(root, 0);
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
