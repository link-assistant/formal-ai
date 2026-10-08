// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=b306e932d64a5554aabf3148e31d2f646f312c442f3cb954c6495798b26347b7 bytes=8453
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

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
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=97bf385d8aa8614fd3f7e0346cdd3f0a8e79365dca37ad469ea3165c43775273
// | /** Mirrors `ELABORATION_SHARED_TERMS` in rust/src/summarization/dependency.rs: shared terms that make one statement elaborate another. */
// | export const ELABORATION_SHARED_TERMS = 2;
pub const ELABORATION_SHARED_TERMS: f64 = 2f64;

// meta-language:translated JavaScript export_statement items=1 sha256=45d8a68bddb1fcfc6013225baac9828732e8c8a9d4641df59b15a9e0b0ec531c
// | /** Mirrors `KEEP_DIVISOR` in rust/src/summarization/dependency.rs: the summary keeps at most one statement in this many. */
// | export const KEEP_DIVISOR = 3;
pub const KEEP_DIVISOR: f64 = 3f64;

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .pop() | method call .test() | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .forEach()
// formal-ai:blockers arrow callback of .forEach() | call of a sibling function | call of an imported function | field access | method call .forEach() | method call .push() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .every() | method call .every() | method call .has() | new Set

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | field access | method call .push() | method call .some() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .filter() | call of an imported function | method call .filter() | method call .has() | new Set

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .forEach() | arrow callback of .map() | call of a sibling function | field access | method call .forEach() | method call .map() | method call .push() | object spread | object without a $ tag

// meta-language:translated JavaScript export_statement items=1 sha256=fbc8d7234d0341fa0b77cddeb0b75bd2ef71e9575c0a0b91b54fd75ce2ea8c36
// | /** Mirrors `fn keep_budget` in rust/src/summarization/dependency.rs: one statement in `KEEP_DIVISOR`, at least one. @param {number} count @returns {number} */
// | export function keepBudget(count) {
// |   return count === 0 ? 0 : Math.max(1, Math.ceil(count / KEEP_DIVISOR));
// | }
pub fn keep_budget(count: f64) -> f64 {
    if (count == (0f64)) {
        0f64
    } else {
        crate::ml_math::max(1f64, ((count / KEEP_DIVISOR)).ceil())
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow callback of .sort() | arrow function | field access | method call .filter() | method call .map() | method call .slice() | method call .sort()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of an imported function | field access | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | destructuring | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | call of a sibling function | call of an imported function | field access | imported value | method call .filter() | method call .flatMap() | method call .sort() | new Set

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | call of an imported function | field access | method call .filter() | method call .flatMap() | method call .has() | new Set
