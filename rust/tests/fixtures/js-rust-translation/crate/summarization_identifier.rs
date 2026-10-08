// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=83011714f20e61cfd3a326e062acf85ac594ff9a59844c28597ecc603f1b1bcd bytes=6325
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=9246fa1727f5fc4d70961741d06d975d2cc0977dc260c4661b50581346c89331
// | /** Mirrors `DEFAULT_IDENTIFIER_MAX_LENGTH` in rust/src/summarization/identifier.rs. */
// | export const DEFAULT_IDENTIFIER_MAX_LENGTH = 40;
pub const DEFAULT_IDENTIFIER_MAX_LENGTH: f64 = 40f64;

// meta-language:translated JavaScript export_statement items=1 sha256=f7295af9cfee94620d944eac1679fd57a7ceb6792117fa38dc7db67217b1e92c
// | /** Mirrors `DEFAULT_IDENTIFIER_MAX_WORDS` in rust/src/summarization/identifier.rs. */
// | export const DEFAULT_IDENTIFIER_MAX_WORDS = 4;
pub const DEFAULT_IDENTIFIER_MAX_WORDS: f64 = 4f64;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | field access

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

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .join() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers arrow callback of .map() | call of a sibling function | method call .join() | method call .map() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers Array.from | call of a sibling function | call of an imported function | method call .replace() | method call .test() | regular expression | sibling value | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .join() | method call .pop() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | field access | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | call of a sibling function | call of an imported function | method call .filter() | method call .join() | method call .map() | method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers Array.from | arrow callback of .every() | arrow callback of .some() | call of an imported function | method call .every() | method call .replace() | method call .some() | method call .test() | regular expression | sibling value
