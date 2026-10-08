// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=e923afcd43fb5b15ea7ba1fa55ff439e60ead127804dd4adbb34287009ce43ab bytes=4699
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=6c10e7c2e03d5a298a55b7cbb759d7e3b7130defdced0ec19e3a583f5b7e5a31
// | /** Mirrors `ANSWER_STATEMENT_LIMIT` in rust/src/formalization/page.rs: the most statements a page answer lists. */
// | export const ANSWER_STATEMENT_LIMIT = 40;
pub const ANSWER_STATEMENT_LIMIT: f64 = 40f64;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .every() | call of an imported function | field access | method call .every() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .flatMap() | call of a sibling function | call of an imported function | field access | imported value | method call .flatMap() | method call .push() | method call .some() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of an imported function | field access | imported value | method call .join() | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .flatMap() | call of a sibling function | field access | method call .flatMap() | method call .join() | method call .push() | method call .slice() | object without a $ tag
