// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=b7f2b2eae45ed6b82fcc40e0aca8fc3773496b2ca1d0c541292bd84c81bee988 bytes=2136
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2; string-methods items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround-prelude begin
pub fn wa_str_replace_all(text: String, pattern: String, replacement: String) -> String {
    text.replace(pattern.as_str(), replacement.as_str()) }
// formal-ai:workaround-prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of an imported function | null | object without a $ tag | sibling value

// meta-language:translated JavaScript lexical_declaration items=1 sha256=19176c9448d9989d8075ad69bae5b894f9babc1cbc4a3b7d8ba0240b03b32848
// | /**
// |  * One `field value` line of a need record.
// |  * @param {string} field
// |  * @param {string} value
// |  * @returns {string}
// |  */
// | const bare = (field, value) => `  ${field} ${value}\n`;
pub fn bare(field: String, value: String) -> String {
    format!("{}{}", (format!("{}{}", (format!("{}{}", (format!("{}{}", (String::from("  ")), field)), (String::from(" ")))), value)), (String::from("\n")))
}

// formal-ai:workaround string-methods JavaScript lexical_declaration items=1 sha256=05c9f7c53b660e7f2ab1283cf3a250c8bfa7be3b32608b4514b3e73f79a4638b
// | /** @param {string} value */
// | const escape = (value) => value.replaceAll('\\', '\\\\').replaceAll('"', '\\"');
// ~ /** @param {string} value */
// ~ const escape = (value) => waStrReplaceAll(waStrReplaceAll(value, '\\', '\\\\'), '"', '\\"');
pub fn escape(value: String) -> String {
    crate::wa_str_replace_all(crate::wa_str_replace_all(value.clone(), String::from("\\"), String::from("\\\\")), String::from("\""), String::from("\\\""))
}

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | global call String() | null | undefined
