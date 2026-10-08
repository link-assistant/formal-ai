// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=5789b5bdcb2b15e168e8e45dd772d1f7bd2509d060f50e201bcffdce2357bd44 bytes=8543
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers arrow callback of .find() | assignment of a field or element | call of an imported function | class | export … | field access | global call String() | method call .append() | method call .find() | method call .push() | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:translated JavaScript lexical_declaration items=1 sha256=61d756bc519074cb25b505f1300fabd64603d540edb72123ade205dd6f733f88
// | const OFFLINE_SKIP = 'skipped:offline';
pub const OFFLINE_SKIP: &str = "skipped:offline";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers destructuring | field access | global call String() | method call .has() | method call .join() | method call .replace() | method call .split() | regular expression | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .push()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=7e491a2da562645b972d537ff8d01b02f2b81b8331377be19ff992f75e9adeca
// | const IMPULSE_KIND = 'impulse';
pub const IMPULSE_KIND: &str = "impulse";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=df513713ae60926b290677595e7753aca52fbb1f9f52da3679da42a42ed1c947
// | const RESPONSE_KIND = 'response';
pub const RESPONSE_KIND: &str = "response";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | call of a sibling function | field access | global call String() | method call .append() | method call .find() | method call .lastOf() | new EventLog | null | nullish coalescing
