// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=fbb5325d2f3c6ff646adbbad708d2552bf95e65ce2301c3a5faac0849a7bc179 bytes=10210
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript export_statement items=1 sha256=c08fca1bfece6b19da6c61552d27df6475f5c8e0334bd5d0030618a777467dfc
// | /** Mirrors `ROOT_HEADER` in rust/src/memory.rs. */
// | export const ROOT_HEADER = 'demo_memory';
pub const ROOT_HEADER: &str = "demo_memory";

// meta-language:translated JavaScript export_statement items=1 sha256=f96670b2b67e0d28929d109d14d432ec23c957e56763e3d015c6874dcd69f590
// | /** Mirrors `MINIMUM_READABLE_MEMORY_SCHEMA_VERSION` in rust/src/memory/upgrade.rs. */
// | export const MINIMUM_READABLE_MEMORY_SCHEMA_VERSION = 1;
pub const MINIMUM_READABLE_MEMORY_SCHEMA_VERSION: f64 = 1f64;

// meta-language:translated JavaScript export_statement items=1 sha256=5c6f3c24bb66bf135eead4d8e95e28365ae25e2a64a578a83cd8b300619af511
// | /** Mirrors `MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION` in rust/src/memory/upgrade.rs. */
// | export const MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION = 2;
pub const MAXIMUM_READABLE_MEMORY_SCHEMA_VERSION: f64 = 2f64;

// meta-language:translated JavaScript export_statement items=1 sha256=0a775020b873d391b446edb369038c6c802f1f42d5e3962711c7a8bd32fe2dc5
// | /** Mirrors `TARGET_MEMORY_SCHEMA_VERSION` in rust/src/memory/upgrade.rs. */
// | export const TARGET_MEMORY_SCHEMA_VERSION = 2;
pub const TARGET_MEMORY_SCHEMA_VERSION: f64 = 2f64;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers null | object spread | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=5f018f59494c176d656a8ad4eb835e92df2273db834b790e53cc08a16d8e09d0
// | /** The persisted field spelling of each optional `MemoryEvent` field, in format order. */
// | const FIELD_KEYS = [
// |   ['kind', 'kind'],
// |   ['role', 'role'],
// |   ['intent', 'intent'],
// |   ['tool', 'tool'],
// |   ['inputs', 'inputs'],
// |   ['outputs', 'outputs'],
// |   ['content', 'content'],
// |   ['sentAt', 'sent_at'],
// |   ['demoLabel', 'demo_label'],
// |   ['conversationId', 'conversation_id'],
// |   ['conversationTitle', 'conversation_title'],
// | ];
pub static FIELD_KEYS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(|| vec![vec![String::from("kind"), String::from("kind")], vec![String::from("role"), String::from("role")], vec![String::from("intent"), String::from("intent")], vec![String::from("tool"), String::from("tool")], vec![String::from("inputs"), String::from("inputs")], vec![String::from("outputs"), String::from("outputs")], vec![String::from("content"), String::from("content")], vec![String::from("sentAt"), String::from("sent_at")], vec![String::from("demoLabel"), String::from("demo_label")], vec![String::from("conversationId"), String::from("conversation_id")], vec![String::from("conversationTitle"), String::from("conversation_title")]]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Map

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: one value is used as a string and as an array; declare the types of the function with JSDoc
// formal-ai:blockers arrow function | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .replace() | method call .test() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers global call String() | method call .join() | method call .split()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | nullish coalescing | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of an imported function | method call .indexOf() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .filter() | method call .get() | method call .has() | method call .push() | method call .slice() | method call .split() | null | nullish coalescing | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers call of a sibling function | destructuring | field access | method call .join() | null | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .indexOf() | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | call of an imported function | method call .slice() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers assignment of a field or element | call of a sibling function | class | export … | field access | nullish coalescing
