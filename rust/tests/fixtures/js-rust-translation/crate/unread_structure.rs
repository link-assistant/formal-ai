// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=529856f6c8d653cd88500c8eba1eb06498762813661d342dcae2870bcd899dcf bytes=5863
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=e58fbf6cc94fd2e1e0b7560b3b4d24bee42f811972c7123cdc026caeb24be3ac
// | import { byteOrder, isAlphabetic, isAlphanumeric, lower, trim } from './rust_str.mjs';
use crate::rust_str::lower;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers Array.from | arrow callback of .sort() | call of an imported function | field access | global call String() | method call .push() | method call .sort()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers Array.from | call of an imported function | method call .pop() | method call .slice() | undefined

// meta-language:translated JavaScript export_statement items=1 sha256=4a65c7441a31d501ffabdb61705a5441c73822dbe404d125ae225be292d58e9c
// | /** Mirrors `QUOTE_PAIRS` in rust/src/coding/unread_structure.rs: literal delimiters that need no word boundary. */
// | export const QUOTE_PAIRS = Object.freeze([['"', '"'], ['`', '`'], ['\u201c', '\u201d'], ['\u00ab', '\u00bb'], ['\u300c', '\u300d'], ['\u300e', '\u300f']]);
// formal-ai:presentation:v1 rawSHA256=4a65c7441a31d501ffabdb61705a5441c73822dbe404d125ae225be292d58e9c rawBytes=378 renderedSHA256=6f1934f621599fef23581b067bd276a67312db113351a9083feae4bf5595771a
// formal-ai:presentation-raw cHViIHN0YXRpYyBRVU9URV9QQUlSUzogc3RkOjpzeW5jOjpMYXp5TG9jazxWZWM8VmVjPFN0cmluZz4+PiA9IHN0ZDo6c3lu
// formal-ai:presentation-raw Yzo6TGF6eUxvY2s6Om5ldyh8fCB2ZWMhW3ZlYyFbU3RyaW5nOjpmcm9tKCJcIiIpLCBTdHJpbmc6OmZyb20oIlwiIildLCB2
// formal-ai:presentation-raw ZWMhW1N0cmluZzo6ZnJvbSgiYCIpLCBTdHJpbmc6OmZyb20oImAiKV0sIHZlYyFbU3RyaW5nOjpmcm9tKCLigJwiKSwgU3Ry
// formal-ai:presentation-raw aW5nOjpmcm9tKCLigJ0iKV0sIHZlYyFbU3RyaW5nOjpmcm9tKCLCqyIpLCBTdHJpbmc6OmZyb20oIsK7IildLCB2ZWMhW1N0
// formal-ai:presentation-raw cmluZzo6ZnJvbSgi44CMIiksIFN0cmluZzo6ZnJvbSgi44CNIildLCB2ZWMhW1N0cmluZzo6ZnJvbSgi44COIiksIFN0cmlu
// formal-ai:presentation-raw Zzo6ZnJvbSgi44CPIildXSk7
// formal-ai:presentation-raw end
pub static QUOTE_PAIRS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(||
    vec![vec![String::from("\""),
    String::from("\"")],
    vec![String::from("`"),
    String::from("`")],
    vec![String::from("“"),
    String::from("”")],
    vec![String::from("«"),
    String::from("»")],
    vec![String::from("「"),
    String::from("」")],
    vec![String::from("『"),
    String::from("』")]]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | QUOTE_PAIRS.find | QUOTE_PAIRS.some | arrow callback of .find() | arrow callback of .some() | global call String() | method call .find() | method call .indexOf() | method call .slice() | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | global call String() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .map() | arrow function | call of an imported function | global call String() | method call .join() | method call .map() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of a sibling function | method call .indexOf() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | call of an imported function | destructuring | method call .join() | method call .push()
