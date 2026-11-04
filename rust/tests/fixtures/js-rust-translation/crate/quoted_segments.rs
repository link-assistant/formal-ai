// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=ca7afbe32ba651fd49560d56a97d9a7e4e157a56a5e4b82447edb5211b6ffe90 bytes=5011
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:translated JavaScript export_statement items=1 sha256=0d52cd7fd2df580606d56b49d75572f3c5b2e1f36e6953cf1be6c8a411b1a0f3
// | export const quotePairs = [
// |   ['```', '```'], ["'", "'"], ['"', '"'], ['`', '`'], ['«', '»'],
// |   ['“', '”'], ['‘', '’'], ['「', '」'], ['『', '』'], ['《', '》'],
// | ];
// formal-ai:presentation:v1 rawSHA256=0d52cd7fd2df580606d56b49d75572f3c5b2e1f36e6953cf1be6c8a411b1a0f3 rawBytes=565 renderedSHA256=5b2c79c9e1d3cb965f3e9a5366653cc1080d70d5118f9e943531405ce5e7a654
// formal-ai:presentation-raw cHViIHN0YXRpYyBxdW90ZVBhaXJzOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxWZWM8U3RyaW5nPj4+ID0gc3RkOjpzeW5j
// formal-ai:presentation-raw OjpMYXp5TG9jazo6bmV3KHx8IHZlYyFbdmVjIVtTdHJpbmc6OmZyb20oImBgYCIpLCBTdHJpbmc6OmZyb20oImBgYCIpXSwg
// formal-ai:presentation-raw dmVjIVtTdHJpbmc6OmZyb20oIiciKSwgU3RyaW5nOjpmcm9tKCInIildLCB2ZWMhW1N0cmluZzo6ZnJvbSgiXCIiKSwgU3Ry
// formal-ai:presentation-raw aW5nOjpmcm9tKCJcIiIpXSwgdmVjIVtTdHJpbmc6OmZyb20oImAiKSwgU3RyaW5nOjpmcm9tKCJgIildLCB2ZWMhW1N0cmlu
// formal-ai:presentation-raw Zzo6ZnJvbSgiwqsiKSwgU3RyaW5nOjpmcm9tKCLCuyIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIuKAnCIpLCBTdHJpbmc6OmZy
// formal-ai:presentation-raw b20oIuKAnSIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIuKAmCIpLCBTdHJpbmc6OmZyb20oIuKAmSIpXSwgdmVjIVtTdHJpbmc6
// formal-ai:presentation-raw OmZyb20oIuOAjCIpLCBTdHJpbmc6OmZyb20oIuOAjSIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIuOAjiIpLCBTdHJpbmc6OmZy
// formal-ai:presentation-raw b20oIuOAjyIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIuOAiiIpLCBTdHJpbmc6OmZyb20oIuOAiyIpXV0pOw==
// formal-ai:presentation-raw end
pub static quotePairs: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(||
    vec![vec![String::from("```"),
    String::from("```")],
    vec![String::from("'"),
    String::from("'")],
    vec![String::from("\""),
    String::from("\"")],
    vec![String::from("`"),
    String::from("`")],
    vec![String::from("«"),
    String::from("»")],
    vec![String::from("“"),
    String::from("”")],
    vec![String::from("‘"),
    String::from("’")],
    vec![String::from("「"),
    String::from("」")],
    vec![String::from("『"),
    String::from("』")],
    vec![String::from("《"),
    String::from("》")]]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal array destructuring
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | method call .push() | method call .slice() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unterminated template literal
// formal-ai:blockers call of a sibling function | destructuring | method call .exec() | method call .slice() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
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
