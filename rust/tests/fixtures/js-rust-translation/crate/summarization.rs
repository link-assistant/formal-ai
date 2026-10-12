// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=981ace44aa9562e35df5dfb610784ce0279143d709f4203175861d1af2ab37c2 bytes=17890
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=0d98de4e8bb33da7a42f6b37345ac4c6d76c5503038e537b267f5de870cb9147
// | /** Mirrors `DEFAULT_MAX_STATEMENTS` in rust/src/summarization/mod.rs. */
// | export const DEFAULT_MAX_STATEMENTS = 30;
pub const DEFAULT_MAX_STATEMENTS: f64 = 30f64;

// meta-language:translated JavaScript export_statement items=1 sha256=047852f5e5de8e4c86d5d4ffe0a0270dc03dc5d3a95ac896c7cae053210db8d9
// | /** Mirrors `ROLE_SUMMARY_CLASSIFICATION_CUE` in rust/src/seed/roles/tooling.rs. */
// | export const ROLE_SUMMARY_CLASSIFICATION_CUE = 'summary_classification_cue';
pub const ROLE_SUMMARY_CLASSIFICATION_CUE: &str = "summary_classification_cue";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow callback of .replace() | method call .replace() | regular expression | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of an imported function | imported value | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | null | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread
// formal-ai:blockers object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread
// formal-ai:blockers object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread
// formal-ai:blockers object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread
// formal-ai:blockers object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access | let without a value | null | sibling value | undefined

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow function | global value Boolean | method call .filter() | method call .join() | method call .slice() | method call .split() | method call .test() | nullish coalescing | object without a $ tag | regular expression | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .map() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | field access | global value Boolean | method call .filter() | method call .some() | method call .split() | regular expression | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replaceAll()
// formal-ai:blockers call of a sibling function | method call .push() | method call .replace() | method call .replaceAll() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.from | JSDoc type {…} | call of a sibling function | method call .has() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | arrow callback of .sort() | assignment of a field or element | call of a sibling function | field access | method call .filter() | method call .map() | method call .push() | method call .sort() | object spread | object without a $ tag | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.from | JSDoc type {…} | field access | method call .has() | method call .join() | method call .pop() | method call .push() | method call .replace() | regular expression | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers Array.from | global value Boolean | method call .filter() | method call .has() | method call .join() | method call .pop() | method call .slice() | method call .split() | regular expression | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access | method call .replace() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | call of an imported function | method call .map() | method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers call of a sibling function | destructuring | method call .join() | method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers call of a sibling function | destructuring | method call .join() | method call .split()
