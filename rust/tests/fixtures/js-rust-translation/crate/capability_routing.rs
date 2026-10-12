// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=9eaa9695a0ee7c5f1cbddf81291310c470c474980e36f2615fb0ce93a2d4af34 bytes=22978
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=8

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=0cc0ba74032ac3247c1a7f9b6f28fc6d026d8dd3288afa1e41ad38c52de31ff3
// | const OBJECT_TYPES = ['url', 'path', 'pattern', 'path_set', 'path_scope', 'quoted_content', 'time_expression',
// |   'relative_period', 'language_name', 'quantity_question', 'task_list', 'delegation', 'bare_term', 'self_surface', 'none'];
// formal-ai:presentation:v1 rawSHA256=0cc0ba74032ac3247c1a7f9b6f28fc6d026d8dd3288afa1e41ad38c52de31ff3 rawBytes=514 renderedSHA256=9db4ca5d457603bfbc6f9aeb0e8ef0f1d8ce34a33226fa9049d8229c9785e88d
// formal-ai:presentation-raw cHViIHN0YXRpYyBPQkpFQ1RfVFlQRVM6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpzeW5jOjpM
// formal-ai:presentation-raw YXp5TG9jazo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJ1cmwiKSwgU3RyaW5nOjpmcm9tKCJwYXRoIiksIFN0cmluZzo6
// formal-ai:presentation-raw ZnJvbSgicGF0dGVybiIpLCBTdHJpbmc6OmZyb20oInBhdGhfc2V0IiksIFN0cmluZzo6ZnJvbSgicGF0aF9zY29wZSIpLCBT
// formal-ai:presentation-raw dHJpbmc6OmZyb20oInF1b3RlZF9jb250ZW50IiksIFN0cmluZzo6ZnJvbSgidGltZV9leHByZXNzaW9uIiksIFN0cmluZzo6
// formal-ai:presentation-raw ZnJvbSgicmVsYXRpdmVfcGVyaW9kIiksIFN0cmluZzo6ZnJvbSgibGFuZ3VhZ2VfbmFtZSIpLCBTdHJpbmc6OmZyb20oInF1
// formal-ai:presentation-raw YW50aXR5X3F1ZXN0aW9uIiksIFN0cmluZzo6ZnJvbSgidGFza19saXN0IiksIFN0cmluZzo6ZnJvbSgiZGVsZWdhdGlvbiIp
// formal-ai:presentation-raw LCBTdHJpbmc6OmZyb20oImJhcmVfdGVybSIpLCBTdHJpbmc6OmZyb20oInNlbGZfc3VyZmFjZSIpLCBTdHJpbmc6OmZyb20o
// formal-ai:presentation-raw Im5vbmUiKV0pOw==
// formal-ai:presentation-raw end
pub static OBJECT_TYPES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("url"),
    String::from("path"),
    String::from("pattern"),
    String::from("path_set"),
    String::from("path_scope"),
    String::from("quoted_content"),
    String::from("time_expression"),
    String::from("relative_period"),
    String::from("language_name"),
    String::from("quantity_question"),
    String::from("task_list"),
    String::from("delegation"),
    String::from("bare_term"),
    String::from("self_surface"),
    String::from("none")]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b0490b7667775168eef7a3ea90a16b6aa9f5e9940311b0954280e99daed4f626
// | const ACTS_IN_PRECEDENCE = ['schedule', 'demonstrate', 'compose', 'enumerate', 'transform', 'explain', 'record',
// |   'learn', 'retrieve'];
// formal-ai:presentation:v1 rawSHA256=b0490b7667775168eef7a3ea90a16b6aa9f5e9940311b0954280e99daed4f626 rawBytes=332 renderedSHA256=1d8a754b6dc6e26f0eff5a266ed578a647b49210825386f4bc90860d51ce7e29
// formal-ai:presentation-raw cHViIHN0YXRpYyBBQ1RTX0lOX1BSRUNFREVOQ0U6IHN0ZDo6c3luYzo6TGF6eUxvY2s8VmVjPFN0cmluZz4+ID0gc3RkOjpz
// formal-ai:presentation-raw eW5jOjpMYXp5TG9jazo6bmV3KHx8IHZlYyFbU3RyaW5nOjpmcm9tKCJzY2hlZHVsZSIpLCBTdHJpbmc6OmZyb20oImRlbW9u
// formal-ai:presentation-raw c3RyYXRlIiksIFN0cmluZzo6ZnJvbSgiY29tcG9zZSIpLCBTdHJpbmc6OmZyb20oImVudW1lcmF0ZSIpLCBTdHJpbmc6OmZy
// formal-ai:presentation-raw b20oInRyYW5zZm9ybSIpLCBTdHJpbmc6OmZyb20oImV4cGxhaW4iKSwgU3RyaW5nOjpmcm9tKCJyZWNvcmQiKSwgU3RyaW5n
// formal-ai:presentation-raw Ojpmcm9tKCJsZWFybiIpLCBTdHJpbmc6OmZyb20oInJldHJpZXZlIildKTs=
// formal-ai:presentation-raw end
pub static ACTS_IN_PRECEDENCE: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("schedule"),
    String::from("demonstrate"),
    String::from("compose"),
    String::from("enumerate"),
    String::from("transform"),
    String::from("explain"),
    String::from("record"),
    String::from("learn"),
    String::from("retrieve")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d2436be8075649fd43b7732a4d46fea54ff23415c380ce3052f27055455c3c30
// | /**
// |  * Mirrors `Act::role`.
// |  * @param {string} act
// |  * @returns {string}
// |  */
// | const actRole = (act) => (act === 'unresolved' ? '' : `capability_act_${act}`);
pub fn act_role(act: String) -> String {
    if (act == "unresolved") {
        String::from("")
    } else {
        format!("{}{}", (String::from("capability_act_")), act)
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=dc6c6bdfd0f774fc603b9302550b88f9b538e1b0a2eb1703975bd1e65f4153e6
// | const LOCI = ['workspace', 'web', 'dialogue', 'self', 'unresolved'];
// formal-ai:presentation:v1 rawSHA256=dc6c6bdfd0f774fc603b9302550b88f9b538e1b0a2eb1703975bd1e65f4153e6 rawBytes=210 renderedSHA256=3187b1c72f7d2c7364d8c3c5e585051bf436be91b69b3e9bb162dd56991db388
// formal-ai:presentation-raw cHViIHN0YXRpYyBMT0NJOiBzdGQ6OnN5bmM6OkxhenlMb2NrPFZlYzxTdHJpbmc+PiA9IHN0ZDo6c3luYzo6TGF6eUxvY2s6
// formal-ai:presentation-raw Om5ldyh8fCB2ZWMhW1N0cmluZzo6ZnJvbSgid29ya3NwYWNlIiksIFN0cmluZzo6ZnJvbSgid2ViIiksIFN0cmluZzo6ZnJv
// formal-ai:presentation-raw bSgiZGlhbG9ndWUiKSwgU3RyaW5nOjpmcm9tKCJzZWxmIiksIFN0cmluZzo6ZnJvbSgidW5yZXNvbHZlZCIpXSk7
// formal-ai:presentation-raw end
pub static LOCI: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(||
    vec![String::from("workspace"),
    String::from("web"),
    String::from("dialogue"),
    String::from("self"),
    String::from("unresolved")]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ab0ec0e153ec0df798bb368a9a21dcb87c4e9527a61edc495a7e8ab0a93483eb
// | const ROUTING_FILE = 'data/seed/capability-routing.lino';
pub const ROUTING_FILE: &str = "data/seed/capability-routing.lino";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .some() | call of an imported function | field access | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow callback of .map() | call of an imported function | field access | method call .filter() | method call .flatMap() | method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of an imported function | method call .filter() | method call .map() | method call .replace() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .some() | call of a sibling function | call of an imported function | method call .flatMap() | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | call of an imported function | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .some() | arrow callback of .sort() | arrow function | call of a sibling function | call of an imported function | imported value | method call .push() | method call .some() | method call .sort() | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers ACTS_IN_PRECEDENCE.map | arrow callback of .filter() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | call of an imported function | method call .filter() | method call .map() | method call .push() | method call .sort()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | field access | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers LOCI.includes | OBJECT_TYPES.includes | call of an imported function | field access | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | field access | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of an imported function | field access | method call .some() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | assignment of a field or element | call of a sibling function | destructuring | field access | method call .filter() | null | nullish coalescing | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f1dce9b6ffe13c82c2925abbc553d7567b657a62964405751ab7e6c7d58da0d8
// | const DEFAULT_OUTCOME = 'ask';
pub const DEFAULT_OUTCOME: &str = "ask";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers field access | method call .push()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .reduce() | call of an imported function | method call .filter() | method call .reduce()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | imported value | method call .every() | method call .some()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | method call .has() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | call of a sibling function | imported value | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | call of an imported function | method call .filter()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .every() | arrow callback of .some() | arrow function | call of an imported function | method call .every() | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of an imported function | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .some() | arrow function | method call .some() | method call .test() | regular expression | undefined

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8281d29ab2869ba70f967cb91ec0d6b13194e1c7a87a914891a366a0159bff50
// | const QUOTE_PAIRS = [['"', '"'], ['“', '”'], ['«', '»']];
// formal-ai:presentation:v1 rawSHA256=8281d29ab2869ba70f967cb91ec0d6b13194e1c7a87a914891a366a0159bff50 rawBytes=238 renderedSHA256=356626a6dec67d85ec4376271c03dfc4819bf751486b937eaf286e52e37d3499
// formal-ai:presentation-raw cHViIHN0YXRpYyBRVU9URV9QQUlSUzogc3RkOjpzeW5jOjpMYXp5TG9jazxWZWM8VmVjPFN0cmluZz4+PiA9IHN0ZDo6c3lu
// formal-ai:presentation-raw Yzo6TGF6eUxvY2s6Om5ldyh8fCB2ZWMhW3ZlYyFbU3RyaW5nOjpmcm9tKCJcIiIpLCBTdHJpbmc6OmZyb20oIlwiIildLCB2
// formal-ai:presentation-raw ZWMhW1N0cmluZzo6ZnJvbSgi4oCcIiksIFN0cmluZzo6ZnJvbSgi4oCdIildLCB2ZWMhW1N0cmluZzo6ZnJvbSgiwqsiKSwg
// formal-ai:presentation-raw U3RyaW5nOjpmcm9tKCLCuyIpXV0pOw==
// formal-ai:presentation-raw end
pub static QUOTE_PAIRS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(||
    vec![vec![String::from("\""),
    String::from("\"")],
    vec![String::from("“"),
    String::from("”")],
    vec![String::from("«"),
    String::from("»")]]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers call of an imported function | destructuring | method call .indexOf() | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .some() | call of a sibling function | method call .some() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | sibling value

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .some() | call of an imported function | method call .some() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.from | arrow callback of .every() | arrow callback of .flatMap() | arrow function | call of a sibling function | call of an imported function | method call .every() | method call .flatMap() | method call .join() | method call .slice() | null

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextEncoder

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextDecoder

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal label …

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal bitwise &
// formal-ai:blockers arrow function | bitwise operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | method call .decode() | method call .encode() | method call .slice() | null | sibling value

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated string literal
// formal-ai:blockers arrow callback of .find() | arrow callback of .map() | arrow callback of .reduceRight() | arrow function | call of a sibling function | call of an imported function | field access | method call .find() | method call .map() | method call .reduceRight() | method call .replace() | method call .slice() | null | regular expression | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow function | call of an imported function | method call .find() | null | sibling value | undefined
