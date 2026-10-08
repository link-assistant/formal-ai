// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1599c452c9f038a089b94ae6259954fda305a70faffc34b1ff969445dbac255a bytes=2107
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

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

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .replaceAll()
// formal-ai:blockers arrow function | method call .replaceAll()

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | global call String() | null | undefined
