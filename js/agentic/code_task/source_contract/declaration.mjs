import { rustIdentifierIsValid } from '../identifier_domain.mjs';
function boundedInteger(value, bits) {
  const numeric = BigInt(value);
  return numeric >= -(1n << BigInt(bits - 1)) && numeric < (1n << BigInt(bits - 1));
}
/** Mirrors source_declaration_slots: bind the complete supported Rust item. */
export function sourceDeclarationSlots(content) {
  const integer = /^pub fn ([A-Za-z_][A-Za-z_0-9]*)\(\) -> i64 \{\n    (-?\d+)\n\}\n$/.exec(content);
  const division = /^pub fn ([A-Za-z_][A-Za-z_0-9]*)\(([A-Za-z_][A-Za-z_0-9]*): (f64)\) -> (f64) \{\n    \2 \/ (-?[0-9]+\.[0-9]+)\n\}\n$/.exec(content);
  const constant = /^pub const ([A-Za-z_][A-Za-z_0-9]*): (&str) = "([A-Za-z_][A-Za-z_0-9]*)";\n$/.exec(content);
  const equality = /^#\[test\]\nfn ([A-Za-z_][A-Za-z_0-9]*)\(\) \{\n    assert_eq!\((-?\d+), (-?\d+)\);\n\}\n$/.exec(content);
  const addition = /^#\[test\]\nfn ([A-Za-z_][A-Za-z_0-9]*)\(\) \{\n    assert_eq!\((-?\d+) \+ (-?\d+), (-?\d+)\);\n\}\n$/.exec(content);
  const declaration = integer ?? division ?? constant ?? equality ?? addition;
  if (declaration === null || !rustIdentifierIsValid(declaration[1])) return null;
  const identifier = declaration[1];
  if (integer !== null) {
    return boundedInteger(integer[2], 64) ? { kind: 'integer', slots: { identifier, value: integer[2] } } : null;
  }
  if (division !== null) {
    const value = Number(division[5]);
    if (!rustIdentifierIsValid(division[2]) || !Number.isFinite(value) || value === 0) return null;
    return { kind: 'float-division', slots: { identifier, value: division[5], 'parameter-type': division[3] } };
  }
  if (constant !== null) {
    return { kind: 'string-constant', slots: { identifier, 'constant-type': constant[2], value: constant[3] } };
  }
  if (!declaration.slice(2).every(value => boundedInteger(value, 32))) return null;
  return equality !== null
    ? { kind: 'equality-test', slots: { identifier, left: equality[2], right: equality[3] } }
    : { kind: 'addition-test', slots: { identifier, left: addition[2], right: addition[3], expected: addition[4] } };
}
