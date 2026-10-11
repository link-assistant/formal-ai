// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=42d291bae62dfa90b20383da8ec7cef429691d84e38310c3b2b2996875853be8 bytes=8005
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript lexical_declaration items=1 sha256=cd930b571097477efb6ed71acb6243b0fbd80a0c79387dafc5d42b2eca8b16e9
// | const LINK_SEPARATOR = '\u0000';
pub const LINK_SEPARATOR: &str = "\u{0}";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | arrow callback of .some() | arrow callback of .sort() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | class | destructuring | export … | field access | imported value | method call .add() | method call .associate() | method call .contains() | method call .entries() | method call .expressions() | method call .filter() | method call .get() | method call .has() | method call .inDegree() | method call .links() | method call .map() | method call .noteRead() | method call .outDegree() | method call .persistIdentified() | method call .push() | method call .retentionScoreMap() | method call .retentionScores() | method call .set() | method call .shift() | method call .slice() | method call .some() | method call .sort() | method call .split() | new AssociativeMemory | new Map | new Set | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers method call .set() | null | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .entries()
// formal-ai:blockers arrow callback of .sort() | call of an imported function | field access | method call .entries() | method call .sort()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | field access | method call .concat() | method call .filter() | method call .join() | null | undefined
