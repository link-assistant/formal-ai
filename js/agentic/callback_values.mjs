// ECMAScript callback-value operations; no coercion, captured state or Rust whitespace substitution.
export function callbackText(value) { return value.text; }
export function callbackValue(value) { return value.value; }
export function callbackTrim(value) { return value.trim(); }
export function callbackNonempty(value) { return value !== ''; }
export function callbackHasLength(value) { return value.length > 0; }
