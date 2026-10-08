// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=9d4ce1a89f1d7c8c352a185f3603e3fc22dbb8ab9bc82b5ee78075c7bc125790 bytes=25827
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=3

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
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

// meta-language:translated JavaScript export_statement items=1 sha256=61755fe9b68d6b81a39d1870daf32e206349e9687ee533cb023372f863678ad9
// | /** Mirrors `MAX_META_LANGUAGE_PARSE_BYTES` in rust/src/summarization/file.rs. */
// | export const MAX_META_LANGUAGE_PARSE_BYTES = 32 * 1024;
pub static MAX_META_LANGUAGE_PARSE_BYTES: std::sync::LazyLock<f64> = std::sync::LazyLock::new(|| (32f64 * 1024f64));

// meta-language:translated JavaScript export_statement items=1 sha256=7b49067fe390ee4ca5ffbc3d9073973b035cf4661975b01fcd87489123bf9e6b
// | /** Mirrors `MAX_PLAIN_TEXT_FORMALIZATION_BYTES` in rust/src/summarization/file.rs. */
// | export const MAX_PLAIN_TEXT_FORMALIZATION_BYTES = 32 * 1024;
pub static MAX_PLAIN_TEXT_FORMALIZATION_BYTES: std::sync::LazyLock<f64> = std::sync::LazyLock::new(|| (32f64 * 1024f64));

// meta-language:translated JavaScript export_statement items=1 sha256=7b6e63aa2306fce8f0b9a8135a781077db9fa36ee778a7f25825a867328b6053
// | /** Mirrors `MAX_PLAIN_TEXT_STATEMENTS` in rust/src/summarization/file.rs. */
// | export const MAX_PLAIN_TEXT_STATEMENTS = 256;
pub const MAX_PLAIN_TEXT_STATEMENTS: f64 = 256f64;

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextEncoder

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextDecoder

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers method call .encode() | method call .parser() | null | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | imported value | method call .encode() | method call .push() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | call of an imported function | field access | method call .join() | method call .push() | null | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access | global call String() | method call .replace() | null | sibling value | undefined

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | call of an imported function | method call .concat() | method call .encode() | method call .slice() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal bitwise &
// formal-ai:blockers bitwise operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .subarray()
// formal-ai:blockers call of a sibling function | method call .decode() | method call .subarray() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .subarray()
// formal-ai:blockers call of a sibling function | method call .decode() | method call .subarray() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal assignment
// formal-ai:blockers assignment of a field or element | call of a sibling function | call of an imported function | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | call of an imported function | imported value | method call .push() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | call of an imported function | imported value | method call .join() | method call .push() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .push() | method call .replace() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | method call .replace() | method call .slice() | null | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.from | call of a sibling function | field access | method call .join() | method call .replace() | method call .slice() | method call .test() | null | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow callback of .replace() | call of a sibling function | method call .replace() | nullish coalescing | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .filter() | method call .filter() | method call .split() | null | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .lastIndexOf()
// formal-ai:blockers method call .lastIndexOf() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow callback of .replace() | method call .replace() | regular expression | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers null

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: one value is used as an array and as a string; declare the types of the function with JSDoc

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: one value is used as an array and as a string; declare the types of the function with JSDoc

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal case test

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .replace() | null | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers call of a sibling function | null | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers call of a sibling function | destructuring | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | global value Boolean | method call .filter() | method call .indexOf() | method call .replace() | method call .slice() | method call .split() | null | regular expression | sibling value | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | call of a sibling function | method call .join() | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers Array.from | method call .replace() | method call .slice() | method call .test() | sibling value | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .replace() | null | nullish coalescing | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers method call .indexOf() | method call .replace() | method call .slice() | null | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers Array.from | arrow callback of .every() | call of a sibling function | method call .every() | method call .indexOf() | method call .replace() | method call .slice() | null | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers call of a sibling function | field access | method call .join()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers method call .push()

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of a sibling function | global value Boolean | method call .filter() | method call .split() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .pop() | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .every() | call of a sibling function | method call .every() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .repeat()
// formal-ai:blockers call of a sibling function | field access | global call String() | method call .repeat()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .repeat()
// formal-ai:blockers call of an imported function | method call .repeat()
