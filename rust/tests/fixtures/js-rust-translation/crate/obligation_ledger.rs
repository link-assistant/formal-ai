// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=2353fbeb50cd0fee554621b001b6a9fa68f0832493270152e2c8a6bde27a6723 bytes=12958
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=5; string-methods items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround-prelude begin
pub fn wa_str_code_point_at(text: String, index: f64) -> f64 {
    let units: Vec<u16> = text.encode_utf16().collect();
    let at = if index.is_nan() { 0.0 } else { index.trunc() };
    if at < 0.0 || at >= units.len() as f64 { return f64::NAN; }
    let first = units[at as usize];
    match units.get(at as usize + 1) {
        Some(&second) if (0xD800..0xDC00).contains(&first) && (0xDC00..0xE000).contains(&second) => {
            f64::from(((u32::from(first) - 0xD800) << 10) + (u32::from(second) - 0xDC00) + 0x10000)
        }
        _ => f64::from(first),
    } }
// formal-ai:workaround-prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextEncoder

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | call of a sibling function | call of an imported function | method call .codePointAt() | method call .filter() | method call .indexOf() | method call .push() | method call .slice() | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers arrow callback of .some() | arrow function | call of an imported function | method call .slice() | method call .some()

// formal-ai:workaround string-methods JavaScript function_declaration items=1 sha256=cd8d2182f8251431a65d5c3ce97284f87d05c9cd05b56e527db0e89ac2e48c8b
// | /** @param {string} character */
// | function isUnspacedScript(character) {
// |   const cp = character.codePointAt(0);
// |   return (cp >= 0x3040 && cp <= 0x30ff) || (cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0x4e00 && cp <= 0x9fff)
// |     || (cp >= 0xac00 && cp <= 0xd7af) || (cp >= 0xf900 && cp <= 0xfaff);
// | }
// ~ /** @param {string} character */
// ~ function isUnspacedScript(character) {
// ~   const cp = waStrCodePointAt(character, 0);
// ~   return (cp >= 0x3040 && cp <= 0x30ff) || (cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0x4e00 && cp <= 0x9fff)
// ~     || (cp >= 0xac00 && cp <= 0xd7af) || (cp >= 0xf900 && cp <= 0xfaff);
// ~ }
pub fn is_unspaced_script(character: String) -> bool {
    {
        let cp = crate::wa_str_code_point_at(character.clone(), 0f64);
        ((((((cp >= (12352f64)) && (cp <= (12543f64))) || ((cp >= (13312f64)) && (cp <= (19903f64)))) || ((cp >= (19968f64)) && (cp <= (40959f64)))) || ((cp >= (44032f64)) && (cp <= (55215f64)))) || ((cp >= (63744f64)) && (cp <= (64255f64))))
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | call of an imported function | method call .slice() | method call .some() | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | assignment of a field or element | call of a sibling function | call of an imported function | method call .push() | method call .slice() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.from | call of a sibling function | call of an imported function | method call .slice() | null | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow function | call of a sibling function | field access | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | call of an imported function | method call .encode() | method call .pop() | method call .slice() | null | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .find() | call of a sibling function | call of an imported function | field access | method call .find() | null | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .map() | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .indexOf() | method call .push() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers field access | method call .every()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers field access | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of an imported function | field access | method call .find()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | call of a sibling function | call of an imported function | field access | method call .join() | method call .reduce() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | field access | global call String() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | call of a sibling function | field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | field access | method call .filter()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | call of a sibling function | field access

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | call of a sibling function

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | call of a sibling function
