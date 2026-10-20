// Validate the bound call, never fill missing values by guessing.
export function invalidArguments(value, schema, path = '$', depth = 0) {
  if (depth > 32) return `${path}: schema depth limit`;
  if (schema === false) return `${path}: forbidden value`;
  if (schema === true || !schema || typeof schema !== 'object') return null;
  if ('$ref' in schema) return `${path}: unresolved schema reference`;
  const matchesPart = part => !invalidArguments(value, part, path, depth + 1);
  if (schema.allOf && !schema.allOf.every(matchesPart)) return `${path}: allOf mismatch`;
  if (schema.anyOf && !schema.anyOf.some(matchesPart)) return `${path}: anyOf mismatch`;
  if (schema.oneOf && schema.oneOf.filter(matchesPart).length !== 1) return `${path}: oneOf mismatch`;
  if (schema.not && matchesPart(schema.not)) return `${path}: forbidden schema`;
  const kind = value === null ? 'null' : Array.isArray(value) ? 'array' : typeof value;
  if (schema.type && ![schema.type].flat().some(type => type === kind || type === 'integer' && Number.isInteger(value))) return `${path}: type mismatch`;
  if (schema.enum && !schema.enum.some(item => JSON.stringify(item) === JSON.stringify(value))) return `${path}: enum mismatch`;
  if ('const' in schema && JSON.stringify(schema.const) !== JSON.stringify(value)) return `${path}: const mismatch`;
  if (typeof value === 'string' && ([...value].length < (schema.minLength ?? 0) || [...value].length > (schema.maxLength ?? Infinity))) return `${path}: string length mismatch`;
  if (typeof value === 'string' && schema.pattern) {
    if (schema.pattern.length > 128 || value.length > 16384 || /[()|{}\\]/u.test(schema.pattern) || (schema.pattern.match(/[+*?]/gu) ?? []).length > 1) return `${path}: unsupported schema pattern`;
    try { if (!new RegExp(schema.pattern, 'u').test(value)) return `${path}: pattern mismatch`; }
    catch { return `${path}: unsupported schema pattern`; }
  }
  if (typeof value === 'number' && (value < (schema.minimum ?? -Infinity) || value > (schema.maximum ?? Infinity))) return `${path}: numeric range mismatch`;
  if (kind === 'array') {
    if (value.length < (schema.minItems ?? 0) || value.length > (schema.maxItems ?? Infinity)) return `${path}: array length mismatch`;
    for (let i = 0; i < value.length; i++) {
      const error = invalidArguments(value[i], schema.items, `${path}[${i}]`, depth + 1);
      if (error) return error;
    }
  }
  if (kind === 'object') {
    for (const key of schema.required ?? []) {
      if (!Object.hasOwn(value, key) || value[key] === '' || value[key] === null) return `${path}.${key}: required value is unbound`;
    }
    for (const [key, item] of Object.entries(value)) {
      const property = schema.properties?.[key] ?? schema.additionalProperties;
      const error = invalidArguments(item, property, `${path}.${key}`, depth + 1);
      if (error) return error;
    }
  }
  return null;
}
