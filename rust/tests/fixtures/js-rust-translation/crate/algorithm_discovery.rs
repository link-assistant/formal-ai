// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=add5df7f1a6dff5f0b8816095d737d02ec9b4e56819593b1199338ea5209da18 bytes=31584
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ea2291a713bdc97af4de7987115368aaf67d29e42a2c819b48ed5e898f01700a
// | const DEFAULT_MIN_STEPS = 2;
pub const DEFAULT_MIN_STEPS: f64 = 2f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=82aec5708d7e17cbdf48272c8c1d9dd8d3095214c883183a48490b235d03c330
// | const DEFAULT_SUPPORT_OCCURRENCES = 2;
pub const DEFAULT_SUPPORT_OCCURRENCES: f64 = 2f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=06d81550f679cb4e998e0a84f98b9d53b88ba30361a84a023f02c350529a5655
// | const DEFAULT_HELD_OUT_OCCURRENCES = 1;
pub const DEFAULT_HELD_OUT_OCCURRENCES: f64 = 1f64;

// meta-language:translated JavaScript export_statement items=1 sha256=b975878b60096ed668d0a902c836c3b4722f8423d62c9b57b1dc022095499ae3
// | /** Mirrors `MAX_DISCOVERY_INPUT_STEPS` in rust/src/algorithm_discovery.rs. */
// | export const MAX_DISCOVERY_INPUT_STEPS = 4096;
pub const MAX_DISCOVERY_INPUT_STEPS: f64 = 4096f64;

// meta-language:translated JavaScript export_statement items=1 sha256=480f3d3ac027acde48cedc366faad56c6ed830001ba3ca22ed9b97e336cca4de
// | /** Mirrors `MAX_DISCOVERED_ALGORITHM_STEPS` in rust/src/algorithm_discovery.rs. */
// | export const MAX_DISCOVERED_ALGORITHM_STEPS = 32;
pub const MAX_DISCOVERED_ALGORITHM_STEPS: f64 = 32f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=cafa27b6be30decb0a662c2a5df11f2a87a1fbb696ca77f45a0f3fe6a30f6a8b
// | /**
// |  * Rust `Ord::cmp` for integers.
// |  * @param {number} left
// |  * @param {number} right
// |  * @returns {number}
// |  */
// | const cmpNum = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
pub fn cmp_num(left: f64, right: f64) -> f64 {
    if (left < right) {
        (-1f64)
    } else {
        if (left > right) {
            1f64
        } else {
            0f64
        }
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unsupported string escape \b

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Object.is

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal BigInt

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSON.stringify

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .entries()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal for with const

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .find()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .every()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript class_declaration (unsupported)
// formal-ai:refusal top-level class statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal throw new …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal throw new …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
