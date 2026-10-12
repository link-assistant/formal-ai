// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=4486a0c714c2c6249a2744f8f1caa9bfba14a16350ec834593e2b879b01d4dde bytes=1388
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=8ec329f423ba3f3fefa0421789f50553a185b2492b901d95887e0f344b6b32fb
// | import { ASSUMED_TRUE_PRIOR, assessStatement, truthValue } from './relative_meta_logic.mjs';
use crate::relative_meta_logic::ASSUMED_TRUE_PRIOR;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of an imported function | global value Boolean | method call .filter() | method call .join() | method call .split() | object without a $ tag | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | call of an imported function | object without a $ tag
