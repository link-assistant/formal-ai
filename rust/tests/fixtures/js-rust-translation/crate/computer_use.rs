// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=edb5b11be9e3f8026181b525041f7ff9419e497049ce205205d49c5caf9ca4d8 bytes=4450
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

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

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal case test

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace member …

// meta-language:translated JavaScript lexical_declaration items=1 sha256=e605d441612d4283c5002d53426550247425ef4a0715989a27c0ca34d58f5d70
// | const PLAN_FIELDS = ['plan_id', 'step_id', 'precondition', 'postcondition'];
pub static PLAN_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("plan_id"), String::from("step_id"), String::from("precondition"), String::from("postcondition")]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
