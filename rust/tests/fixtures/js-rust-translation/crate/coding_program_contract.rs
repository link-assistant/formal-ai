// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1cb7fbb3ef3ab9e5ea27b1fc709c806526518735388f685720bec8f3adef1472 bytes=13898
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=6; string-methods items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround-prelude begin
pub fn wa_str_split(text: String, separator: String) -> Vec<String> {
    if separator.is_empty() {
        return text.encode_utf16().map(|unit| String::from_utf16_lossy(&[unit])).collect();
    }
    text.split(separator.as_str()).map(String::from).collect() }

pub fn wa_strs_join(items: Vec<String>, separator: String) -> String {
    items.join(separator.as_str()) }
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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | field access | null | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | null | nullish coalescing | optional chaining

// formal-ai:workaround string-methods JavaScript lexical_declaration items=1 sha256=f4ef0e4b5a63c97b00c8db77a68055413812986b3af5304063a8b60165db73a4
// | /** @param {string} text @param {string} from @param {string} to */
// | const replaceText = (text, from, to) => text.split(from).join(to);
// ~ /** @param {string} text @param {string} from @param {string} to */
// ~ const replaceText = (text, from, to) => waStrsJoin(waStrSplit(text, from), to);
pub fn replace_text(text: String, from: String, to: String) -> String {
    crate::wa_strs_join(crate::wa_str_split(text.clone(), from.clone()), to.clone())
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of an imported function | field access | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers call of an imported function | method call .join() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | null | nullish coalescing | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | null

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unterminated string literal
// formal-ai:blockers method call .replace() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | method call .filter() | method call .map() | method call .split() | null | regular expression | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | method call .test() | null | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .add() | method call .split() | new Set | null | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .every() | arrow callback of .map() | call of an imported function | field access | method call .every() | method call .map()

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .filter() | method call .join() | method call .map() | null | nullish coalescing | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | field access | null | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers method call .codePointAt() | method call .padStart()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of an imported function | field access | method call .filter() | null | object spread | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .join() | method call .replace() | method call .split() | null | regular expression

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers arrow callback of .map() | assignment of a field or element | call of an imported function | destructuring | global call String() | global value Boolean | method call .filter() | method call .indexOf() | method call .lastIndexOf() | method call .map() | method call .slice() | method call .split() | method call .test() | null | object without a $ tag | regular expression | undefined
