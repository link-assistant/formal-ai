// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=c5b4a60c1d578fe939e740b518dbcd5f7bc826b8157d02f95e0f1c8afb1a8c58 bytes=7255
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=4; string-methods items=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround-prelude begin
pub fn wa_str_last_index_of(text: String, search: String) -> f64 {
    let units: Vec<u16> = text.encode_utf16().collect();
    let needle: Vec<u16> = search.encode_utf16().collect();
    if needle.is_empty() { return units.len() as f64; }
    if needle.len() > units.len() { return -1.0; }
    units.windows(needle.len()).rposition(|window| window == needle.as_slice()).map_or(-1.0, |at| at as f64) }

pub fn wa_index(index: f64, len: usize) -> usize {
    let whole = if index.is_nan() { 0.0 } else { index.trunc() };
    (if whole < 0.0 { (len as f64 + whole).max(0.0) } else { whole.min(len as f64) }) as usize }

pub fn wa_str_slice(text: String, start: f64, end: f64) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    let (from, to) = (wa_index(start, units.len()), wa_index(end, units.len()));
    if from >= to { String::new() } else { String::from_utf16_lossy(&units[from..to]) } }

pub fn wa_str_slice_from(text: String, start: f64) -> String {
    wa_str_slice(text, start, f64::INFINITY) }
// formal-ai:workaround-prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=8315bb4c8544b356f02abca6e612b26c65fb36c445c557e0d3861c7038bf0ee5
// | import { FETCH_OPERATION, learned } from './computer_use_induction.mjs';
use crate::computer_use_induction::FETCH_OPERATION;

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript lexical_declaration items=1 sha256=7c66196baa00cc4ac1d98f619300dbe714133f2226b3282a3e166feb25f4b3d8
// | /** `PATH_FIELDS`: argument fields bound per request from the data flow. */
// | const PATH_FIELDS = ['path', 'paths', 'input', 'output', 'source', 'save_as', 'archive', 'destination', 'from', 'to'];
pub static PATH_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("path"), String::from("paths"), String::from("input"), String::from("output"), String::from("source"), String::from("save_as"), String::from("archive"), String::from("destination"), String::from("from"), String::from("to")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=24a6a7ac79386ca440832e059f6864089d6a7c47aba4c4f54eb77324235fd4c5
// | /** `RESOURCE_FIELDS`: argument fields taken from the resource binding. */
// | const RESOURCE_FIELDS = ['selector', 'pointer', 'column', 'equals'];
pub static RESOURCE_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("selector"), String::from("pointer"), String::from("column"), String::from("equals")]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .some() | call of a sibling function | call of an imported function | field access | imported value | method call .get() | method call .has() | method call .join() | method call .map() | method call .push() | method call .some() | null | object without a $ tag | optional chaining | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.isArray | Object.prototype | arrow function | field access | method call .call() | null | object without a $ tag | typeof operator | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers PATH_FIELDS.includes | RESOURCE_FIELDS.includes | arrow function | assignment of a field or element | call of a sibling function | destructuring | field access | global call structuredClone() | let without a value | method call .get() | method call .has() | method call .pop() | method call .split() | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Object.keys
// formal-ai:blockers Object.keys | call of an imported function | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | field access | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal assignment
// formal-ai:blockers assignment of a field or element | call of an imported function | destructuring | field access | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | global call String() | method call .map() | method call .padStart() | object spread | object without a $ tag

// formal-ai:workaround string-methods JavaScript function_declaration items=1 sha256=4fd47d3e6112d9a19868365b81e9a7fd4aa3aa7632d19de2b8c33aea1af5ee3c
// | /** Mirrors `fn parent_of`. @param {string} path */
// | function parentOf(path) {
// |   const at = path.lastIndexOf('/');
// |   return at < 0 ? '.' : path.slice(0, at);
// | }
// ~ /** Mirrors `fn parent_of`. @param {string} path */
// ~ function parentOf(path) {
// ~   const at = waStrLastIndexOf(path, '/');
// ~   return at < 0 ? '.' : waStrSlice(path, 0, at);
// ~ }
pub fn parent_of(path: String) -> String {
    {
        let at = crate::wa_str_last_index_of(path.clone(), String::from("/"));
        if (at < (0f64)) {
            String::from(".")
        } else {
            crate::wa_str_slice(path.clone(), 0f64, at)
        }
    }
}

// formal-ai:workaround string-methods JavaScript function_declaration items=1 sha256=9db9d1c72a358d31b984db1c6edc9e278a80ba56a22878c8cf635d64b0c285e2
// | /** Mirrors `fn basename_of`. @param {string} path */
// | function basenameOf(path) {
// |   const at = path.lastIndexOf('/');
// |   return at < 0 ? path : path.slice(at + 1);
// | }
// ~ /** Mirrors `fn basename_of`. @param {string} path */
// ~ function basenameOf(path) {
// ~   const at = waStrLastIndexOf(path, '/');
// ~   return at < 0 ? path : waStrSliceFrom(path, at + 1);
// ~ }
pub fn basename_of(path: String) -> String {
    {
        let at = crate::wa_str_last_index_of(path.clone(), String::from("/"));
        if (at < (0f64)) {
            path.clone()
        } else {
            crate::wa_str_slice_from(path.clone(), (at + 1f64))
        }
    }
}
