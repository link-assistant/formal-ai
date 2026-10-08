// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=e449f94e28042b7bde536b78f9ace0332046e2f539846fed6f5170562c0ea0a8 bytes=10854
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8ae9b7470652ebb303eb11b75c0809021d4ffb7cee3ae4ab124df3da48a1b304
// | const NUMBER_PLACEHOLDER = '%number%';
pub const NUMBER_PLACEHOLDER: &str = "%number%";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .test() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow function | call of a sibling function | destructuring | method call .slice() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers JSON.parse | null | try statement

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Array.isArray
// formal-ai:blockers Array.isArray | arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers null | optional chaining | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers field access | optional chaining | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers Array.isArray | arrow callback of .filter() | arrow callback of .map() | field access | method call .filter() | method call .map() | optional chaining | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers field access | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | call of an imported function | destructuring | field access | method call .join() | method call .push() | method call .split() | null | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers call of a sibling function | field access | global call String() | let without a value | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers arrow function | field access | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.isArray | JSDoc type {…} | arrow callback of .forEach() | arrow callback of .map() | arrow callback of .sort() | arrow function | call of a sibling function | destructuring | field access | method call .forEach() | method call .get() | method call .keys() | method call .map() | method call .set() | method call .sort() | new Map | null | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers Array.isArray | arrow callback of .filter() | arrow callback of .map() | call of a sibling function | field access | method call .filter() | method call .map() | method call .push() | nullish coalescing | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | destructuring | field access | method call .get() | method call .join() | method call .keys() | method call .map() | method call .set() | method call .sort() | new Map | null | nullish coalescing | optional chaining | undefined
