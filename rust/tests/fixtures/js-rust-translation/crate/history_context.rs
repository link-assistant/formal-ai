// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=04e92446eaba4e26554cdc2382501697c9938c6bf2b137af3ca457ce054f32bd bytes=14733
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=73bef6fe09d2fd4895ee0b619b32941c072458ebfaac4628f972dca2f0d3cef2
// | const SEED_PATH = 'data/seed/history-formalization.lino';
pub const SEED_PATH: &str = "data/seed/history-formalization.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8ae9b7470652ebb303eb11b75c0809021d4ffb7cee3ae4ab124df3da48a1b304
// | const NUMBER_PLACEHOLDER = '%number%';
pub const NUMBER_PLACEHOLDER: &str = "%number%";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b3407d380205e2a63c05f450a2d2b09ad6a2cd84efec8b650d194a62b574e79c
// | const RECORD_SEPARATOR = '\u001e';
pub const RECORD_SEPARATOR: &str = "\u{1e}";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=178fa8b8bdc3241312c4a9f567a777aed2109bf73ef9868a7beee13829722bad
// | const UNIT_SEPARATOR = '\u001f';
pub const UNIT_SEPARATOR: &str = "\u{1f}";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:translated JavaScript lexical_declaration items=1 sha256=e6f2dcf29641818fd080d3fadbae4e7687a7541944ea2dc303bde2a3bd5bb71b
// | const STATEMENT_SEPARATORS = [' ', '.', ':', ')', '*'];
pub static STATEMENT_SEPARATORS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from(" "), String::from("."), String::from(":"), String::from(")"), String::from("*")]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .get()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
