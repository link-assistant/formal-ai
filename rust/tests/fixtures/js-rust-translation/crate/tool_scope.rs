// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=7ed3359f31ab39fa3ed97736f0afb3736b4040155731bbd792d5f6ebda511523 bytes=7051
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=3

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:translated JavaScript import_statement items=1 sha256=ea67ac58c941251801bc82c22574a8e03c0fd9db18bf89015315cef5823b17a3
// | import { lower as canonicalUnaryLower } from './rust_str.mjs';
use crate::rust_str::lower as canonical_unary_lower;

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

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow function | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow function | call of an imported function | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers assignment of a field or element | call of a sibling function | call of an imported function | field access | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | field access | method call .some() | nullish coalescing | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | field access | method call .some() | sibling value

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | Object.prototype | arrow callback of .filter() | call of a sibling function | field access | method call .call() | method call .filter() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow function | call of an imported function | global value Boolean | method call .filter() | method call .slice() | method call .split() | method call .test() | null | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .flatMap() | global value Boolean | method call .filter() | method call .flatMap() | method call .map() | method call .split() | regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Object.prototype.hasOwnProperty.call
// formal-ai:blockers Object.prototype | arrow function | call of an imported function | field access | method call .call() | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers Array.isArray | arrow callback of .filter() | arrow callback of .find() | call of a sibling function | call of an imported function | method call .filter() | method call .find() | method call .map() | typeof operator | undefined
