// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=4c2680db17b15da8e413e09ce09e7160af39abfafdc00997d918c1a5effb090d bytes=9686
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:translated JavaScript lexical_declaration items=1 sha256=311fb20bb2142f369c4e97bcfda4e68e288f1d39535dc2ff5655bf819b7021b5
// | const PATH='data/seed/text-formalization.lino';
pub const PATH: &str = "data/seed/text-formalization.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=601c7f16f52095487eaaecad6e293c7b2ebf13fbdae15d88be0152a1156bad58
// | const questionKinds=[['where','location'],['how-many','quantity'],['why','text'],
// |   ['who','entity'],['when','time'],['which','text']];
// formal-ai:presentation:v1 rawSHA256=601c7f16f52095487eaaecad6e293c7b2ebf13fbdae15d88be0152a1156bad58 rawBytes=414 renderedSHA256=7302e6b7e5296124562ba724e5bdd9085d15c25d191bf55742dc0efb689ef498
// formal-ai:presentation-raw cHViIHN0YXRpYyBxdWVzdGlvbktpbmRzOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxWZWM8U3RyaW5nPj4+ID0gc3RkOjpz
// formal-ai:presentation-raw eW5jOjpMYXp5TG9jazo6bmV3KHx8IHZlYyFbdmVjIVtTdHJpbmc6OmZyb20oIndoZXJlIiksIFN0cmluZzo6ZnJvbSgibG9j
// formal-ai:presentation-raw YXRpb24iKV0sIHZlYyFbU3RyaW5nOjpmcm9tKCJob3ctbWFueSIpLCBTdHJpbmc6OmZyb20oInF1YW50aXR5IildLCB2ZWMh
// formal-ai:presentation-raw W1N0cmluZzo6ZnJvbSgid2h5IiksIFN0cmluZzo6ZnJvbSgidGV4dCIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIndobyIpLCBT
// formal-ai:presentation-raw dHJpbmc6OmZyb20oImVudGl0eSIpXSwgdmVjIVtTdHJpbmc6OmZyb20oIndoZW4iKSwgU3RyaW5nOjpmcm9tKCJ0aW1lIild
// formal-ai:presentation-raw LCB2ZWMhW1N0cmluZzo6ZnJvbSgid2hpY2giKSwgU3RyaW5nOjpmcm9tKCJ0ZXh0IildXSk7
// formal-ai:presentation-raw end
pub static questionKinds: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(||
    vec![vec![String::from("where"),
    String::from("location")],
    vec![String::from("how-many"),
    String::from("quantity")],
    vec![String::from("why"),
    String::from("text")],
    vec![String::from("who"),
    String::from("entity")],
    vec![String::from("when"),
    String::from("time")],
    vec![String::from("which"),
    String::from("text")]]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f3bd6b5ecf20704d1cfc79664e5ed1007756076a8c50accc8a31b47a84cd7447
// | const topics=['location','experience','compensation','contract','education','availability'];
// formal-ai:presentation:v1 rawSHA256=f3bd6b5ecf20704d1cfc79664e5ed1007756076a8c50accc8a31b47a84cd7447 rawBytes=255 renderedSHA256=f88a06df3c7879d21583941d55c33462be83edcd09b74cd173db527f27519cec
// formal-ai:presentation-raw cHViIHN0YXRpYyB0b3BpY3M6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpzeW5jOjpMYXp5TG9j
// formal-ai:presentation-raw azo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJsb2NhdGlvbiIpLCBTdHJpbmc6OmZyb20oImV4cGVyaWVuY2UiKSwgU3Ry
// formal-ai:presentation-raw aW5nOjpmcm9tKCJjb21wZW5zYXRpb24iKSwgU3RyaW5nOjpmcm9tKCJjb250cmFjdCIpLCBTdHJpbmc6OmZyb20oImVkdWNh
// formal-ai:presentation-raw dGlvbiIpLCBTdHJpbmc6OmZyb20oImF2YWlsYWJpbGl0eSIpXSk7
// formal-ai:presentation-raw end
pub static topics: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("location"),
    String::from("experience"),
    String::from("compensation"),
    String::from("contract"),
    String::from("education"),
    String::from("availability")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=5f4283f0271bde4a6adf96edf9693de63dcd38441ce2fbe518eb3e926a596f34
// | const currencies=['RUB','USD','EUR'];
pub static currencies: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("RUB"), String::from("USD"), String::from("EUR")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=bd87ce5f9644172dca70c9d1e6845422d7977eaeaad7d6fb9e77ddcd12570155
// | const units=['year','month','hour'];
pub static units: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("year"), String::from("month"), String::from("hour")]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .normalize()
// formal-ai:blockers arrow function | global call String() | method call .normalize() | method call .replace() | regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers arrow function | method call .test() | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .filter() | method call .find() | method call .map() | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers Array.from | call of a sibling function | method call .indexOf() | method call .pop() | method call .slice()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | arrow function | call of a sibling function | method call .some()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | arrow function | call of a sibling function | method call .some()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | arrow function | call of a sibling function | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Number
// formal-ai:blockers arrow callback of .every() | arrow callback of .some() | call of a sibling function | method call .at() | method call .every() | method call .join() | method call .replace() | method call .slice() | method call .some() | method call .split() | method call .test() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .find() | arrow callback of .from() | arrow callback of .some() | call of a sibling function | field access | method call .encode() | method call .filter() | method call .find() | method call .matchAll() | method call .pop() | method call .push() | method call .slice() | method call .some() | method call .test() | new TextEncoder | null | nullish coalescing | object without a $ tag | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal several declarators
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .find() | arrow callback of .from() | arrow callback of .some() | call of a sibling function | method call .filter() | method call .find() | method call .matchAll() | method call .replace() | method call .some() | method call .test() | null | nullish coalescing | object without a $ tag | optional chaining | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers field access | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers ++ operator | arrow callback of .every() | arrow callback of .map() | call of a sibling function | field access | method call .concat() | method call .every() | method call .map() | method call .push() | method call .replace() | method call .slice() | method call .split() | null | object spread | object without a $ tag | optional chaining | regular expression
