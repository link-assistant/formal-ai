// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=4b1768fc4a6b833920e4b4d1336c9f9f866a57e44544de2425f02893b74685ad bytes=13494
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:translated JavaScript export_statement items=1 sha256=944558e206c5d5b92ddcf91572bdb24dd3f45d1a6813127db2592aba83d66d85
// | export const CNL_POLICY_PATH = 'data/seed/cnl-output.lino';
pub const CNL_POLICY_PATH: &str = "data/seed/cnl-output.lino";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .map() | call of an imported function | field access | global value Boolean | method call .filter() | method call .map() | method call .split() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .slice() | method call .test() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .find() | arrow callback of .some() | call of a sibling function | field access | method call .find() | method call .some() | null | nullish coalescing | object without a $ tag | typeof operator | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | field access | method call .filter() | method call .indexOf() | method call .push() | method call .slice() | method call .some() | object without a $ tag | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | field access | method call .map()

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unsupported string escape \p
// formal-ai:blockers method call .match() | nullish coalescing | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | field access | method call .filter() | method call .find() | method call .join() | method call .map() | method call .sort() | method call .split() | object without a $ tag

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated string literal
// formal-ai:blockers arrow callback of .find() | arrow callback of .flatMap() | arrow callback of .map() | arrow callback of .some() | call of a sibling function | field access | method call .find() | method call .flatMap() | method call .has() | method call .map() | method call .push() | method call .replaceAll() | method call .some() | method call .split() | method call .test() | new Set | null | object spread | object without a $ tag | optional chaining | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | field access | object without a $ tag

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated template literal
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | field access | method call .replace() | method call .search() | method call .slice() | method call .split() | method call .test() | null | object without a $ tag | regular expression | try statement | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSON.stringify
// formal-ai:blockers JSON.stringify | call of a sibling function | destructuring | field access | method call .join() | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | method call .reverse() | null | nullish coalescing | object spread | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | method call .slice() | optional chaining
