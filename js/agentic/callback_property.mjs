// ECMAScript named-property callback factory; the source-derived name is captured once.
// Mirrors callback_property in rust/src/callback_values.rs; native receivers define property semantics.
export function callbackProperty(property) { return value => value[property]; }
