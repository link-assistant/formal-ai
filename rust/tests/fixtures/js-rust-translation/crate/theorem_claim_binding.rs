// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=2c8efe4e39d63e15aa284bf6fb793f4f4afdea56a242be5f7a753d3bc0e46f36 bytes=4748
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

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

// meta-language:translated JavaScript export_statement items=1 sha256=5d8688f2bdacabf4e26abfe3975c76386d7f4965d96f39bb0f32b73b7e233493
// | /** Mirrors `PROOF_LIBRARY_PATH` in rust/src/proof_engine/claim_binding.rs. */
// | export const PROOF_LIBRARY_PATH = 'data/seed/proof-library.lino';
pub const PROOF_LIBRARY_PATH: &str = "data/seed/proof-library.lino";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Object.freeze | arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow function | call of an imported function | field access | imported value | method call .filter() | method call .find() | method call .map() | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | null | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers Array.from | arrow callback of .find() | arrow callback of .sort() | call of an imported function | imported value | method call .filter() | method call .find() | method call .join() | method call .slice() | method call .sort() | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | call of an imported function | method call .every()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of a sibling function | method call .indexOf() | method call .slice() | sibling value
