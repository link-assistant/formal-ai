// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=7bd6a47c94e92713c2bfe61177f5ec500780b6d37b322f829eabf67f98e14d98 bytes=3609
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds string-methods items=3

// formal-ai:workaround-prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
pub fn wa_str_replace_all(text: String, pattern: String, replacement: String) -> String {
    text.replace(pattern.as_str(), replacement.as_str()) }
// formal-ai:workaround-prelude end

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated string literal
// formal-ai:blockers method call .replaceAll() | method call .split() | method call .test() | regular expression

// formal-ai:workaround string-methods JavaScript function_declaration items=1 sha256=3710144a8224276679abd600d64b4ff17cd3546ee870b6f78664cfa6be4ffc43
// | /** Mirrors `fn format_indented_value` in lino-objects-codec `format`. @param {string} value */
// | function formatIndentedValue(value) {
// |   const hasSingle = value.includes("'");
// |   const hasDouble = value.includes('"');
// |   if (hasDouble && !hasSingle) return `'${value}'`;
// |   if (hasSingle && !hasDouble) return `"${value}"`;
// |   if (hasSingle && hasDouble) return `'${value.replaceAll("'", "''")}'`;
// |   return `"${value}"`;
// | }
// ~ /** Mirrors `fn format_indented_value` in lino-objects-codec `format`. @param {string} value */
// ~ function formatIndentedValue(value) {
// ~   const hasSingle = value.includes("'");
// ~   const hasDouble = value.includes('"');
// ~   if (hasDouble && !hasSingle) return `'${value}'`;
// ~   if (hasSingle && !hasDouble) return `"${value}"`;
// ~   if (hasSingle && hasDouble) return `'${waStrReplaceAll(value, "'", "''")}'`;
// ~   return `"${value}"`;
// ~ }
pub fn format_indented_value(value: String) -> String {
    {
        let has_single = value.contains("'");
        {
            let has_double = value.contains("\"");
            if (has_double && !has_single) {
                format!("{}{}", (format!("{}{}", (String::from("'")), value)), (String::from("'")))
            } else {
                if (has_single && !has_double) {
                    format!("{}{}", (format!("{}{}", (String::from("\"")), value)), (String::from("\"")))
                } else {
                    if (has_single && has_double) {
                        format!("{}{}", (format!("{}{}", (String::from("'")), (crate::wa_str_replace_all(value.clone(), String::from("'"), String::from("''"))))), (String::from("'")))
                    } else {
                        format!("{}{}", (format!("{}{}", (String::from("\"")), value)), (String::from("\"")))
                    }
                }
            }
        }
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | destructuring | method call .join() | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | call of a sibling function | method call .map()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | method call .repeat() | null | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | method call .repeat() | null | undefined

// formal-ai:workaround string-methods JavaScript export_statement items=1 sha256=b48fa0f5be9516dc11885b643335047871f167350ba5da68283945db9a7014c1
// | /** Mirrors `fn sanitize_lino_value`. @param {string} value */
// | export function sanitizeLinoValue(value) {
// |   return value.replaceAll('\\', '\\\\').replaceAll('\r', '\\r').replaceAll('\n', '\\n').replaceAll('\t', '\\t');
// | }
// ~ /** Mirrors `fn sanitize_lino_value`. @param {string} value */
// ~ export function sanitizeLinoValue(value) {
// ~   return waStrReplaceAll(waStrReplaceAll(waStrReplaceAll(waStrReplaceAll(value, '\\', '\\\\'), '\r', '\\r'), '\n', '\\n'), '\t', '\\t');
// ~ }
pub fn sanitize_lino_value(value: String) -> String {
    crate::wa_str_replace_all(crate::wa_str_replace_all(crate::wa_str_replace_all(crate::wa_str_replace_all(value.clone(), String::from("\\"), String::from("\\\\")), String::from("\r"), String::from("\\r")), String::from("\n"), String::from("\\n")), String::from("\t"), String::from("\\t"))
}

// formal-ai:workaround string-methods JavaScript export_statement items=1 sha256=d8f6485e46d5641332fe5b4567c2bda33884ba3f1cdd46e9d63b4c934fafdfa7
// | /** Mirrors `fn flatten_lino_value`. @param {string} value */
// | export function flattenLinoValue(value) {
// |   return value.replaceAll('\r', '\\r').replaceAll('\n', '\\n').replaceAll('\t', '\\t');
// | }
// ~ /** Mirrors `fn flatten_lino_value`. @param {string} value */
// ~ export function flattenLinoValue(value) {
// ~   return waStrReplaceAll(waStrReplaceAll(waStrReplaceAll(value, '\r', '\\r'), '\n', '\\n'), '\t', '\\t');
// ~ }
pub fn flatten_lino_value(value: String) -> String {
    crate::wa_str_replace_all(crate::wa_str_replace_all(crate::wa_str_replace_all(value.clone(), String::from("\r"), String::from("\\r")), String::from("\n"), String::from("\\n")), String::from("\t"), String::from("\\t"))
}
