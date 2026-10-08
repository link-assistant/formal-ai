// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=fa8f9abe925a34c99c22260ac216e0159ca06f0d18c5778df7a268d070fc2ffc bytes=6431
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=3

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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript lexical_declaration items=1 sha256=29901c80cfba3b51858a52e46abe27cf42c09641ef80b1c3630a3eb8a617f3c9
// | const INTENT_EVIDENCE_SUMMARY = 'summarization_evidence_summary';
pub const INTENT_EVIDENCE_SUMMARY: &str = "summarization_evidence_summary";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=abfd8bcb241db333698662e3f5f13456e47ee2bb5960d697d709d4e552369436
// | const INTENT_EVIDENCE_DENIED = 'summarization_evidence_denied';
pub const INTENT_EVIDENCE_DENIED: &str = "summarization_evidence_denied";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=be2a126d45e16e2297a200f6718e5fedc743e535771c245f619c789afbc5acb4
// | const INTENT_DISPUTED_STATEMENT = 'summarization_disputed_statement';
pub const INTENT_DISPUTED_STATEMENT: &str = "summarization_disputed_statement";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=78bb3fe913053cd8e2304c854e825ad71771e30e6792577cb5a964d2506884fd
// | const ASSERTED_PLACEHOLDER = '{asserted}';
pub const ASSERTED_PLACEHOLDER: &str = "{asserted}";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=05ecd4fa7b0433bf600c2c6916c3cd437b946c6c250c9ba008cf29be6b096f3e
// | const TOTAL_PLACEHOLDER = '{total}';
pub const TOTAL_PLACEHOLDER: &str = "{total}";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=1413ebf1c5bcc7036f0e531be1427f5120a22b35afa6e0f11d556da21ce0ae32
// | const DENIED_PLACEHOLDER = '{denied}';
pub const DENIED_PLACEHOLDER: &str = "{denied}";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=7ecf0e78dc65f51f6d69c3b5fcbdc52defcb176c24bd44b5c02a31627268ea62
// | const STATEMENT_PLACEHOLDER = '{statement}';
pub const STATEMENT_PLACEHOLDER: &str = "{statement}";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=71569ab9e3456ffc6731c0eccc38ddb24bf53cc54e4ad2c314be0f7e46b3c40b
// | const EVIDENCE_PLACEHOLDER = '{evidence}';
pub const EVIDENCE_PLACEHOLDER: &str = "{evidence}";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:translated JavaScript function_declaration items=1 sha256=3b2b435991ad3a3b88871967edbd931adb698b6e4c0a5e42485543ab185cf396
// | /**
// |  * Mirrors `fn percentage`: `part x 100 / whole` clamped to 100, `0` for an empty whole.
// |  * @param {number} part
// |  * @param {number} whole
// |  * @returns {number}
// |  */
// | function percentage(part, whole) {
// |   if (whole === 0) return 0;
// |   return Math.min(Math.floor((part * 100) / whole), 100);
// | }
pub fn percentage(part: f64, whole: f64) -> f64 {
    if (whole == (0f64)) {
        0f64
    } else {
        crate::ml_math::min((((part * 100f64) / whole)).floor(), 100f64)
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | call of an imported function | method call .reduce()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | arrow callback of .map() | call of a sibling function | call of an imported function | destructuring | field access | imported value | method call .find() | method call .map() | method call .push() | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | call of an imported function | field access | method call .map() | method call .sort()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers field access

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | call of an imported function | field access | global call String()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .map() | method call .slice() | object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function
