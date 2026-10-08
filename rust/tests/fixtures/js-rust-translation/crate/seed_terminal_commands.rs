// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=888fc1bc8929bd8ee1319501dc6df265e8b25e0bbbc63e48074e3aee695e543a bytes=1804
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

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | field access | method call .filter() | method call .map()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow function | call of a sibling function | field access | method call .filter() | method call .flatMap()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | assignment of a field or element | call of a sibling function | call of an imported function | field access | object without a $ tag
