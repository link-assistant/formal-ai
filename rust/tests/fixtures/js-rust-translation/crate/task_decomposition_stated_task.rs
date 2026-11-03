// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=504dfa34f906662f90d41fcd252b1eb95387b8a46d920430a5173c183ff1f8af bytes=2593
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:translated JavaScript lexical_declaration items=1 sha256=c908f4428cb781f6d4e2e3d1f405768b5dd6becacce7c298f8be00718a45e63c
// | const QUOTE_PAIRS = [['«', '»'], ['“', '”'], ['‘', '’'], ['「', '」'],
// |   ['『', '』'], ['"', '"'], ["'", "'"]];
// formal-ai:presentation:v1 rawSHA256=c908f4428cb781f6d4e2e3d1f405768b5dd6becacce7c298f8be00718a45e63c rawBytes=426 renderedSHA256=b4af8a39cda4059e79c2fdd691be2ae967ec1337fce0f0237960d810f9a497a4
// formal-ai:presentation-raw cHViIHN0YXRpYyBRVU9URV9QQUlSUzogc3RkOjpzeW5jOjpMYXp5TG9jazxWZWM8VmVjPFN0cmluZz4+PiA9IHN0ZDo6c3lu
// formal-ai:presentation-raw Yzo6TGF6eUxvY2s6Om5ldyh8fCB2ZWMhW3ZlYyFbU3RyaW5nOjpmcm9tKCLCqyIpLCBTdHJpbmc6OmZyb20oIsK7IildLCB2
// formal-ai:presentation-raw ZWMhW1N0cmluZzo6ZnJvbSgi4oCcIiksIFN0cmluZzo6ZnJvbSgi4oCdIildLCB2ZWMhW1N0cmluZzo6ZnJvbSgi4oCYIiks
// formal-ai:presentation-raw IFN0cmluZzo6ZnJvbSgi4oCZIildLCB2ZWMhW1N0cmluZzo6ZnJvbSgi44CMIiksIFN0cmluZzo6ZnJvbSgi44CNIildLCB2
// formal-ai:presentation-raw ZWMhW1N0cmluZzo6ZnJvbSgi44COIiksIFN0cmluZzo6ZnJvbSgi44CPIildLCB2ZWMhW1N0cmluZzo6ZnJvbSgiXCIiKSwg
// formal-ai:presentation-raw U3RyaW5nOjpmcm9tKCJcIiIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIiciKSwgU3RyaW5nOjpmcm9tKCInIildXSk7
// formal-ai:presentation-raw end
pub static QUOTE_PAIRS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(||
    vec![vec![String::from("«"),
    String::from("»")],
    vec![String::from("“"),
    String::from("”")],
    vec![String::from("‘"),
    String::from("’")],
    vec![String::from("「"),
    String::from("」")],
    vec![String::from("『"),
    String::from("』")],
    vec![String::from("\""),
    String::from("\"")],
    vec![String::from("'"),
    String::from("'")]]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | method call .has() | method call .test() | regular expression | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .map() | global value Boolean | method call .filter() | method call .join() | method call .map() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | QUOTE_PAIRS.find | QUOTE_PAIRS.some | arrow callback of .find() | arrow callback of .findIndex() | arrow callback of .some() | method call .findIndex() | method call .join() | method call .lastIndexOf() | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers String.fromCodePoint | call of an imported function | method call .codePointAt() | method call .has() | method call .push() | method call .slice() | null | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | method call .lastIndexOf() | method call .slice() | null | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | nullish coalescing
