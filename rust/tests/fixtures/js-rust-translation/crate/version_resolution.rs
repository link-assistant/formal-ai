// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=543ac1b912a9831f7a60b91961765af2364fcfd5aaba19920d8f316669602059 bytes=12081
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1; string-methods items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround-prelude begin
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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:translated JavaScript lexical_declaration items=1 sha256=96d5c1c208b6092a1de30c865ab273d1e9f3ba2b8e3a916f97de2c437207482f
// | const TOOLCHAINS = 'data/seed/toolchains.lino';
pub const TOOLCHAINS: &str = "data/seed/toolchains.lino";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | field access | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | call of an imported function | field access | method call .find() | null | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d13f659d5ba21cf75f8e21f12baf7494d8e95cc9ee421318dd4e207a1f148218
// | const PIN_IDS = [
// |   ['checkout', 'actions_checkout'],
// |   ['setup_java', 'actions_setup_java'],
// |   ['setup_kotlin', 'setup_kotlin_action'],
// |   ['setup_python', 'actions_setup_python'],
// |   ['python_interpreter', 'python_interpreter'],
// |   ['kotlin', 'kotlin_compiler'],
// |   ['java_lts', 'java_lts'],
// |   ['setup_coursier', 'setup_coursier_action'],
// |   ['scala', 'scala_compiler'],
// | ];
pub static PIN_IDS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(|| vec![vec![String::from("checkout"), String::from("actions_checkout")], vec![String::from("setup_java"), String::from("actions_setup_java")], vec![String::from("setup_kotlin"), String::from("setup_kotlin_action")], vec![String::from("setup_python"), String::from("actions_setup_python")], vec![String::from("python_interpreter"), String::from("python_interpreter")], vec![String::from("kotlin"), String::from("kotlin_compiler")], vec![String::from("java_lts"), String::from("java_lts")], vec![String::from("setup_coursier"), String::from("setup_coursier_action")], vec![String::from("scala"), String::from("scala_compiler")]]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers JSON.parse | field access | null | try statement | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object spread
// formal-ai:blockers field access | null | object spread | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal call of a computed function
// formal-ai:blockers call of a sibling function | field access | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal call of a computed function
// formal-ai:blockers JSON.parse | field access | global call String() | method call .push() | null | object without a $ tag | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers assignment of a field or element | call of a sibling function | field access | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers destructuring | field access | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | call of a sibling function | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | field access | null | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers PIN_IDS.find | arrow callback of .find() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of an imported function | field access | method call .filter() | method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .flatMap() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | call of an imported function | field access | method call .find() | null

// formal-ai:workaround string-methods JavaScript function_declaration items=2 sha256=bc1fbf588ccdfea58af274e84bc7043839ea1b7e5d3780b29dfc244b96b27774
// | /** @param {string} tag */
// | function trimStartV(tag) {
// |   let out = tag;
// |   while (out.startsWith('v')) out = out.slice(1);
// |   return out;
// | }
// ~ /** @param {string} tag */
// ~ function trimStartV(tag) {
// ~   let out = tag;
// ~   while (out.startsWith('v')) out = waStrSliceFrom(out, 1);
// ~   return out;
// ~ }
pub fn trim_start_v(tag: String) -> String {
    {
        let out = tag.clone();
        {
            let out_2 = crate::ml_trim_start_v_loop1(out.clone());
            out_2.clone()
        }
    }
}

pub fn ml_trim_start_v_loop1(mut out: String) -> String {
    loop {
        return if out.starts_with("v") {
            {
                let out_2 = crate::wa_str_slice_from(out.clone(), 1f64);
                {
                    out = out_2.clone();
                    continue;
                }
            }
        } else {
            out.clone()
        };
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | call of a sibling function | call of an imported function | field access | method call .join() | method call .reduce() | method call .replace() | method call .split() | null | regular expression

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .indexOf() | method call .search() | method call .slice() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of an imported function | method call .indexOf() | method call .slice()
