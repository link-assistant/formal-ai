// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=34ac4a78212e0924540740ec593daa35cc739cbe4465257a23acff00e5e5abb1 bytes=4948
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .flatMap()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .map()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=44f1c840798f1cf263e63276d83cf74e47f35234de148a5d8ac1c20251e389f9
// | const ARGUMENTS = ['path', 'name_lead', 'one_path', 'two_paths', 'remainder', 'search_query'];
pub static ARGUMENTS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("path"), String::from("name_lead"), String::from("one_path"), String::from("two_paths"), String::from("remainder"), String::from("search_query")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
