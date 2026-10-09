// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=5c99e24b6d91ac41048eb2c940459bca937bb03c0024c1211703ac9abf8a4859 bytes=5806
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=4

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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript lexical_declaration items=1 sha256=327e9ae59b6bd6d0e3209d5a45b20cb61db60dec0ed762f51da9a4957e2dcda0
// | const IMPULSE = 'impulse';
pub const IMPULSE: &str = "impulse";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=c7ebc5ad854c0349fe49925113bc4344a13c8dbd8a237b1765cccb2daa6d7bb7
// | const LANGUAGE = 'language';
pub const LANGUAGE: &str = "language";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=78777f638983c7e28f0ddd3219a30b1104653c82593898a5c59df9ae29abdbb1
// | const ROUTE = 'intent_formalization:route';
pub const ROUTE: &str = "intent_formalization:route";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f8a309f40a61063722d97cdaa5518c1eb35618fa7ccc38aca99bc6a14b2d9f83
// | const CANDIDATE = 'candidate';
pub const CANDIDATE: &str = "candidate";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f4259bab73fe7066d4c7a76523f11df19d00adfbb8a73522c2f4f44eeff6a067
// | const META_RESPONSE = 'response:meta_reasoner';
pub const META_RESPONSE: &str = "response:meta_reasoner";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=c4ef579a5720f22e6653b84c45d1de91b7b2ebff5071bc005ee5ae0bd311f73d
// | const DISPATCH_STEP = 'dispatch_handler';
pub const DISPATCH_STEP: &str = "dispatch_handler";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ab85a4cdc46963a0d9d81f4eeec6ef43ee817cf5e92f18473e0fd74d5bf5db2f
// | const UNKNOWN_RULE_SELECTION = 'initial unknown reason no_seed_route next try_rule_synthesis';
pub const UNKNOWN_RULE_SELECTION: &str = "initial unknown reason no_seed_route next try_rule_synthesis";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers Array.isArray | Object.hasOwn | Object.keys | arrow callback of .find() | call of an imported function | field access | global call String() | method call .find() | method call .workerHandlerRegistryDefinition() | null | nullish coalescing | object without a $ tag | optional chaining | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.isArray | JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | arrow callback of .some() | call of a sibling function | call of an imported function | field access | global call String() | method call .filter() | method call .has() | method call .map() | method call .push() | method call .slice() | method call .some() | null | nullish coalescing | object without a $ tag | optional chaining | sibling value
