import { agenticMessage } from '../messages.mjs';
// Validate the bound call, never fill missing values by guessing.
export function invalidArguments(value, schema, path = '$', depth = 0) {
  if (depth > 32) return agenticMessage('tool_validation_schema_depth_limit', {path: path});
  if (schema === false) return agenticMessage('tool_validation_forbidden_value', {path: path});
  if (schema === true || !schema || typeof schema !== 'object') return null;
  if ('$ref' in schema) return agenticMessage('tool_validation_unresolved_schema_reference', {path: path});
  const matchesPart = part => !invalidArguments(value, part, path, depth + 1);
  if (schema.allOf && !schema.allOf.every(matchesPart)) return agenticMessage('tool_validation_allof_mismatch', {path: path});
  if (schema.anyOf && !schema.anyOf.some(matchesPart)) return agenticMessage('tool_validation_anyof_mismatch', {path: path});
  if (schema.oneOf && schema.oneOf.filter(matchesPart).length !== 1) return agenticMessage('tool_validation_oneof_mismatch', {path: path});
  if (schema.not && matchesPart(schema.not)) return agenticMessage('tool_validation_forbidden_schema', {path: path});
  const kind = value === null ? 'null' : Array.isArray(value) ? 'array' : typeof value;
  if (schema.type && ![schema.type].flat().some(type => typeof type === 'string' && (type.toLowerCase() === kind || type.toLowerCase() === 'integer' && Number.isInteger(value)))) return agenticMessage('tool_validation_type_mismatch', {path: path});
  if (schema.enum && !schema.enum.some(item => JSON.stringify(item) === JSON.stringify(value))) return agenticMessage('tool_validation_enum_mismatch', {path: path});
  if ('const' in schema && JSON.stringify(schema.const) !== JSON.stringify(value)) return agenticMessage('tool_validation_const_mismatch', {path: path});
  if (typeof value === 'string' && ([...value].length < (schema.minLength ?? 0) || [...value].length > (schema.maxLength ?? Infinity))) return agenticMessage('tool_validation_string_length_mismatch', {path: path});
  if (typeof value === 'string' && schema.pattern) {
    if (schema.pattern.length > 128 || value.length > 16384 || /[()|{}\\]/u.test(schema.pattern) || (schema.pattern.match(/[+*?]/gu) ?? []).length > 1) return agenticMessage('tool_validation_unsupported_schema_pattern', {path: path});
    try { if (!new RegExp(schema.pattern, 'u').test(value)) return agenticMessage('tool_validation_pattern_mismatch', {path: path}); }
    catch { return agenticMessage('tool_validation_unsupported_schema_pattern', {path: path}); }
  }
  if (typeof value === 'number' && (value < (schema.minimum ?? -Infinity) || value > (schema.maximum ?? Infinity))) return agenticMessage('tool_validation_numeric_range_mismatch', {path: path});
  if (kind === 'array') {
    if (value.length < (schema.minItems ?? 0) || value.length > (schema.maxItems ?? Infinity)) return agenticMessage('tool_validation_array_length_mismatch', {path: path});
    for (let i = 0; i < value.length; i++) {
      const error = invalidArguments(value[i], schema.items, `${path}[${i}]`, depth + 1);
      if (error) return error;
    }
  }
  if (kind === 'object') {
    for (const key of schema.required ?? []) {
      if (!Object.hasOwn(value, key) || value[key] === '' || value[key] === null) return agenticMessage('tool_validation_required_value_is_unbound', {path: `${path}.${key}`});
    }
    for (const [key, item] of Object.entries(value)) {
      const property = schema.properties?.[key] ?? schema.additionalProperties;
      const error = invalidArguments(item, property, `${path}.${key}`, depth + 1);
      if (error) return error;
    }
  }
  return null;
}
