// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=77449ee3ed15b6326d66d01df79315aa79d56ffeb1492ccd2a3c636e90b3d21a bytes=5940
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=ebfb9344535898a2a5adcde2cfa6613a808a2755c61ba3f481370c8c56f05427
// | import { previousChar, quotePairs as PAIRS, quotedSegmentSpans } from './quoted_segments.mjs';
use crate::quoted_segments::quotePairs as PAIRS;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers export … | imported value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of an imported function | field access | method call .indexOf() | method call .push() | method call .slice() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers call of an imported function | imported value | method call .map()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8d11aa6be05c35a0b3967fa8a92c41dd426df899501f42aa9f4e345431a26952
// | const ASCII_QUOTES = ["'", '"', '`'];
pub static ASCII_QUOTES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("'"), String::from("\""), String::from("`")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b5466aab9ffb8a8cb91a1e17b8e2d836f980f7e5f1105ccaf5bb57735d02a874
// | const OPEN_ONLY = ['«', '“', '‘', '「', '『', '《'];
pub static OPEN_ONLY: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("«"), String::from("“"), String::from("‘"), String::from("「"), String::from("『"), String::from("《")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f6532f03f769e557f27ad4e3d8275f2b265f98bf2c1d44a2df852c946511ab6c
// | const CLOSE_FOLLOWERS = '.,;:!?)]}';
pub const CLOSE_FOLLOWERS: &str = ".,;:!?)]}";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=14f87ced3772ddb3bb74b94e42e98a4cfbd3f9e543f02e739fa50e02884883bb
// | const FAULT_FRAGMENT_CHARS = 32;
pub const FAULT_FRAGMENT_CHARS: f64 = 32f64;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers ASCII_QUOTES.includes | Array.from | CLOSE_FOLLOWERS.includes | OPEN_ONLY.includes | arrow callback of .reduce() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .join() | method call .push() | method call .reduce() | method call .slice() | method call .test() | null | object without a $ tag | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of an imported function | method call .test() | regular expression | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of an imported function | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers PAIRS.some | arrow callback of .some() | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers PAIRS.some | arrow callback of .some()
