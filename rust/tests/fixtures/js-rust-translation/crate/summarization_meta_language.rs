// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=fcfb2e16223fc8c3e2b80c03c27584522090467805d4e9efc4f79aced4765ac2 bytes=7016
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8fc3287c3029994d875b3fbe4b54b92bd58ce7ce2d1529a272d009804c1d15bc
// | /** The 13 `SELF_DESCRIPTION_ROOTS` terms of meta-language's `self_description.rs`. */
// | const SELF_DESCRIPTION_TERMS = [
// |   'link', 'reference', 'relation link', 'language', 'grammar', 'type', 'Type', 'concept', 'point',
// |   'field', 'trivia', 'region', 'object',
// | ];
pub static SELF_DESCRIPTION_TERMS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("link"), String::from("reference"), String::from("relation link"), String::from("language"), String::from("grammar"), String::from("type"), String::from("Type"), String::from("concept"), String::from("point"), String::from("field"), String::from("trivia"), String::from("region"), String::from("object")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d6f6ad759846f778bfa5067858b5bf9dfa78ad5548e9d09df8ffe6a191a201b4
// | /** Links per extra token under the default `TriviaAttachmentPolicy::Both`. */
// | const TRIVIA_LINKS_PER_EXTRA_TOKEN = 2;
pub const TRIVIA_LINKS_PER_EXTRA_TOKEN: f64 = 2f64;

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal assignment

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
