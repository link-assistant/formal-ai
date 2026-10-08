// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=62557d1fac8a26e8488ca73cb6186dc41e2185ce026e64c0932927779f9a4e46 bytes=10088
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal array destructuring

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8d11aa6be05c35a0b3967fa8a92c41dd426df899501f42aa9f4e345431a26952
// | const ASCII_QUOTES = ["'", '"', '`'];
pub static ASCII_QUOTES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("'"), String::from("\""), String::from("`")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b5466aab9ffb8a8cb91a1e17b8e2d836f980f7e5f1105ccaf5bb57735d02a874
// | const OPEN_ONLY = ['«', '“', '‘', '「', '『', '《'];
pub static OPEN_ONLY: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("«"), String::from("“"), String::from("‘"), String::from("「"), String::from("『"), String::from("《")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f6532f03f769e557f27ad4e3d8275f2b265f98bf2c1d44a2df852c946511ab6c
// | const CLOSE_FOLLOWERS = '.,;:!?)]}';
pub const CLOSE_FOLLOWERS: &str = ".,;:!?)]}";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=14f87ced3772ddb3bb74b94e42e98a4cfbd3f9e543f02e739fa50e02884883bb
// | const FAULT_FRAGMENT_CHARS = 32;
pub const FAULT_FRAGMENT_CHARS: f64 = 32f64;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b5080c827663144aa597ce903f1e55e22c2af301252ec8291fbaf9aa2b657987
// | const PAIRS = [
// |   ['```', '```'], ["'", "'"], ['"', '"'], ['`', '`'], ['«', '»'],
// |   ['“', '”'], ['‘', '’'], ['「', '」'], ['『', '』'], ['《', '》'],
// | ];
pub static PAIRS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(|| vec![vec![String::from("```"), String::from("```")], vec![String::from("'"), String::from("'")], vec![String::from("\""), String::from("\"")], vec![String::from("`"), String::from("`")], vec![String::from("«"), String::from("»")], vec![String::from("“"), String::from("”")], vec![String::from("‘"), String::from("’")], vec![String::from("「"), String::from("」")], vec![String::from("『"), String::from("』")], vec![String::from("《"), String::from("》")]]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
