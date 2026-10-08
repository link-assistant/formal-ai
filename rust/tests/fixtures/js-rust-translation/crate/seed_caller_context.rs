// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=76e5d9b789858eee7a6efa2b334a2930df596ac7c58b36c1455d34ad01f5c1b3 bytes=3973
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2; string-methods items=1

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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow callback of .map() | arrow function | field access | method call .filter() | method call .flatMap() | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .filter() | method call .map() | object without a $ tag

// formal-ai:workaround string-methods JavaScript function_declaration items=1 sha256=09cb445220d41e9e0d176bafbd7e105c30c7e325b6ffbf98ad06aeb950aea7c9
// | /** Mirrors `const fn is_unspaced_script`. @param {string} character */
// | function isUnspacedScript(character) {
// |   const cp = character.codePointAt(0);
// |   return (cp >= 0x3400 && cp <= 0x9fff) || (cp >= 0xf900 && cp <= 0xfaff);
// | }
// ~ /** Mirrors `const fn is_unspaced_script`. @param {string} character */
// ~ function isUnspacedScript(character) {
// ~   const cp = waStrCodePointAt(character, 0);
// ~   return (cp >= 0x3400 && cp <= 0x9fff) || (cp >= 0xf900 && cp <= 0xfaff);
// ~ }
pub fn is_unspaced_script(character: String) -> bool {
    {
        let cp = crate::wa_str_code_point_at(character.clone(), 0f64);
        (((cp >= (13312f64)) && (cp <= (40959f64))) || ((cp >= (63744f64)) && (cp <= (64255f64))))
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | call of an imported function | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .find() | arrow function | call of a sibling function | call of an imported function | field access | method call .find() | method call .some() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated string literal
// formal-ai:blockers Array.from | arrow callback of .some() | call of a sibling function | field access | method call .some() | method call .split() | object without a $ tag | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | field access | method call .some()
