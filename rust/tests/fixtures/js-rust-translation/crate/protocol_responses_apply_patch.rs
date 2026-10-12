// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=bf47027e26ccb56e6d3f668170ea95d7ebff6b59b40d0fa6684670b05d4e0956 bytes=1366
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:translated JavaScript lexical_declaration items=1 sha256=563173ce78cfeea54c78b53e217ba3f7d05255a5ee4f643ac009353e81503b82
// | const PATCH_BEGIN = '*** Begin Patch';
pub const PATCH_BEGIN: &str = "*** Begin Patch";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=dd882f4a3afb9a0871e2091a14c0b9515f71efa11c2567ba7c3e920020947940
// | const PATCH_ADD_FILE = '*** Add File:';
pub const PATCH_ADD_FILE: &str = "*** Add File:";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=70d6227dd95f17fb7edc8316eec34d9e15bbe43422792df2ded07b0f0c70d1df
// | const PATCH_END = '*** End Patch';
pub const PATCH_END: &str = "*** End Patch";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers Array.isArray | JSON.parse | null | try statement | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | call of a sibling function | call of an imported function | field access | method call .find() | method call .test() | null | regular expression | typeof operator | undefined
