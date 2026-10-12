// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=fb478c65c1999e73da65c567d592e27744a2c8c388489ead440061c72baf14d5 bytes=4384
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8ae9b7470652ebb303eb11b75c0809021d4ffb7cee3ae4ab124df3da48a1b304
// | const NUMBER_PLACEHOLDER = '%number%';
pub const NUMBER_PLACEHOLDER: &str = "%number%";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8dc6be7a9d29c2045eb5487005a0685caf80788e6c0501b3ac7c9f9b05455944
// | const LINEAGE_INTENT = 'repository_lineage';
pub const LINEAGE_INTENT: &str = "repository_lineage";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=6599a6c36c5c8147061fa895e5d91316cb3f2f7c179a6b0217c68a7000ccd8d1
// | const UNRECORDED_INTENT = 'repository_lineage_unrecorded';
pub const UNRECORDED_INTENT: &str = "repository_lineage_unrecorded";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .some() | call of a sibling function | field access | method call .push() | method call .some() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | method call .slice() | method call .test() | null | regular expression | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of an imported function | destructuring | method call .join() | method call .split() | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | arrow callback of .map() | arrow function | call of a sibling function | field access | global call String() | method call .find() | method call .join() | method call .map() | method call .slice() | method call .split() | null | nullish coalescing | object without a $ tag | optional chaining
