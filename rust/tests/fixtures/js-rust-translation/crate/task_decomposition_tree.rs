// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=e8d592616362c163fae74c116531746f3d367f230358622e51fed4d847ddbf45 bytes=7637
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=3

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]

/// JavaScript's Number operations that Rust's f64 does not share.
pub mod ml_number {
    /// ECMAScript Number::toString: the shortest decimal that reads back as
    /// the value, the even one on a tie, laid out as JavaScript prints it.
    pub fn js_number(x: f64) -> String {
        if x.is_nan() {
            return String::from("NaN");
        }
        if x.is_infinite() {
            return String::from(if x > 0.0 { "Infinity" } else { "-Infinity" });
        }
        if x == 0.0 {
            return String::from("0");
        }
        let (mut digits, n) = decimal(&format!("{:e}", x.abs()));
        // Rust's shortest digits round a tie away from zero; ECMAScript takes the even neighbour.
        let (exact, _) = decimal(&format!("{:.800e}", x.abs()));
        let rest = &exact[digits.len().min(exact.len())..];
        if digits.len() > 1 && rest.starts_with('5') && rest[1..].bytes().all(|b| b == b'0') {
            let last = digits.as_bytes()[digits.len() - 1] - b'0';
            let down = exact[..digits.len()].to_string();
            let other = if digits == down { increment(&down) } else { Some(down) };
            if let Some(other) = other {
                let back = format!("{}.{}e{}", &other[..1], &other[1..], n - 1);
                if last % 2 == 1 && back.parse::<f64>() == Ok(x.abs()) {
                    digits = other.trim_end_matches('0').to_string();
                }
            }
        }
        let k = digits.len() as i32;
        let body = if k <= n && n <= 21 {
            format!("{digits}{}", "0".repeat((n - k) as usize))
        } else if 0 < n && n <= 21 {
            format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
        } else if -6 < n && n <= 0 {
            format!("0.{}{digits}", "0".repeat((-n) as usize))
        } else {
            let sign = if n - 1 < 0 { '-' } else { '+' };
            let power = (n - 1).abs();
            if k == 1 {
                format!("{digits}e{sign}{power}")
            } else {
                format!("{}.{}e{sign}{power}", &digits[..1], &digits[1..])
            }
        };
        if x < 0.0 {
            format!("-{body}")
        } else {
            body
        }
    }

    /// What console.log prints: -0 as "-0", where String(-0) is "0".
    pub fn js_console(x: f64) -> String {
        if x == 0.0 && x.is_sign_negative() {
            String::from("-0")
        } else {
            js_number(x)
        }
    }

    /// SameValue, as Object.is and assert.strictEqual compare: NaN equals
    /// NaN, and 0 differs from -0.
    pub fn same_value(a: f64, b: f64) -> bool {
        if a.is_nan() || b.is_nan() {
            a.is_nan() && b.is_nan()
        } else {
            a.to_bits() == b.to_bits()
        }
    }

    /// A decimal digit string plus one, or None when it gains a digit.
    fn increment(digits: &str) -> Option<String> {
        let mut bytes = digits.as_bytes().to_vec();
        for index in (0..bytes.len()).rev() {
            if bytes[index] == b'9' {
                bytes[index] = b'0';
            } else {
                bytes[index] += 1;
                return String::from_utf8(bytes).ok();
            }
        }
        None
    }

    /// The significant digits without trailing zeros, and the exponent n with
    /// value 0.digits × 10^n, of Rust's {:e} text.
    fn decimal(text: &str) -> (String, i32) {
        let (mantissa, exponent) = text.split_once('e').expect("exponent");
        let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
        let digits = digits.trim_end_matches('0').to_string();
        let digits = if digits.is_empty() { String::from("0") } else { digits };
        (digits, exponent.parse::<i32>().expect("exponent") + 1)
    }
}
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript function_declaration items=1 sha256=1d38dea5082b05fa03787097a0e13e0bde40eb19282f077bd769083004b3ca15
// | /**
// |  * Mirrors `fn child_path` in rust/src/task_decomposition.rs: `1`, `2`, ... under the root; `1.1`, ... under a sub-task.
// |  * @param {string} parent
// |  * @param {number} index
// |  * @returns {string}
// |  */
// | function childPath(parent, index) {
// |   const number = index + 1;
// |   return parent === '' ? String(number) : `${parent}.${number}`;
// | }
pub fn child_path(parent: String, index: f64) -> String {
    {
        let number = (index + 1f64);
        if (parent == "") {
            crate::ml_number::js_number(number)
        } else {
            format!("{}{}", (format!("{}{}", parent, (String::from(".")))), (crate::ml_number::js_number(number)))
        }
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | call of an imported function | object without a $ tag

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers field access | method call .push()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers field access | method call .map() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of an imported function | field access | global call String() | method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers call of an imported function | global call String() | method call .join()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers call of a sibling function | call of an imported function | field access | global call String() | method call .join()

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unsupported template escape
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | arrow function | call of a sibling function | field access | method call .add() | method call .attempt() | method call .extend_for() | method call .filter() | method call .has() | method call .map() | method call .push() | method call .retry_after_children() | new Set | object without a $ tag | sibling value
