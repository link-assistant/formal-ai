// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=0a243e07783b05f7190de7f40e2be75bd7e6fa359ac027bbff051b713fe83294 bytes=6966
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

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers export … | imported value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers imported value

// meta-language:translated JavaScript export_statement items=1 sha256=584c4bdc069894c7b21938e4288568b352892954fcb2982f4270fffa1c723b5c
// | /** Mirrors `ComputerUsePrimitive::permission_key`. @param {string} primitive */
// | export function permissionKey(primitive) {
// |   return `tool:computer:${primitive}`;
// | }
pub fn permission_key(primitive: String) -> String {
    format!("{}{}", (String::from("tool:computer:")), primitive)
}

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers method call .has() | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers object without a $ tag | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace member …
// formal-ai:blockers object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=e605d441612d4283c5002d53426550247425ef4a0715989a27c0ca34d58f5d70
// | const PLAN_FIELDS = ['plan_id', 'step_id', 'precondition', 'postcondition'];
pub static PLAN_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("plan_id"), String::from("step_id"), String::from("precondition"), String::from("postcondition")]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers assignment of a field or element | call of a sibling function | object without a $ tag | sibling value

// meta-language:translated JavaScript lexical_declaration items=1 sha256=5d441f64b244f463fb978e60399e211ac7da68a236f9d07170650bd7ed4761e1
// | const PLAN_FENCE = '```computer-use';
pub const PLAN_FENCE: &str = "```computer-use";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextEncoder

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers Array.isArray | Object.entries | Object.hasOwn | arrow callback of .every() | call of a sibling function | field access | method call .every() | object without a $ tag | sibling value | typeof operator | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers Array.isArray | JSON.parse | Object.hasOwn | Object.keys | arrow callback of .findIndex() | arrow callback of .some() | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | global call String() | let without a value | method call .encode() | method call .findIndex() | method call .join() | method call .padStart() | method call .push() | method call .slice() | method call .some() | method call .split() | null | object without a $ tag | regular expression | sibling value | try statement | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .findIndex() | call of a sibling function | call of an imported function | method call .findIndex() | method call .join() | method call .slice() | method call .split() | null | regular expression
