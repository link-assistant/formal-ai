// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a6af27c82925febbe84a1c5a9af55c940095c73be93a0b227ec40bf31c087350 bytes=10313
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of an imported function | field access | method call .indexOf() | method call .push() | method call .slice() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal array destructuring
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | method call .push() | method call .slice() | null | object without a $ tag

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
// formal-ai:blockers ASCII_QUOTES.includes | Array.from | CLOSE_FOLLOWERS.includes | OPEN_ONLY.includes | arrow callback of .reduce() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .join() | method call .push() | method call .reduce() | method call .slice() | method call .test() | null | object without a $ tag | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of an imported function | method call .test() | regular expression | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of an imported function | method call .slice()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b5080c827663144aa597ce903f1e55e22c2af301252ec8291fbaf9aa2b657987
// | const PAIRS = [
// |   ['```', '```'], ["'", "'"], ['"', '"'], ['`', '`'], ['«', '»'],
// |   ['“', '”'], ['‘', '’'], ['「', '」'], ['『', '』'], ['《', '》'],
// | ];
pub static PAIRS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(|| vec![vec![String::from("```"), String::from("```")], vec![String::from("'"), String::from("'")], vec![String::from("\""), String::from("\"")], vec![String::from("`"), String::from("`")], vec![String::from("«"), String::from("»")], vec![String::from("“"), String::from("”")], vec![String::from("‘"), String::from("’")], vec![String::from("「"), String::from("」")], vec![String::from("『"), String::from("』")], vec![String::from("《"), String::from("》")]]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers PAIRS.some | arrow callback of .some() | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers PAIRS.some | arrow callback of .some()

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unterminated template literal
// formal-ai:blockers call of a sibling function | destructuring | method call .exec() | method call .slice() | null | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers Array.from | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .indexOf() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers Array.from | call of a sibling function | call of an imported function | method call .indexOf() | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers Array.from | call of a sibling function | call of an imported function | method call .indexOf() | method call .slice() | method call .test() | null | regular expression | undefined
