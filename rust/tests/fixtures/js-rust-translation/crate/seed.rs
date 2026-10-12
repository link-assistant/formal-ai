// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=b2cb566d102dda4f74a7704c2adcfad5f9b8b2932c901dcdeedf1fb1e388a4f9 bytes=5195
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

// meta-language:translated JavaScript lexical_declaration items=1 sha256=2d9b8c1bb4b4437bcb320c603c2f802a000e40fa9092307c59f02af758705f8f
// | /** The one inventory of the seed files and the lexicons that read them. */
// | const SEED_REGISTRY = 'data/meta/seed-registry.lino';
pub const SEED_REGISTRY: &str = "data/meta/seed-registry.lino";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow callback of .some() | arrow function | call of an imported function | field access | method call .filter() | method call .map() | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .map() | method call .push() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers call of a sibling function | field access | null | nullish coalescing | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers bitwise operator | call of a sibling function | field access | method call .encode() | new TextEncoder | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .reduce() | call of a sibling function | method call .join() | method call .reduce() | method call .split() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | global value Boolean | method call .filter() | method call .map() | method call .slice() | method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .push() | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of an imported function | field access | method call .map()
