// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=c391e0f7a4707488f08282467f54c68ef4c360b2f6e253815866a65e279b543c bytes=5940
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

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b5080c827663144aa597ce903f1e55e22c2af301252ec8291fbaf9aa2b657987
// | const PAIRS = [
// |   ['```', '```'], ["'", "'"], ['"', '"'], ['`', '`'], ['«', '»'],
// |   ['“', '”'], ['‘', '’'], ['「', '」'], ['『', '』'], ['《', '》'],
// | ];
pub static PAIRS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(|| vec![vec![String::from("```"), String::from("```")], vec![String::from("'"), String::from("'")], vec![String::from("\""), String::from("\"")], vec![String::from("`"), String::from("`")], vec![String::from("«"), String::from("»")], vec![String::from("“"), String::from("”")], vec![String::from("‘"), String::from("’")], vec![String::from("「"), String::from("」")], vec![String::from("『"), String::from("』")], vec![String::from("《"), String::from("》")]]);

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
