// formal-ai:self-translation:v1 source=JavaScript target=Rust sha256=5d0e5a1af8e6c6132e0497ff55a51c28ae7463d5455b00b430b20411d7b8c014 bytes=4472

// formal-ai:prelude begin
#![allow(
    clippy::float_cmp,
    clippy::imprecise_flops,
    clippy::missing_const_for_fn,
    clippy::suboptimal_flops
)]
// formal-ai:prelude end

// formal-ai:carried JavaScript import (no definition)
// | // Ratings arithmetic and text predicates the portable fragment translates,
// | // beside the items it carries with the construct map's reason.
// | import { readFileSync } from 'node:fs';

// formal-ai:translated JavaScript constant items=1 sha256=62b7974ff34f9de4816b23459faeab27d2815c1e292d6866de08e87c049f30a6
// | /** The largest score a rating holds. */
// | export const MAXIMUM_SCORE = 10;
pub const MAXIMUM_SCORE: f64 = 10.0;

// formal-ai:translated JavaScript constant items=1 sha256=6d5e8051899a763a6059e1be28494589bb1ca139bdbbcb797ab01e7f9d1e8d85
// | /** The word a rating is filed under. */
// | export const RATING_UNIT = 'stars';
pub const RATING_UNIT: &str = "stars";

// formal-ai:translated JavaScript function items=1 sha256=1f488f08de662e1c73595f5a3aa601ee9467ac4bb8a707daad2c8fa48c84550f
// | /**
// |  * The sum of `a` and `b`.
// |  * @param {number} a
// |  * @param {number} b
// |  * @returns {number}
// |  */
// | export function add(a, b) {
// |   return a + b;
// | }
#[must_use]
pub fn add(a: f64, b: f64) -> f64 {
    a + b
}

// formal-ai:translated JavaScript function items=1 sha256=6108042fae673ffce027838d491ab449c8bfb8260c5ce523652229a670622806
// | /**
// |  * The mean of `count` scores summing to `total`; zero when there are none.
// |  * @param {number} total
// |  * @param {number} count
// |  * @returns {number}
// |  */
// | export function average(total, count) {
// |   if (count <= 0) {
// |     return 0;
// |   }
// |   return total / count;
// | }
#[must_use]
pub fn average(total: f64, count: f64) -> f64 {
    if count <= 0.0 { 0.0 } else { total / count }
}

// formal-ai:translated JavaScript function items=1 sha256=de9f8845ec23bdc06833f557a7defc8f58b73b05543dc9002d245600fe009cd8
// | /**
// |  * The distance of the point (`x`, `y`) from the origin.
// |  * @param {number} x
// |  * @param {number} y
// |  * @returns {number}
// |  */
// | export function distance(x, y) {
// |   const squares = x * x + y * y;
// |   return Math.sqrt(squares);
// | }
#[must_use]
pub fn distance(x: f64, y: f64) -> f64 {
    let squares = x * x + y * y;
    f64::sqrt(squares)
}

// formal-ai:translated JavaScript function items=1 sha256=8919bb2d6c63136c732a9d6e60bff554dd53ffbc2d0236d19d96c2d1f489ef6d
// | /**
// |  * The band a score falls in.
// |  * @param {number} score
// |  * @returns {string}
// |  */
// | export function scoreBand(score) {
// |   return score >= MAXIMUM_SCORE ? 'top' : score > 5 ? 'high' : 'low';
// | }
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

// formal-ai:translated JavaScript function items=1 sha256=b1f823ddd5f63d5f99dcd8ca26ad6148b8b8deed1cec5bf3f977f1917a59c31c
// | /**
// |  * Whether `word` names the rating unit or a word built on it.
// |  * @param {string} word
// |  * @returns {boolean}
// |  */
// | export function isRatingUnit(word) {
// |   return word === RATING_UNIT || (word.startsWith('star') && word.endsWith('rs'));
// | }
#[must_use]
pub fn is_rating_unit(word: &str) -> bool {
    word == RATING_UNIT || word.starts_with("star") && word.ends_with("rs")
}

// formal-ai:translated JavaScript function items=1 sha256=6c10acf836850b5ef69a44fa8cf7cb8bd3f82ae0bd05480b9bcf450197cee311
// | /**
// |  * Whether `text` mentions the rating unit inside a longer text.
// |  * @param {string} text
// |  * @returns {boolean}
// |  */
// | export function mentionsRating(text) {
// |   return text.includes(RATING_UNIT) && text !== RATING_UNIT;
// | }
#[must_use]
pub fn mentions_rating(text: &str) -> bool {
    text.contains(RATING_UNIT) && text != RATING_UNIT
}

// formal-ai:translated JavaScript function items=1 sha256=0cd7cdf1e62ba732a2943fab1697a03323370254cbd4c074cca44b787ee6c8da
// | /**
// |  * The label of a score: its band when it is the top one.
// |  * @param {number} score
// |  * @returns {string}
// |  */
// | export function ratingLabel(score) {
// |   const band = scoreBand(score);
// |   return band === 'top' ? band : 'rated';
// | }
#[must_use]
pub fn rating_label(score: f64) -> String {
    let band = score_band(score);
    if band == "top" {
        band
    } else {
        "rated".to_string()
    }
}

// formal-ai:translated JavaScript function items=1 sha256=7dc3400800f3dda3b61471c30d42b292c098ba3270becf84a5608d26fc5230e6
// | /**
// |  * Whether a score counts as rated: shown and above zero.
// |  * @param {number} score
// |  * @param {boolean} hidden
// |  * @returns {boolean}
// |  */
// | export function isRated(score, hidden) {
// |   return !hidden && -score < 0;
// | }
#[must_use]
pub fn is_rated(score: f64, hidden: bool) -> bool {
    !hidden && -score < 0.0
}

// formal-ai:translated JavaScript function items=1 sha256=0060ee3525d39087614dc729a175dbf973cb67094f9ceff6406551b399e677b9
// | /**
// |  * How far apart two scores are.
// |  * @param {number} a
// |  * @param {number} b
// |  * @returns {number}
// |  */
// | export function spread(a, b) {
// |   return Math.abs(a - b);
// | }
#[must_use]
pub fn spread(a: f64, b: f64) -> f64 {
    f64::abs(a - b)
}

// formal-ai:translated JavaScript function items=1 sha256=959fa429c2f1ebe95fc9bf3d22e1c558ec17fc496ee9d4e5a707c49147aed7fe
// | /**
// |  * The whole stars shown for a score: odd floors round up to at most the
// |  * maximum, even ones hold.
// |  * @param {number} score
// |  * @returns {number}
// |  */
// | export function wholeStars(score) {
// |   return Math.floor(score) % 2 !== 0 ? Math.min(Math.ceil(score), MAXIMUM_SCORE) : Math.max(Math.floor(score), 1);
// | }
#[must_use]
pub fn whole_stars(score: f64) -> f64 {
    if f64::floor(score) % 2.0 == 0.0 {
        f64::max(f64::floor(score), 1.0)
    } else {
        f64::min(f64::ceil(score), MAXIMUM_SCORE)
    }
}

// The items below sit outside portable-pure-v1; each is carried with the
// construct map's reason.

// formal-ai:carried JavaScript comment (comment the target cannot hold)
// | //! An inner doc comment that no item follows.

// formal-ai:carried JavaScript class (a class is outside portable-pure-v1)
// | export class Rating {
// |   constructor(score) {
// |     this.score = score;
// |   }
// | }

// formal-ai:carried JavaScript function (a parameter or result without a JSDoc number, boolean or string type: none)
// | export function untypedScore(score) {
// |   return score;
// | }

// formal-ai:carried JavaScript function (let, var and assignment are outside portable-pure-v1)
// | /**
// |  * @param {number} score
// |  * @returns {number}
// |  */
// | export function mutableScore(score) {
// |   let doubled = score;
// |   doubled += score;
// |   return doubled;
// | }

// formal-ai:carried JavaScript function (loops are outside portable-pure-v1)
// | /**
// |  * @param {number} count
// |  * @returns {number}
// |  */
// | export function countDown(count) {
// |   while (count > 0) {
// |     return count;
// |   }
// |   return 0;
// | }

// formal-ai:carried JavaScript function (arrays and objects are outside portable-pure-v1)
// | /**
// |  * @param {number} score
// |  * @returns {number}
// |  */
// | export function scorePair(score) {
// |   return [score, score];
// | }

// formal-ai:carried JavaScript function (template literals are outside portable-pure-v1)
// | /**
// |  * @param {number} score
// |  * @returns {string}
// |  */
// | export function templateLabel(score) {
// |   return `${score} stars`;
// | }

// formal-ai:carried JavaScript function (string concatenation is outside portable-pure-v1: add of string and string)
// | /**
// |  * @param {string} word
// |  * @returns {string}
// |  */
// | export function concatenated(word) {
// |   return word + ' stars';
// | }

// formal-ai:carried JavaScript function (Math.round rounds halves up, f64::round away from zero)
// | /**
// |  * @param {number} score
// |  * @returns {number}
// |  */
// | export function roundedScore(score) {
// |   return Math.round(score);
// | }

// formal-ai:carried JavaScript function (a string length counts UTF-16 units in JavaScript and bytes in Rust)
// | /**
// |  * @param {string} word
// |  * @returns {number}
// |  */
// | export function wordLength(word) {
// |   return word.length;
// | }

// formal-ai:carried JavaScript function (a call to a function the module does not define: undefined_helper)
// | /**
// |  * @param {number} score
// |  * @returns {number}
// |  */
// | export function helperScore(score) {
// |   return undefinedHelper(score);
// | }

// formal-ai:carried JavaScript function (operand types the construct map does not accept: equal of number and string)
// | /**
// |  * @param {number} score
// |  * @returns {boolean}
// |  */
// | export function mixedTypes(score) {
// |   return score === RATING_UNIT;
// | }

// formal-ai:carried JavaScript function (a path that ends without a return)
// | /**
// |  * @param {number} score
// |  * @returns {number}
// |  */
// | export function partialScore(score) {
// |   if (score > 0) {
// |     return score;
// |   }
// | }

// formal-ai:carried JavaScript function (async functions and await are outside portable-pure-v1)
// | /**
// |  * @param {number} score
// |  * @returns {number}
// |  */
// | export async function laterScore(score) {
// |   return score;
// | }

// formal-ai:carried JavaScript function (a name with no spelling both languages read back: snake_score)
// | /**
// |  * @param {number} score
// |  * @returns {number}
// |  */
// | export function snake_score(score) {
// |   return score;
// | }

// formal-ai:carried JavaScript function (a construct outside portable-pure-v1: the operator ??)
// | /**
// |  * @param {number} score
// |  * @returns {number}
// |  */
// | export function defaultedScore(score) {
// |   return score ?? 0;
// | }

// formal-ai:carried JavaScript statement (a top-level statement runs once, as a program)
// | Object.freeze(Rating);
