// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a4acac80069ba3f34910772528f79f536bc794aa23e48a0cd64094d6ad9e6bab bytes=15893
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=4; string-methods items=1

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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=dacb82787d1636a91a39db254d82dfc76f2d39de0d6f9c8656061e3161a668c7
// | /** Mirrors `const NOT_RECORDED`. */
// | export const NOT_RECORDED = 'not recorded';
pub const NOT_RECORDED: &str = "not recorded";

// meta-language:translated JavaScript export_statement items=1 sha256=d1e0dd11be3d51937ac74d9c72f1469d7420fea37ad705594e0006abbe38312b
// | /** Mirrors `const SEARCH_REQUEST_KIND`. */
// | export const SEARCH_REQUEST_KIND = 'web_search:request';
pub const SEARCH_REQUEST_KIND: &str = "web_search:request";

// meta-language:translated JavaScript export_statement items=1 sha256=ea67da03880dc27bc1d6238dd28197fc3a4312820c208b3ae02df05f8eaff04c
// | /** Mirrors `const SOURCE_HTTP_KIND`. */
// | export const SOURCE_HTTP_KIND = 'source:http';
pub const SOURCE_HTTP_KIND: &str = "source:http";

// meta-language:translated JavaScript export_statement items=1 sha256=cac2692932a14fe239bfafa9b89916931bf9ee9f00dc4e2d8ca7bb34eb929148
// | /** Mirrors `const FORMALIZE_FRAGMENT_KIND`. */
// | export const FORMALIZE_FRAGMENT_KIND = 'formalize:fragment';
pub const FORMALIZE_FRAGMENT_KIND: &str = "formalize:fragment";

// meta-language:translated JavaScript export_statement items=1 sha256=14c52aa090e7e5067f4823ccef8241db7e620189f189974faeaadc2bef5dda62
// | /** Mirrors `const DECOMPOSE_PART_KIND`. */
// | export const DECOMPOSE_PART_KIND = 'decompose:part';
pub const DECOMPOSE_PART_KIND: &str = "decompose:part";

// meta-language:translated JavaScript export_statement items=1 sha256=70d1f2c71210f7a34db6b5bb3781126349d06e155094f5c8419237fc41840fc3
// | /** Mirrors `const RECOMPOSE_BIND_KIND`. */
// | export const RECOMPOSE_BIND_KIND = 'recompose:bind';
pub const RECOMPOSE_BIND_KIND: &str = "recompose:bind";

// meta-language:translated JavaScript export_statement items=1 sha256=c75ba2ab978d59cf00590eb51a21ff23f7e3e330ab91427ed811547072a72a66
// | /** Mirrors `const RENDER_EMIT_KIND`. */
// | export const RENDER_EMIT_KIND = 'render:emit';
pub const RENDER_EMIT_KIND: &str = "render:emit";

// meta-language:translated JavaScript export_statement items=1 sha256=346e74afe5e32dbfec4be209353d188bb80ad346af4e08f6f99b85ea9aec28b8
// | /** Mirrors `const VERIFICATION_KIND`. */
// | export const VERIFICATION_KIND = 'verify:evidence';
pub const VERIFICATION_KIND: &str = "verify:evidence";

// meta-language:translated JavaScript export_statement items=1 sha256=a206a1ef6caa34a4a1545d20bf21f54c3cdc5ea28a6ded6f148c010d36671c95
// | /** Mirrors `const DERIVATIONS_DIR`. */
// | export const DERIVATIONS_DIR = 'data/cache/derivations';
pub const DERIVATIONS_DIR: &str = "data/cache/derivations";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a18f072d5194fb0c8930a6d71790f93ae805785aeb997f767f9a873c16d8f0c5
// | /** Mirrors `const SCHEMA` (read through the host, not embedded). */
// | const SCHEMA_PATH = 'data/seed/derivation-schema.lino';
pub const SCHEMA_PATH: &str = "data/seed/derivation-schema.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=09e4cdff344c514015cba402b8280edf681f2fb003e3dc269f5a78e3d42027a5
// | /** Mirrors `const APPLIED_RULE_FIELD`. */
// | const APPLIED_RULE_FIELD = 'rule';
pub const APPLIED_RULE_FIELD: &str = "rule";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | field access | method call .add() | new Set | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers exponentiation | method call .test() | null | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | global call String() | null | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .lastIndexOf()
// formal-ai:blockers method call .lastIndexOf() | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers JSDoc type {…} | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | field access | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | method call .slice() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers null | object without a $ tag

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | call of a sibling function | destructuring | field access | method call .has() | method call .push() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .find() | method call .push() | null | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .map() | method call .join() | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .join() | method call .map() | nullish coalescing

// meta-language:translated JavaScript function_declaration items=1 sha256=37b99e02d27b8d457127e00b1ecd45d619f6944d2cae859115c424a47aab4624
// | /** `Path::join` for the `/`-separated paths this module builds. @param {string} root @param {string} relative */
// | function joinPath(root, relative) {
// |   if (root === '') return relative;
// |   return root.endsWith('/') ? `${root}${relative}` : `${root}/${relative}`;
// | }
pub fn join_path(root: String, relative: String) -> String {
    if (root == "") {
        relative.clone()
    } else {
        if root.ends_with("/") {
            format!("{}{}", root, relative)
        } else {
            format!("{}{}", (format!("{}{}", root, (String::from("/")))), relative)
        }
    }
}

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers method call .test() | null | regular expression

// meta-language:translated JavaScript export_statement items=1 sha256=629b50d2ce83266a05e86adf04bdbb234dbd49ecd49b5c7339826b5b6b2e05b9
// | /** The refusal `persist` returns for an id `storePath` will not address. */
// | export const UNUSABLE_ANSWER_ID = 'derivation_answer_id_unusable';
pub const UNUSABLE_ANSWER_ID: &str = "derivation_answer_id_unusable";

// formal-ai:workaround string-methods JavaScript function_declaration items=1 sha256=86c13561216c13ea6e19f6d113728bc3c23a64e5343b445e053f76bee597b3e1
// | /** The parent directory of a `/`-separated path (`Path::parent`). @param {string} file */
// | function parentPath(file) {
// |   const index = file.lastIndexOf('/');
// |   return index <= 0 ? '' : file.slice(0, index);
// | }
// ~ /** The parent directory of a `/`-separated path (`Path::parent`). @param {string} file */
// ~ function parentPath(file) {
// ~   const index = waStrLastIndexOf(file, '/');
// ~   return index <= 0 ? '' : waStrSlice(file, 0, index);
// ~ }
pub fn parent_path(file: String) -> String {
    {
        let index = crate::wa_str_last_index_of(file.clone(), String::from("/"));
        if (index <= (0f64)) {
            String::from("")
        } else {
            crate::wa_str_slice(file.clone(), 0f64, index)
        }
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | field access | method call .createDirAll() | method call .writeText() | null | nullish coalescing | object without a $ tag | optional chaining | try statement | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | field access | let without a value | method call .readText() | null | object without a $ tag | try statement | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | null | object without a $ tag

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: function value DERIVATIONS_DIR: functions and namespaces are only portable when a function is called
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .find() | arrow function | call of a sibling function | call of an imported function | let without a value | method call .find() | method call .slice() | method call .split() | null | nullish coalescing | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | method call .push() | null
