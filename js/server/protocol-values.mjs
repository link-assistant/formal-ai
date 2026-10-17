// Stateless protocol envelope values; each call retains its original allocations and access order.
export function isMap(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

export function optionalValue(object, field) {
  const value = object[field];
  return value === undefined || value === null ? null : value;
}

export function chatMessage(role, content, extra = {}) {
  return { role, content, tool_calls: [], tool_call_id: null, name: null, is_error: false, ...extra };
}
