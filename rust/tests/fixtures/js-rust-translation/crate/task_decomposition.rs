// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1800c7825e0b339afca324c5005c7321053ecb970c4e7d7e9036712717f1427d bytes=11516
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=6

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

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=3b39d68dbe60936401cdc1913220f9edb9db005077dea0b4c98f20a2e9b8a1f1
// | /** Mirrors `DEFAULT_SPLIT_DEPTH_BOUND` in rust/src/recursive_execution.rs. */
// | export const DEFAULT_SPLIT_DEPTH_BOUND = 4;
pub const DEFAULT_SPLIT_DEPTH_BOUND: f64 = 4f64;

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of an imported function | field access | method call .find() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .filter() | method call .map() | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .every() | arrow callback of .find() | arrow function | call of an imported function | field access | method call .every() | method call .find() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .every() | arrow callback of .filter() | arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .every() | method call .filter() | method call .map() | method call .test() | new Set | null | nullish coalescing | object without a $ tag | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .find() | method call .has() | method call .join() | method call .map() | method call .split() | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .some() | null | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .every() | arrow function | call of a sibling function | call of an imported function | method call .every() | method call .lastIndexOf() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow function | call of an imported function | imported value | method call .every() | method call .slice() | method call .split() | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | null | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal array destructuring
// formal-ai:blockers arrow callback of .map() | assignment of a field or element | call of a sibling function | destructuring | method call .map()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f4586b0ff115aa5c9d24a8580c22c6dab6a6cf202f97743c0600387770dd1552
// | /** Mirrors `LITERAL_MASK_BASE` and `LITERAL_MASK_SLOTS`. */
// | const LITERAL_MASK_BASE = 0xe000;
pub const LITERAL_MASK_BASE: f64 = 57344f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=7cf6955f64581ee75aca0a32cf9941877548f8de5685517227ccb1b19035ea3b
// | const LITERAL_MASK_SLOTS = 0x1900;
pub const LITERAL_MASK_SLOTS: f64 = 6400f64;

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers arrow function | method call .codePointAt() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | String.fromCodePoint | arrow callback of .some() | call of a sibling function | call of an imported function | field access | method call .push() | method call .slice() | method call .some() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .forEach() | call of a sibling function | method call .forEach() | method call .test() | object without a $ tag | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .split() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .flatMap() | call of a sibling function | method call .flatMap()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow callback of .replace() | arrow function | method call .replace() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | imported value | method call .join() | method call .map() | method call .push() | method call .some() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .map() | call of a sibling function | call of an imported function | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | null | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of a sibling function | method call .push() | null

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of an imported function | method call .push()
