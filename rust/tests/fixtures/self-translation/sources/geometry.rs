// Geometry written in Rust inside portable-pure-v1; the corpus translates it
// to JavaScript and back.
#![allow(clippy::missing_const_for_fn)]

/// The square of `x`.
#[must_use]
pub const fn squared(x: f64) -> f64 {
    x * x
}

/// The hypotenuse of a right triangle with legs `a` and `b`.
#[must_use]
pub fn hypotenuse(a: f64, b: f64) -> f64 {
    let sum = squared(a) + squared(b);
    f64::sqrt(sum)
}

/// The quadrant name of the point (`x`, `y`).
#[must_use]
pub fn quadrant(x: f64, y: f64) -> String {
    if x >= 0.0 && y >= 0.0 {
        return String::from("first");
    }
    let side = if x < 0.0 { "west" } else { "east" };
    side.to_string()
}

/// Whether `name` names the unit.
#[must_use]
pub fn is_unit(name: &str) -> bool {
    name.starts_with("unit") || name == "one"
}
