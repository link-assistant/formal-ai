// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=2d3a991bf049d5a273090337e9edb9f5e48d6c23290bd6e54dbc051759966db9 bytes=5290
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=3; string-methods items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround-prelude begin
pub fn wa_str_replace_all(text: String, pattern: String, replacement: String) -> String {
    text.replace(pattern.as_str(), replacement.as_str()) }
// formal-ai:workaround-prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

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

// meta-language:translated JavaScript lexical_declaration items=1 sha256=bb89c7826310af616f61a50da6311997457185628741037bd58553de08c838e7
// | const SOURCE_MODULE_TAG = 'source_module';
pub const SOURCE_MODULE_TAG: &str = "source_module";

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers call of an imported function | field access | method call .test() | null | optional chaining | regular expression | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | null | object without a $ tag

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers field access | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | method call .join() | method call .slice() | method call .split() | object without a $ tag

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers arrow function | call of an imported function | method call .join() | method call .slice() | method call .split() | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of an imported function | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | method call .join() | method call .slice()

// formal-ai:workaround string-methods JavaScript export_statement items=1 sha256=fb9c0e3f7c40a2791894e2c6f9252dd6217842486f68d2f0edca1b100dffdf61
// | /** Mirrors `fn quote` in rust/src/change_request.rs (and rebuild_plan.rs). @param {string} value */
// | export function quote(value) {
// |   return value.replaceAll('\\', '\\\\').replaceAll('"', "'").replaceAll('\n', '\\n').replaceAll('\r', '\\r').replaceAll('\t', '\\t');
// | }
// ~ /** Mirrors `fn quote` in rust/src/change_request.rs (and rebuild_plan.rs). @param {string} value */
// ~ export function quote(value) {
// ~   return waStrReplaceAll(waStrReplaceAll(waStrReplaceAll(waStrReplaceAll(waStrReplaceAll(value, '\\', '\\\\'), '"', "'"), '\n', '\\n'), '\r', '\\r'), '\t', '\\t');
// ~ }
// formal-ai:presentation:v1 rawSHA256=fb9c0e3f7c40a2791894e2c6f9252dd6217842486f68d2f0edca1b100dffdf61 rawBytes=398 renderedSHA256=9770a93bf6e447e2c253d76ce9a36b09f45a6bf052f76be32a7b7989d88df510
// formal-ai:presentation-raw cHViIGZuIHF1b3RlKHZhbHVlOiBTdHJpbmcpIC0+IFN0cmluZyB7CiAgICBjcmF0ZTo6d2Ffc3RyX3JlcGxhY2VfYWxsKGNy
// formal-ai:presentation-raw YXRlOjp3YV9zdHJfcmVwbGFjZV9hbGwoY3JhdGU6OndhX3N0cl9yZXBsYWNlX2FsbChjcmF0ZTo6d2Ffc3RyX3JlcGxhY2Vf
// formal-ai:presentation-raw YWxsKGNyYXRlOjp3YV9zdHJfcmVwbGFjZV9hbGwodmFsdWUuY2xvbmUoKSwgU3RyaW5nOjpmcm9tKCJcXCIpLCBTdHJpbmc6
// formal-ai:presentation-raw OmZyb20oIlxcXFwiKSksIFN0cmluZzo6ZnJvbSgiXCIiKSwgU3RyaW5nOjpmcm9tKCInIikpLCBTdHJpbmc6OmZyb20oIlxu
// formal-ai:presentation-raw IiksIFN0cmluZzo6ZnJvbSgiXFxuIikpLCBTdHJpbmc6OmZyb20oIlxyIiksIFN0cmluZzo6ZnJvbSgiXFxyIikpLCBTdHJp
// formal-ai:presentation-raw bmc6OmZyb20oIlx0IiksIFN0cmluZzo6ZnJvbSgiXFx0IikpCn0=
// formal-ai:presentation-raw end
pub fn quote(value: String) -> String {
    crate::wa_str_replace_all(crate::wa_str_replace_all(crate::wa_str_replace_all(crate::wa_str_replace_all(crate::wa_str_replace_all(value.clone(),
    String::from("\\"),
    String::from("\\\\")),
    String::from("\""),
    String::from("'")),
    String::from("\n"),
    String::from("\\n")),
    String::from("\r"),
    String::from("\\r")),
    String::from("\t"),
    String::from("\\t"))
}

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function
