// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=b1515b9e9c8925143be3caca25d5ba6e8b245e213ec4be338d1d284a3ffa171a bytes=4919
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=e54475f14ec7e4575b5b236821782fa012c42b003313d2b67cf3938e5e75ecd2
// | const UNEXPECTED_RESULT = 'unexpected-result';
pub const UNEXPECTED_RESULT: &str = "unexpected-result";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a877f7dc65e574afba098a86e182a432903f655837f083e31364e013d93ca579
// | const UNKNOWN_TOOL = 'unknown';
pub const UNKNOWN_TOOL: &str = "unknown";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of an imported function | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.isArray | JSDoc type {…} | assignment of a field or element | call of a sibling function | call of an imported function | field access | global call structuredClone() | method call .push() | null | nullish coalescing | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers Array.isArray | JSON.parse | field access | global call Boolean() | try statement | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of an imported function | method call .some() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers field access | null | optional chaining | undefined
