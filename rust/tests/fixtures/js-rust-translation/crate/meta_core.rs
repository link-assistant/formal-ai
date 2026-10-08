// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=ed0d3c6c30cf39636c3e3330aee9d0efd72835bd9050ee07172aaa4a35ee2ec5 bytes=21160
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=5

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

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

// meta-language:translated JavaScript import_statement items=1 sha256=db1db602a17d3ecab07f5c4309d4329ebfaab8c5ec21785ae83f30dcafd889b1
// | import { DEFAULT_SPLIT_DEPTH_BOUND } from './recursive_execution.mjs';
use crate::recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND;

// meta-language:translated JavaScript export_statement items=1 sha256=518d3a1101b953aff3050fc0e1905819721fcc92c9fd1a22824e48a6973ef055
// | /** Mirrors `SolverConfig::default().max_decomposition_depth`. */
// | export const DEFAULT_MAX_DECOMPOSITION_DEPTH = 4;
pub const DEFAULT_MAX_DECOMPOSITION_DEPTH: f64 = 4f64;

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow function | method call .encode() | new TextEncoder

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow function | call of an imported function | field access | null | nullish coalescing | optional chaining

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.from | call of an imported function | field access | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers call of an imported function | field access | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of an imported function | field access | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | arrow callback of .reduce() | call of a sibling function | call of an imported function | destructuring | field access | global call String() | method call .map() | method call .push() | method call .reduce() | null | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .map() | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | field access | global call String() | method call .push() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | arrow function | field access | method call .every()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | arrow callback of .filter() | call of a sibling function | call of an imported function | field access | global call String() | method call .every() | method call .filter() | method call .push() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow function | call of a sibling function | call of an imported function | destructuring | field access | global call String() | method call .filter() | method call .forEach() | method call .push() | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | destructuring | field access | global call String() | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow callback of .sort() | arrow function | assignment of a field or element | call of an imported function | field access | method call .filter() | method call .find() | method call .map() | method call .sort() | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow callback of .map() | call of a sibling function | call of an imported function | destructuring | field access | global call String() | method call .filter() | method call .flatMap() | method call .map() | method call .push() | nullish coalescing | object spread | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers call of an imported function | field access | null | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | call of an imported function | field access | global call String() | method call .push() | null | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | global call String() | method call .push() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .filter() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | field access | global call String() | method call .filter() | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | global call String() | method call .push() | object without a $ tag
