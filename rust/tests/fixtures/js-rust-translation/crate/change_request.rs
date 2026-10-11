// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=8c1097cf9aef03f722cd98e13d87505d12b2648b41b3cab159d51e1ff7ea4a78 bytes=5481
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

// meta-language:translated JavaScript export_statement items=1 sha256=8d6c5ad72885fd47128957ecd0604737218898d88e34c12aaaf61746fc6d4a6f
// | /** The canonical change's grounded target (Rust `canonical_change_request`). */
// | export const CANONICAL_TARGET_MODULE = 'src/agentic_coding/planner.rs';
pub const CANONICAL_TARGET_MODULE: &str = "src/agentic_coding/planner.rs";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | null | object without a $ tag | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | call of an imported function | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers field access | object without a $ tag

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: function value CANONICAL_TARGET_MODULE: functions and namespaces are only portable when a function is called
// formal-ai:blockers call of a sibling function | call of an imported function

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
pub fn quote(value: String) -> String {
    crate::wa_str_replace_all(crate::wa_str_replace_all(crate::wa_str_replace_all(crate::wa_str_replace_all(crate::wa_str_replace_all(value.clone(), String::from("\\"), String::from("\\\\")), String::from("\""), String::from("'")), String::from("\n"), String::from("\\n")), String::from("\r"), String::from("\\r")), String::from("\t"), String::from("\\t"))
}

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function
