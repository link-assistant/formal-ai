// formal-ai:self-translation:v1 source=Rust target=JavaScript sha256=aa05176829624d2c8eb0022d6d43ec66eb2349d5031bae55fd460c52a68191b0 bytes=797

// formal-ai:carried Rust attribute (a Rust item outside portable-pure-v1)
// | // Geometry written in Rust inside portable-pure-v1; the corpus translates it
// | // to JavaScript and back.
// | #![allow(clippy::missing_const_for_fn)]

// formal-ai:translated Rust function items=2 sha256=e23a41d9f960d5563f80d3a1100e45d31520d76ac1bc16e3923abb3c6eecf127
// | /// The square of `x`.
// | #[must_use]
// | pub const fn squared(x: f64) -> f64 {
// |     x * x
// | }
/**
 * The square of `x`.
 * @param {number} x
 * @returns {number}
 */
export function squared(x) {
  return x * x;
}

// formal-ai:translated Rust function items=2 sha256=0cfda7ad824f6001292e85e2e9bcbab61aab2c702874c54b00f5e445a3696eba
// | /// The hypotenuse of a right triangle with legs `a` and `b`.
// | #[must_use]
// | pub fn hypotenuse(a: f64, b: f64) -> f64 {
// |     let sum = squared(a) + squared(b);
// |     f64::sqrt(sum)
// | }
/**
 * The hypotenuse of a right triangle with legs `a` and `b`.
 * @param {number} a
 * @param {number} b
 * @returns {number}
 */
export function hypotenuse(a, b) {
  const sum = squared(a) + squared(b);
  return Math.sqrt(sum);
}

// formal-ai:translated Rust function items=2 sha256=4d2c24eb9128b3e9403e414e494b52ffe583a9f463d82522f850972a4734264a
// | /// The quadrant name of the point (`x`, `y`).
// | #[must_use]
// | pub fn quadrant(x: f64, y: f64) -> String {
// |     if x >= 0.0 && y >= 0.0 {
// |         return String::from("first");
// |     }
// |     let side = if x < 0.0 { "west" } else { "east" };
// |     side.to_string()
// | }
/**
 * The quadrant name of the point (`x`, `y`).
 * @param {number} x
 * @param {number} y
 * @returns {string}
 */
export function quadrant(x, y) {
  if (x >= 0 && y >= 0) {
    return 'first';
  } else {
    const side = x < 0 ? 'west' : 'east';
    return side;
  }
}

// formal-ai:translated Rust function items=2 sha256=95ff414b46d21c48bf506de242d3a938a3c38069d44cf544b094062000ccbda6
// | /// Whether `name` names the unit.
// | #[must_use]
// | pub fn is_unit(name: &str) -> bool {
// |     name.starts_with("unit") || name == "one"
// | }
/**
 * Whether `name` names the unit.
 * @param {string} name
 * @returns {boolean}
 */
export function isUnit(name) {
  return name.startsWith('unit') || name === 'one';
}
