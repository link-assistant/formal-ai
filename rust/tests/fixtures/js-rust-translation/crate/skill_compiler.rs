// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=9f9a95c9e25fb2db4dd194f222ddc159ce59b990bdeec9f3bb69cf271668c832 bytes=7394
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=4

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

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

// meta-language:translated JavaScript import_statement items=1 sha256=c93397a6a5b6986d1b1ba5601bff2c4a7cfb3e7f36ebe32a56036a356de2e49d
// | import { KNOWLEDGE_SCHEMA_VERSION } from './skill_procedure.mjs';
use crate::skill_procedure::KNOWLEDGE_SCHEMA_VERSION;

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of an imported function | field access | method call .some()

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | call of a sibling function | call of an imported function | method call .filter() | method call .indexOf() | method call .slice() | method call .some() | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=5fdc3cb6948f26c3e3bebb44cb36b29133b6908f2f1fbce0bf7a6a0f6925f52d
// | // Trigger/response subset of CompiledSkillPackage. Structured descriptions
// | // stay unsupported until their typed permissions and effects are implemented.
// | const structuredLabels = ['Skill', 'Input', 'Precondition', 'Step', 'Effect',
// |   'Expected test', 'Target', 'Tool', 'Permission'];
pub static structuredLabels: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("Skill"), String::from("Input"), String::from("Precondition"), String::from("Step"), String::from("Effect"), String::from("Expected test"), String::from("Target"), String::from("Tool"), String::from("Permission")]);

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unterminated template literal
// formal-ai:blockers arrow callback of .some() | method call .slice() | method call .some() | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unterminated string literal
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .indexOf() | method call .map() | method call .replace() | method call .slice() | method call .sort() | method call .split() | null | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | destructuring | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers assignment of a field or element | call of a sibling function | call of an imported function | destructuring | method call .some() | method call .split() | null | object without a $ tag | regular expression
