// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=16720dc659d43025219b6f9e312941ada9ed0381871f546b1ccfce38f999c517 bytes=21981
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=5b12825948f4a6b4b426e4b72501e049a844a06601b5f08c8e1ac9de50fd870e
// | export const CNL_LANGUAGE_PATH = 'data/seed/cnl-language-{language}.lino';
pub const CNL_LANGUAGE_PATH: &str = "data/seed/cnl-language-{language}.lino";

// meta-language:translated JavaScript export_statement items=1 sha256=0b5e513042bc6261ee096caf2ba078f8902437f0ad6d8049d4f2727745e21a90
// | /** Term register of input-only surfaces; realization never selects it. */
// | export const INPUT_REGISTER = 'input';
pub const INPUT_REGISTER: &str = "input";

// meta-language:carried JavaScript lexical_declaration (syntax)
// formal-ai:refusal syntax: unsupported string escape \p
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:translated JavaScript lexical_declaration items=1 sha256=90f625288f5e681ab23464bfaa6f814cfcbca3f898b7ba4e7eaf25a53a24e664
// | const MAXIMUM_SPAN = 24;
pub const MAXIMUM_SPAN: f64 = 24f64;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of an imported function | global call String() | method call .split() | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers call of a sibling function | global value Boolean | method call .filter()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .indexOf() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .map() | method call .split() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal call of a computed function
// formal-ai:blockers Object.fromEntries | arrow callback of .map() | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .map() | method call .split() | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replaceAll()
// formal-ai:blockers ++ operator | arrow callback of .every() | arrow callback of .find() | arrow callback of .flatMap() | arrow callback of .map() | field access | global call String() | method call .every() | method call .find() | method call .flatMap() | method call .get() | method call .map() | method call .match() | method call .push() | method call .replaceAll() | nullish coalescing | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .matchAll()
// formal-ai:blockers call of a sibling function | field access | method call .matchAll() | method call .push() | method call .slice() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow callback of .forEach() | arrow callback of .map() | arrow callback of .sort() | arrow function | assignment of a field or element | call of a sibling function | destructuring | field access | global call String() | method call .filter() | method call .flatMap() | method call .forEach() | method call .get() | method call .has() | method call .join() | method call .map() | method call .match() | method call .push() | method call .replaceAll() | method call .set() | method call .sort() | method call .split() | new Map | new Set | nullish coalescing | object spread | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Object.keys
// formal-ai:blockers Object.keys | arrow callback of .map() | field access | method call .join() | method call .map() | method call .sort()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | field access | method call .get() | method call .join() | method call .map() | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .every() | arrow callback of .filter() | arrow function | call of a sibling function | destructuring | field access | method call .add() | method call .every() | method call .filter() | method call .get() | method call .has() | method call .join() | method call .push() | method call .set() | method call .slice() | method call .test() | new Map | new Set | nullish coalescing | object spread | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | field access | method call .every() | optional chaining | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .get()
// formal-ai:blockers Object.fromEntries | arrow callback of .every() | arrow callback of .filter() | arrow callback of .map() | call of a sibling function | field access | method call .clear() | method call .every() | method call .filter() | method call .get() | method call .map() | method call .push() | method call .set() | method call .test() | nullish coalescing | object without a $ tag | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .get() | method call .keys() | method call .map() | method call .set() | method call .sort() | new Map

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow callback of .replace() | method call .replace() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | field access | method call .filter() | method call .flatMap() | nullish coalescing

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated string literal
// formal-ai:blockers arrow callback of .replace() | call of a sibling function | field access | global call String() | method call .get() | method call .indexOf() | method call .replace() | method call .slice() | method call .test() | null | nullish coalescing | object without a $ tag | optional chaining | regular expression | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers call of a sibling function | destructuring | method call .add() | method call .has() | method call .push() | new Set | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .map() | field access | method call .flatMap() | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | global call String() | method call .join() | method call .match() | method call .push() | method call .slice() | method call .test() | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replaceAll()
// formal-ai:blockers arrow function | global call String() | method call .filter() | method call .match() | method call .replaceAll() | nullish coalescing | optional chaining | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .some() | arrow function | field access | method call .get() | method call .has() | method call .some() | method call .test() | optional chaining | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers Object.values | arrow callback of .every() | arrow callback of .some() | field access | method call .every() | method call .has() | method call .some()

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated string literal
// formal-ai:blockers arrow callback of .every() | arrow callback of .filter() | arrow callback of .map() | arrow callback of .some() | call of a sibling function | destructuring | field access | method call .count() | method call .entries() | method call .every() | method call .filter() | method call .flatMap() | method call .get() | method call .has() | method call .map() | method call .minimum() | method call .some() | method call .test() | new Set | null | nullish coalescing | object without a $ tag | regular expression | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal array destructuring
// formal-ai:blockers arrow callback of .filter() | assignment of a field or element | call of a sibling function | destructuring | field access | method call .add() | method call .filter() | method call .has() | method call .push() | method call .shift() | new Set

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow callback of .replace() | call of a sibling function | field access | method call .replace() | nullish coalescing | regular expression

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated template literal
// formal-ai:blockers call of a sibling function | field access | method call .test() | null | object without a $ tag | regular expression | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow callback of .replace() | call of a sibling function | field access | method call .replace() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | field access | method call .filter()
