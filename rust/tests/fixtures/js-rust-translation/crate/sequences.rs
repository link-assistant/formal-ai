// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1cd32b7ef9e7d2c36d82be926f297b5812b0df8027a92f7bba914c151c18f6e7 bytes=5474
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:translated JavaScript export_statement items=1 sha256=ead057e80df0822470f21b6eaf17f83e9fbb5214ef8c2590251bd513f7ba938a
// | /** Mirrors `NULL_LINK` in rust/src/sequences/store.rs. */
// | export const NULL_LINK = 0;
pub const NULL_LINK: f64 = 0f64;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers arrow function | assignment of a field or element | class | export … | field access | method call .get() | method call .push() | method call .set() | new Map | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers assignment of a field or element | class | export … | field access | method call .createPoint() | method call .get() | method call .has() | method call .set() | new Map

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers method call .getOrCreate() | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | method call .getOrCreate()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal destructured parameter

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers call of a sibling function | field access | method call .set() | method call .values() | new Map | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal destructured parameter
// formal-ai:blockers method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers arrow callback of .every() | arrow callback of .flatMap() | call of a sibling function | field access | method call .every() | method call .expand() | method call .flatMap() | method call .getOrCreate() | method call .push() | method call .slice() | object without a $ tag
