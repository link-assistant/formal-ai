import { agenticMessage } from '../messages.mjs';
import { jsonText } from '../plan.mjs';
// Validate the bound call, never fill missing values by guessing.
export function invalidArguments(value, schema, path = '$', depth = 0) {
  if (depth > 32) return agenticMessage('tool-validation-schema-depth-limit', {path: path});
  if (schema === false) return agenticMessage('tool-validation-forbidden-value', {path: path});
  if (schema === true || !schema || typeof schema !== 'object') return null;
  if ('$ref' in schema) return agenticMessage('tool-validation-unresolved-schema-reference', {path: path});
  const matchesPart = part => !invalidArguments(value, part, path, depth + 1);
  if (schema.allOf && !schema.allOf.every(matchesPart)) return agenticMessage('tool-validation-allof-mismatch', {path: path});
  if (schema.anyOf && !schema.anyOf.some(matchesPart)) return agenticMessage('tool-validation-anyof-mismatch', {path: path});
  if (schema.oneOf && schema.oneOf.filter(matchesPart).length !== 1) return agenticMessage('tool-validation-oneof-mismatch', {path: path});
  if (schema.not && matchesPart(schema.not)) return agenticMessage('tool-validation-forbidden-schema', {path: path});
  const kind = value === null ? 'null' : Array.isArray(value) ? 'array' : typeof value;
  if (schema.type && ![schema.type].flat().some(type => typeof type === 'string' && (type.toLowerCase() === kind || type.toLowerCase() === 'integer' && Number.isInteger(value)))) return agenticMessage('tool-validation-type-mismatch', {path: path});
  if (schema.enum && !schema.enum.some(item => jsonText(item) === jsonText(value))) return agenticMessage('tool-validation-enum-mismatch', {path: path});
  if ('const' in schema && jsonText(schema.const) !== jsonText(value)) return agenticMessage('tool-validation-const-mismatch', {path: path});
  if (typeof value === 'string' && ([...value].length < (schema.minLength ?? 0) || [...value].length > (schema.maxLength ?? Infinity))) return agenticMessage('tool-validation-string-length-mismatch', {path: path});
  if (typeof value === 'string' && schema.pattern) {
    if (schema.pattern.length > 128 || value.length > 16384 || /[()|{}\\]/u.test(schema.pattern) || (schema.pattern.match(/[+*?]/gu) ?? []).length > 1) return agenticMessage('tool-validation-unsupported-schema-pattern', {path: path});
    try { if (!new RegExp(schema.pattern, 'u').test(value)) return agenticMessage('tool-validation-pattern-mismatch', {path: path}); }
    catch { return agenticMessage('tool-validation-unsupported-schema-pattern', {path: path}); }
  }
  if (typeof value === 'number' && (value < (schema.minimum ?? -Infinity) || value > (schema.maximum ?? Infinity))) return agenticMessage('tool-validation-numeric-range-mismatch', {path: path});
  if (kind === 'array') {
    if (value.length < (schema.minItems ?? 0) || value.length > (schema.maxItems ?? Infinity)) return agenticMessage('tool-validation-array-length-mismatch', {path: path});
    for (let i = 0; i < value.length; i++) {
      const error = invalidArguments(value[i], schema.items, `${path}[${i}]`, depth + 1);
      if (error) return error;
    }
  }
  if (kind === 'object') {
    for (const key of schema.required ?? []) {
      if (!Object.hasOwn(value, key) || value[key] === '' || value[key] === null) return agenticMessage('tool-validation-required-value-is-unbound', {path: `${path}.${key}`});
    }
    for (const [key, item] of Object.entries(value)) {
      const property = schema.properties && Object.hasOwn(schema.properties, key) ? schema.properties[key] : schema.additionalProperties;
      const error = invalidArguments(item, property, `${path}.${key}`, depth + 1);
      if (error) return error;
    }
  }
  return null;
}
