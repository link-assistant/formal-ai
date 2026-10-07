// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1599c452c9f038a089b94ae6259954fda305a70faffc34b1ff969445dbac255a bytes=2107
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

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

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .replaceAll()

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
