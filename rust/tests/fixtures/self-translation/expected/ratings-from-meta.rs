#![allow(
    clippy::float_cmp,
    clippy::imprecise_flops,
    clippy::missing_const_for_fn,
    clippy::suboptimal_flops
)]

pub const MAXIMUM_SCORE: f64 = 10.0;

pub const RATING_UNIT: &str = "stars";

#[must_use]
pub fn add(a: f64, b: f64) -> f64 {
    a + b
}

#[must_use]
pub fn average(total: f64, count: f64) -> f64 {
    if count <= 0.0 { 0.0 } else { total / count }
}

#[must_use]
pub fn distance(x: f64, y: f64) -> f64 {
    let squares = x * x + y * y;
    f64::sqrt(squares)
}

#[must_use]
pub fn score_band(score: f64) -> String {
    if score >= MAXIMUM_SCORE {
        "top".to_string()
    } else if score > 5.0 {
        "high".to_string()
    } else {
        "low".to_string()
    }
}

#[must_use]
pub fn is_rating_unit(word: &str) -> bool {
    word == RATING_UNIT || word.starts_with("star") && word.ends_with("rs")
}

#[must_use]
pub fn mentions_rating(text: &str) -> bool {
    text.contains(RATING_UNIT) && text != RATING_UNIT
}

#[must_use]
pub fn rating_label(score: f64) -> String {
    let band = score_band(score);
    if band == "top" {
        band
    } else {
        "rated".to_string()
    }
}

#[must_use]
pub fn is_rated(score: f64, hidden: bool) -> bool {
    !hidden && -score < 0.0
}

#[must_use]
pub fn spread(a: f64, b: f64) -> f64 {
    f64::abs(a - b)
}

#[must_use]
pub fn whole_stars(score: f64) -> f64 {
    if f64::floor(score) % 2.0 == 0.0 {
        f64::max(f64::floor(score), 1.0)
    } else {
        f64::min(f64::ceil(score), MAXIMUM_SCORE)
    }
}
