// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=467ade3ae3359bbe0be421f7425eac0e7a1482978064a2a23f22f86676cba338 bytes=1255
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:translated JavaScript import_statement items=1 sha256=ea67ac58c941251801bc82c22574a8e03c0fd9db18bf89015315cef5823b17a3
// | import { lower as canonicalUnaryLower } from './rust_str.mjs';
use crate::rust_str::lower as canonical_unary_lower;

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .flatMap() | arrow callback of .map() | arrow function | call of an imported function | field access | method call .filter() | method call .find() | method call .flatMap() | method call .map() | object without a $ tag
