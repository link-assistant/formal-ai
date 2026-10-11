// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=ef378f4ca18932786f8a3f8848363dc095c12e82c75d885b64efa17dbf8f6652 bytes=3439
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

// meta-language:translated JavaScript lexical_declaration items=1 sha256=32f597fc36333b042636378857d6a9b3b434e35de22c107c9ff4bc7cd3e6d129
// | const MAX_REWRITE_STEPS = 100000;
pub const MAX_REWRITE_STEPS: f64 = 100000f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=9776a11d9eca8e154f16b63dbd4a9fc4acf0416bb3309d4648587aa0d0e39bc6
// | const REWRITE_FRAME = '\u0000';
pub const REWRITE_FRAME: &str = "\u{0}";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers arrow function | call of an imported function | undefined

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers Array.from | arrow function | method call .pop() | method call .slice()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers Array.from | arrow function | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | call of an imported function | method call .filter()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .every() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers arrow callback of .some() | call of a sibling function | call of an imported function | field access | method call .push() | method call .some() | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | field access | let without a value | object without a $ tag | sibling value
