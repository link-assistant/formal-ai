// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=7e82e6c04d4ce30f8478397b80eed6d903045655130898b8fa8a8e9da9e2f234 bytes=6465
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript export_statement items=1 sha256=1bb4cbeebc085918297f8259e3d67dccf9691e3d097695d82347c34fc64b34e8
// | /** Mirrors `SENTENCE_PUNCTUATION_LINO` (the repository path it embeds). */
// | export const SENTENCE_PUNCTUATION_PATH = 'data/seed/sentence-punctuation.lino';
pub const SENTENCE_PUNCTUATION_PATH: &str = "data/seed/sentence-punctuation.lino";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers Object.values | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers arrow function | global call String() | method call .replace() | nullish coalescing | regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow function | undefined

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers global call parseInt() | method call .indexOf() | method call .slice() | method call .test() | null | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .filter() | method call .find() | method call .map() | null | nullish coalescing | object without a $ tag | optional chaining | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers arrow callback of .find() | arrow callback of .some() | call of a sibling function | field access | method call .codePointAt() | method call .find() | method call .some() | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | arrow function | call of a sibling function | field access | method call .some()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | arrow function | call of a sibling function | field access | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .find() | call of a sibling function | method call .find() | method call .push() | null | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of an imported function | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers method call .decode() | method call .encode() | method call .slice() | new TextDecoder | new TextEncoder

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .forEach() | call of a sibling function | call of an imported function | method call .forEach() | null | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .flatMap() | call of a sibling function | call of an imported function | destructuring | field access | method call .filter() | method call .flatMap() | method call .push() | null | object spread | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .push() | object without a $ tag
