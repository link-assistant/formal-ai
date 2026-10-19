// Object recognition without conversion hooks or Boolean-global evaluation.
export const nonNullObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
