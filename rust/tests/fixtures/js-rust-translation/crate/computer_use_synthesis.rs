// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=97ede073b2bf1eea45b014e93076dda2c9a4540da1a587eab3a61af826535ff8 bytes=7213
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=4

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=8315bb4c8544b356f02abca6e612b26c65fb36c445c557e0d3861c7038bf0ee5
// | import { FETCH_OPERATION, learned } from './computer_use_induction.mjs';
use crate::computer_use_induction::FETCH_OPERATION;

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript lexical_declaration items=1 sha256=7c66196baa00cc4ac1d98f619300dbe714133f2226b3282a3e166feb25f4b3d8
// | /** `PATH_FIELDS`: argument fields bound per request from the data flow. */
// | const PATH_FIELDS = ['path', 'paths', 'input', 'output', 'source', 'save_as', 'archive', 'destination', 'from', 'to'];
pub static PATH_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("path"), String::from("paths"), String::from("input"), String::from("output"), String::from("source"), String::from("save_as"), String::from("archive"), String::from("destination"), String::from("from"), String::from("to")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=24a6a7ac79386ca440832e059f6864089d6a7c47aba4c4f54eb77324235fd4c5
// | /** `RESOURCE_FIELDS`: argument fields taken from the resource binding. */
// | const RESOURCE_FIELDS = ['selector', 'pointer', 'column', 'equals'];
pub static RESOURCE_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("selector"), String::from("pointer"), String::from("column"), String::from("equals")]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .some() | call of a sibling function | call of an imported function | field access | imported value | method call .get() | method call .has() | method call .join() | method call .map() | method call .push() | method call .some() | null | object without a $ tag | optional chaining | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.isArray | Object.prototype | arrow function | field access | method call .call() | null | object without a $ tag | typeof operator | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers PATH_FIELDS.includes | RESOURCE_FIELDS.includes | arrow function | assignment of a field or element | call of a sibling function | destructuring | field access | global call structuredClone() | let without a value | method call .get() | method call .has() | method call .pop() | method call .split() | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Object.keys
// formal-ai:blockers Object.keys | call of an imported function | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | field access | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal assignment
// formal-ai:blockers assignment of a field or element | call of an imported function | destructuring | field access | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | global call String() | method call .map() | method call .padStart() | object spread | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .lastIndexOf()
// formal-ai:blockers method call .lastIndexOf() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .lastIndexOf()
// formal-ai:blockers method call .lastIndexOf() | method call .slice()
