// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=dbe66b6262d2489ae304d48d7034d9bc2681dae424737e4519c1e9b1004a65c6 bytes=2281
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | field access | method call .filter() | method call .map()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow function | call of a sibling function | field access | method call .filter() | method call .flatMap()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow callback of .some() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .filter() | method call .flatMap() | method call .some() | object without a $ tag
