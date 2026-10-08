// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=fcc8b987029187f77e6dcfbeac8356ce2eb27ed13acce5192cb2d563ba4f5423 bytes=2801
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript export_statement items=1 sha256=268343b4f36856e8cb54f62b64056d17ec0bc8e83fee4255045529ba752c59ff
// | /** Mirrors `TITLE_MAX_LENGTH`. */
// | export const TITLE_MAX_LENGTH = 120;
pub const TITLE_MAX_LENGTH: f64 = 120f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=19a796112d3422802f0c0d3d837e8c29b5b4e0fc819a831cca549ac4d2ab773f
// | const TITLE_JOIN = '` + `';
pub const TITLE_JOIN: &str = "` + `";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow function

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow function | global value Boolean | method call .filter() | method call .split() | regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of an imported function | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | destructuring | field access | method call .filter() | method call .join() | method call .map() | method call .pop() | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers Array.from | method call .join() | method call .replace() | method call .slice() | method call .test() | regular expression
