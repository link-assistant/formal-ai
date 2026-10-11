// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=f0935e4834fb242eb0b3669e4fa0ad30540b26dff7c0c06dfc305b9ba0e942b7 bytes=6797
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

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | call of an imported function | field access | method call .filter() | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of an imported function | field access | method call .indexOf() | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.from | JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | call of an imported function | destructuring | field access | method call .filter() | method call .indexOf() | method call .map() | method call .pop() | method call .slice() | null

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unsupported template escape \u
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .encode() | method call .exec() | method call .replace() | method call .slice() | new TextEncoder | null | optional chaining | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of an imported function | method call .indexOf() | method call .slice() | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .indexOf() | method call .push() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | call of an imported function | method call .encode() | method call .filter() | method call .map() | method call .slice() | method call .test() | new TextEncoder | nullish coalescing | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers Array.from | imported value | method call .indexOf() | method call .slice() | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers Array.from | arrow callback of .find() | call of a sibling function | call of an imported function | field access | method call .find() | method call .repeat() | method call .slice() | method call .some() | method call .test() | null | regular expression
