// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=04e92446eaba4e26554cdc2382501697c9938c6bf2b137af3ca457ce054f32bd bytes=14733
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=73bef6fe09d2fd4895ee0b619b32941c072458ebfaac4628f972dca2f0d3cef2
// | const SEED_PATH = 'data/seed/history-formalization.lino';
pub const SEED_PATH: &str = "data/seed/history-formalization.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8ae9b7470652ebb303eb11b75c0809021d4ffb7cee3ae4ab124df3da48a1b304
// | const NUMBER_PLACEHOLDER = '%number%';
pub const NUMBER_PLACEHOLDER: &str = "%number%";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b3407d380205e2a63c05f450a2d2b09ad6a2cd84efec8b650d194a62b574e79c
// | const RECORD_SEPARATOR = '\u001e';
pub const RECORD_SEPARATOR: &str = "\u{1e}";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=178fa8b8bdc3241312c4a9f567a777aed2109bf73ef9868a7beee13829722bad
// | const UNIT_SEPARATOR = '\u001f';
pub const UNIT_SEPARATOR: &str = "\u{1f}";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | field access | method call .filter() | method call .map() | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .push() | method call .set() | new Map | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | global call String() | method call .filter() | method call .map() | method call .push() | method call .reverse() | method call .split() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .exec() | method call .indexOf() | method call .slice() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | arrow callback of .map() | arrow callback of .sort() | field access | method call .find() | method call .map() | method call .push() | method call .sort() | optional chaining | typeof operator | undefined

// meta-language:translated JavaScript lexical_declaration items=1 sha256=e6f2dcf29641818fd080d3fadbae4e7687a7541944ea2dc303bde2a3bd5bb71b
// | const STATEMENT_SEPARATORS = [' ', '.', ':', ')', '*'];
pub static STATEMENT_SEPARATORS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from(" "), String::from("."), String::from(":"), String::from(")"), String::from("*")]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | STATEMENT_SEPARATORS.includes | arrow callback of .find() | field access | global call String() | method call .exec() | method call .find() | method call .push() | method call .slice() | method call .split() | regular expression | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .flatMap() | field access | global call String() | method call .flatMap() | method call .push() | method call .slice() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers method call .split() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow callback of .filter() | arrow function | call of a sibling function | method call .exec() | method call .filter() | method call .indexOf() | method call .join() | method call .replace() | method call .slice() | method call .split() | method call .test() | null | regular expression | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .get()
// formal-ai:blockers arrow callback of .sort() | call of a sibling function | field access | global call String() | method call .get() | method call .set() | method call .sort() | method call .split() | new Map | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | method call .get() | method call .has() | method call .keys() | method call .push() | method call .sort() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | arrow callback of .map() | arrow function | call of a sibling function | field access | method call .find() | method call .join() | method call .map() | method call .push() | method call .split() | null | object without a $ tag | undefined
