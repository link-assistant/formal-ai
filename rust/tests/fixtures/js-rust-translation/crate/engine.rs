// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=66685878cbccb44886d3142c9121ea273c81fff5c79685dfc6981ef482d97582 bytes=2764
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds string-methods items=1

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

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// formal-ai:workaround string-methods JavaScript function_declaration items=1 sha256=e1001b40a62a22b64237088c0828198320d5d0b3b6f9ddbcf3d79d6145d84278
// | /** Mirrors `const fn is_script_combining_mark` in rust/src/engine.rs. @param {string} character */
// | function isScriptCombiningMark(character) {
// |   const code = character.codePointAt(0);
// |   return (code >= 0x0300 && code <= 0x036f)
// |     || (code >= 0x0900 && code <= 0x094f)
// |     || (code >= 0x0951 && code <= 0x0957)
// |     || (code >= 0x0962 && code <= 0x0963)
// |     || (code >= 0x0980 && code <= 0x09ff)
// |     || (code >= 0x0a00 && code <= 0x0a7f)
// |     || (code >= 0x0a80 && code <= 0x0aff)
// |     || (code >= 0x0b00 && code <= 0x0b7f);
// | }
// ~ /** Mirrors `const fn is_script_combining_mark` in rust/src/engine.rs. @param {string} character */
// ~ function isScriptCombiningMark(character) {
// ~   const code = waStrCodePointAt(character, 0);
// ~   return (code >= 0x0300 && code <= 0x036f)
// ~     || (code >= 0x0900 && code <= 0x094f)
// ~     || (code >= 0x0951 && code <= 0x0957)
// ~     || (code >= 0x0962 && code <= 0x0963)
// ~     || (code >= 0x0980 && code <= 0x09ff)
// ~     || (code >= 0x0a00 && code <= 0x0a7f)
// ~     || (code >= 0x0a80 && code <= 0x0aff)
// ~     || (code >= 0x0b00 && code <= 0x0b7f);
// ~ }
// formal-ai:presentation:v1 rawSHA256=e1001b40a62a22b64237088c0828198320d5d0b3b6f9ddbcf3d79d6145d84278 rawBytes=548 renderedSHA256=4e73cf79a7a8aa3b0e92830bbb3a5f172e827026f331ee3a5e7a836e78eaec13
// formal-ai:presentation-raw cHViIGZuIGlzX3NjcmlwdF9jb21iaW5pbmdfbWFyayhjaGFyYWN0ZXI6IFN0cmluZykgLT4gYm9vbCB7CiAgICB7CiAgICAg
// formal-ai:presentation-raw ICAgbGV0IGNvZGUgPSBjcmF0ZTo6d2Ffc3RyX2NvZGVfcG9pbnRfYXQoY2hhcmFjdGVyLmNsb25lKCksIDBmNjQpOwogICAg
// formal-ai:presentation-raw ICAgICgoKCgoKCgoKGNvZGUgPj0gKDc2OGY2NCkpICYmIChjb2RlIDw9ICg4NzlmNjQpKSkgfHwgKChjb2RlID49ICgyMzA0
// formal-ai:presentation-raw ZjY0KSkgJiYgKGNvZGUgPD0gKDIzODNmNjQpKSkpIHx8ICgoY29kZSA+PSAoMjM4NWY2NCkpICYmIChjb2RlIDw9ICgyMzkx
// formal-ai:presentation-raw ZjY0KSkpKSB8fCAoKGNvZGUgPj0gKDI0MDJmNjQpKSAmJiAoY29kZSA8PSAoMjQwM2Y2NCkpKSkgfHwgKChjb2RlID49ICgy
// formal-ai:presentation-raw NDMyZjY0KSkgJiYgKGNvZGUgPD0gKDI1NTlmNjQpKSkpIHx8ICgoY29kZSA+PSAoMjU2MGY2NCkpICYmIChjb2RlIDw9ICgy
// formal-ai:presentation-raw Njg3ZjY0KSkpKSB8fCAoKGNvZGUgPj0gKDI2ODhmNjQpKSAmJiAoY29kZSA8PSAoMjgxNWY2NCkpKSkgfHwgKChjb2RlID49
// formal-ai:presentation-raw ICgyODE2ZjY0KSkgJiYgKGNvZGUgPD0gKDI5NDNmNjQpKSkpCiAgICB9Cn0=
// formal-ai:presentation-raw end
pub fn is_script_combining_mark(character: String) -> bool {
    {
        let code = crate::wa_str_code_point_at(character.clone(), 0f64);
        (((((((((code >= (768f64)) &&
    (code <= (879f64))) ||
    ((code >= (2304f64)) &&
    (code <= (2383f64)))) ||
    ((code >= (2385f64)) &&
    (code <= (2391f64)))) ||
    ((code >= (2402f64)) &&
    (code <= (2403f64)))) ||
    ((code >= (2432f64)) &&
    (code <= (2559f64)))) ||
    ((code >= (2560f64)) &&
    (code <= (2687f64)))) ||
    ((code >= (2688f64)) &&
    (code <= (2815f64)))) ||
    ((code >= (2816f64)) &&
    (code <= (2943f64))))
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .from() | call of a sibling function | global call String() | global value Boolean | method call .filter() | method call .join() | method call .replaceAll() | method call .split() | method call .test() | regular expression | sibling value

// meta-language:translated JavaScript lexical_declaration items=1 sha256=cb9fdcb64553fc365383881e4f579ec72c03aaf69efbc625b7165184b32a7a77
// | const LANGUAGES_FILE = 'data/seed/languages.lino';
pub const LANGUAGES_FILE: &str = "data/seed/languages.lino";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | global value Boolean | method call .filter() | method call .indexOf() | method call .push() | method call .replace() | method call .slice() | method call .split() | null | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .every() | arrow callback of .find() | call of a sibling function | method call .every() | method call .find() | method call .join() | method call .push() | method call .split()
