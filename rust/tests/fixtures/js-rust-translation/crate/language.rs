// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=51244623f503f1e8f217cd41d53b6f4f0705dbeb5feef8f6e647f0abf184e163 bytes=10600
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds array-push items=1; string-methods items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround-prelude begin
/// Reads of JavaScript arrays, which the portable core never mutates.
pub mod ml_array {
    /// The element at an index; a read outside the array, undefined in
    /// JavaScript, aborts.
    pub fn at<T: Clone>(values: &[T], index: Option<usize>) -> T {
        match index.and_then(|index| values.get(index)) {
            Some(value) => value.clone(),
            None => panic!("array index out of range"),
        }
    }

    /// The index a Number names: a non-negative integer, -0 included.
    pub fn number_index(index: f64) -> Option<usize> {
        if index >= 0.0 && index.fract() == 0.0 && index < 9007199254740992.0 {
            Some(index as usize)
        } else {
            None
        }
    }

    pub fn append<T>(mut left: Vec<T>, right: Vec<T>) -> Vec<T> {
        left.extend(right);
        left
    }
}

pub fn wa_str_index_of(text: String, search: String) -> f64 {
    let units: Vec<u16> = text.encode_utf16().collect();
    let needle: Vec<u16> = search.encode_utf16().collect();
    if needle.is_empty() { return 0.0; }
    if needle.len() > units.len() { return -1.0; }
    units.windows(needle.len()).position(|window| window == needle.as_slice()).map_or(-1.0, |at| at as f64) }

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

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:translated JavaScript export_statement items=1 sha256=a352c99d140ed49b027e7695923a4339383b233523a9bdac690072abf69cd7a9
// | export const UNKNOWN = 'unknown';
pub const UNKNOWN: &str = "unknown";

// meta-language:translated JavaScript export_statement items=1 sha256=8f6f2ef506bd33e287fdbdbd8de2738f0b4f5467a7c3c450c74f57c50dc9eda4
// | export const ENGLISH = 'en';
pub const ENGLISH: &str = "en";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | method call .test() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | method call .slice() | null

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers global call parseInt() | method call .slice() | method call .test() | null | regular expression

// formal-ai:workaround array-push+string-methods JavaScript function_declaration items=2 sha256=c0e8c96a5c0d4b2663a01e5439989f3b3a33adac893535de111b3753176a7cfa
// | /** Mirrors `fn parse_quoted_list`. @param {string} value */
// | function parseQuotedList(value) {
// |   const items = [];
// |   let rest = value;
// |   for (let open = rest.indexOf('"'); open >= 0; open = rest.indexOf('"')) {
// |     const after = rest.slice(open + 1);
// |     const close = after.indexOf('"');
// |     if (close < 0) break;
// |     items.push(after.slice(0, close));
// |     rest = after.slice(close + 1);
// |   }
// |   return items;
// | }
// ~ /** Mirrors `fn parse_quoted_list`. @param {string} value */
// ~ function parseQuotedList(value) {
// ~   let items = [];
// ~   let rest = value;
// ~   for (let open = waStrIndexOf(rest, '"'); open >= 0; open = waStrIndexOf(rest, '"')) {
// ~     const after = waStrSliceFrom(rest, open + 1);
// ~     const close = waStrIndexOf(after, '"');
// ~     if (close < 0) break;
// ~     items = [...items, waStrSlice(after, 0, close)];
// ~     rest = waStrSliceFrom(after, close + 1);
// ~   }
// ~   return items;
// ~ }
pub fn parse_quoted_list(value: String) -> Vec<String> {
    {
        let items = Vec::<String>::new();
        {
            let rest = value.clone();
            {
                let open = crate::wa_str_index_of(rest.clone(), String::from("\""));
                {
                    let items_2 = crate::ml_parse_quoted_list_loop1(items.clone(), rest.clone(), open);
                    items_2.clone()
                }
            }
        }
    }
}

pub fn ml_parse_quoted_list_loop1(mut items: Vec<String>, mut rest: String, mut open: f64) -> Vec<String> {
    loop {
        return if (open >= (0f64)) {
            {
                let after = crate::wa_str_slice_from(rest.clone(), (open + 1f64));
                {
                    let close = crate::wa_str_index_of(after.clone(), String::from("\""));
                    if (close < (0f64)) {
                        items.clone()
                    } else {
                        {
                            let items_2 = crate::ml_array::append(items.clone(), vec![crate::wa_str_slice(after.clone(), 0f64, close)]);
                            {
                                let rest_2 = crate::wa_str_slice_from(after.clone(), (close + 1f64));
                                {
                                    let open_2 = crate::wa_str_index_of(rest_2.clone(), String::from("\""));
                                    {
                                        (items, rest, open) = (items_2.clone(), rest_2.clone(), open_2);
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            items.clone()
        };
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .filter() | method call .push() | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers call of a sibling function | field access | method call .codePointAt()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | null | nullish coalescing | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .map()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .find() | arrow callback of .some() | call of a sibling function | field access | method call .filter() | method call .find() | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | nullish coalescing | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | nullish coalescing | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .find() | assignment of a field or element | call of a sibling function | field access | method call .find() | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow function | field access | method call .find() | nullish coalescing | optional chaining

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .reduce() | arrow function | field access | method call .filter() | method call .reduce()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | arrow function | field access | method call .reduce()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .slice() | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .some() | call of a sibling function | field access | method call .indexOf() | method call .pop() | method call .slice() | method call .some() | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | call of a sibling function | field access | method call .filter() | method call .some() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .filter() | arrow callback of .reduce() | call of a sibling function | field access | method call .filter() | method call .reduce() | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | global call String()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.from | arrow callback of .every() | arrow callback of .find() | arrow callback of .replace() | call of a sibling function | field access | method call .encode() | method call .every() | method call .find() | method call .replace() | method call .slice() | new TextEncoder | null | object without a $ tag | regular expression | undefined
