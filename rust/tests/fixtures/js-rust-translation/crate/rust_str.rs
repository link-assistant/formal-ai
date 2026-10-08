// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=b6653a850adc56e8cf192eb97d89d87b772cf54a6a61e85d892af246ff9bcfd4 bytes=8890
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]

/// The Math functions that Rust's f64 methods do not share with JavaScript.
pub mod ml_math {
    /// Math.round: the nearest integer, the one towards +Infinity on a tie,
    /// with the sign of a zero result from the argument.
    pub fn round(x: f64) -> f64 {
        if !x.is_finite() || x == 0.0 {
            return x;
        }
        if x > 0.0 && x < 0.5 {
            return 0.0;
        }
        if x < 0.0 && x >= -0.5 {
            return -0.0;
        }
        let floor = x.floor();
        if x - floor >= 0.5 {
            floor + 1.0
        } else {
            floor
        }
    }

    /// Math.sign, which keeps -0, 0 and NaN, where f64::signum does not.
    pub fn sign(x: f64) -> f64 {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            x
        }
    }

    /// Math.max of two Numbers: NaN when either is, and 0 above -0.
    pub fn max(a: f64, b: f64) -> f64 {
        if a > b {
            a
        } else if b > a {
            b
        } else if a == b {
            if a.is_sign_negative() {
                b
            } else {
                a
            }
        } else {
            f64::NAN
        }
    }

    /// Math.min of two Numbers: NaN when either is, and -0 below 0.
    pub fn min(a: f64, b: f64) -> f64 {
        if a < b {
            a
        } else if b < a {
            b
        } else if a == b {
            if a.is_sign_negative() {
                a
            } else {
                b
            }
        } else {
            f64::NAN
        }
    }

    pub fn max_of(values: &[f64]) -> f64 {
        values.iter().fold(f64::NEG_INFINITY, |a, &b| max(a, b))
    }

    pub fn min_of(values: &[f64]) -> f64 {
        values.iter().fold(f64::INFINITY, |a, &b| min(a, b))
    }

    pub fn is_integer(x: f64) -> bool {
        x.is_finite() && x.trunc() == x
    }

    pub fn is_safe_integer(x: f64) -> bool {
        is_integer(x) && x.abs() <= 9007199254740991.0
    }
}

/// Reads of JavaScript arrays, which the portable core never mutates.
pub mod ml_array {
    /// The element at an index; a read outside the array, undefined in
    /// JavaScript, aborts.
    pub fn at<T: Clone>(values: &[T], index: Option<usize>) -> T {
        match index.and_then(|index| values.get(index)) {
            Some(value) => value.clone(),
            None => panic!("array index out of range"),
        }
    }

    /// The index a Number names: a non-negative integer, -0 included.
    pub fn number_index(index: f64) -> Option<usize> {
        if index >= 0.0 && index.fract() == 0.0 && index < 9007199254740992.0 {
            Some(index as usize)
        } else {
            None
        }
    }

    pub fn append<T>(mut left: Vec<T>, right: Vec<T>) -> Vec<T> {
        left.extend(right);
        left
    }
}
// meta-language:prelude end

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | method call .test() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated template literal
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow function | method call .replace() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow function | method call .replace() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow function | method call .replace() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow function | global value Boolean | method call .filter() | method call .split() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow function | method call .encode() | new TextEncoder

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow callback of .replace() | arrow function | method call .replace() | regular expression

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .join() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .join() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .join() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .lastIndexOf()
// formal-ai:blockers method call .lastIndexOf() | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers arrow function | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers arrow function | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow function | method call .join() | method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | method call .push() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers JSON.stringify | call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSON.stringify
// formal-ai:blockers JSON.stringify | call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.isArray
// formal-ai:blockers Array.isArray | Object.keys | assignment of a field or element | method call .map() | method call .sort() | object without a $ tag | sibling value | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .from() | method call .codePointAt()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers JSON.parse | try statement | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers Array.isArray | arrow function | global call Boolean() | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Object.prototype.hasOwnProperty.call
// formal-ai:blockers Object.prototype | arrow function | call of a sibling function | field access | method call .call() | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers arrow function | null | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers arrow function | null | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .test() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .test() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers arrow function | call of a sibling function | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of a sibling function | method call .indexOf()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | let without a value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | let without a value | undefined

// meta-language:translated JavaScript export_statement items=3 sha256=dbc15d06aab1cdbac5f23e0b3658b091d44b9459ffd10edf22b05aa018a4c271
// | /**
// |  * Lexicographic comparison of numeric tuples (Rust tuple `Ord`).
// |  * @param {number[]} left
// |  * @param {number[]} right
// |  * @returns {number}
// |  */
// | export function compareTuples(left, right) {
// |   for (let index = 0; index < Math.min(left.length, right.length); index += 1) {
// |     if (left[index] !== right[index]) return left[index] < right[index] ? -1 : 1;
// |   }
// |   return left.length - right.length;
// | }
pub fn compare_tuples(left: Vec<f64>, right: Vec<f64>) -> f64 {
    {
        let index = 0f64;
        {
            let ml_s1 = crate::ml_compare_tuples_loop1(left.clone(), right.clone(), index);
            match ml_s1.clone() {
                crate::MlCompareTuplesLoop1Result::MlCompareTuplesLoop1Done => {
                    ((left.len() as f64) - (right.len() as f64))
                }
                crate::MlCompareTuplesLoop1Result::MlCompareTuplesLoop1Return(ml_result1) => {
                    ml_result1
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MlCompareTuplesLoop1Result {
    MlCompareTuplesLoop1Done,
    MlCompareTuplesLoop1Return(f64),
}

pub fn ml_compare_tuples_loop1(mut left: Vec<f64>, mut right: Vec<f64>, mut index: f64) -> crate::MlCompareTuplesLoop1Result {
    loop {
        return if (index < (crate::ml_math::min((left.len() as f64), (right.len() as f64)))) {
            if ((crate::ml_array::at(&left, crate::ml_array::number_index(index))) != (crate::ml_array::at(&right, crate::ml_array::number_index(index)))) {
                crate::MlCompareTuplesLoop1Result::MlCompareTuplesLoop1Return(if ((crate::ml_array::at(&left, crate::ml_array::number_index(index))) < (crate::ml_array::at(&right, crate::ml_array::number_index(index)))) {
                    (-1f64)
                } else {
                    1f64
                })
            } else {
                {
                    let index_2 = (index + 1f64);
                    {
                        index = index_2;
                        continue;
                    }
                }
            }
        } else {
            crate::MlCompareTuplesLoop1Result::MlCompareTuplesLoop1Done
        };
    }
}
