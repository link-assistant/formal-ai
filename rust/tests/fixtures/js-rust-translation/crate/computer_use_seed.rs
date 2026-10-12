// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1c702236d3ea22d0e781e966b9e7004de0190d2f87d8cd727c7045536b9fbcaf bytes=9772
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

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

// meta-language:translated JavaScript lexical_declaration items=1 sha256=3101dcb60c56ddac57d98cdcd675df2b1612ac0d7c7dca9a2735aa682571799f
// | const TASKS_PATH = 'data/seed/computer-use-tasks.lino';
pub const TASKS_PATH: &str = "data/seed/computer-use-tasks.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=246adad424cf39ade3b0e7789f714c81f37e165cdec30cd9731e32aaffb4e7f4
// | const TOOLS_PATH = 'data/seed/tools.lino';
pub const TOOLS_PATH: &str = "data/seed/tools.lino";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers JSON.parse | null | try statement | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers JSON.parse | try statement | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .reduce() | method call .join() | method call .reduce() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | method call .get() | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | method call .get() | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers call of an imported function | method call .join()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow callback of .sort() | assignment of a field or element | call of an imported function | method call .find() | method call .push() | method call .sort()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | field access | global call String() | let without a value | method call .padStart() | method call .push() | method call .set() | new Map | null | object without a $ tag | undefined

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal JSON.stringify
// formal-ai:blockers JSON.stringify | arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread
// formal-ai:blockers field access | global call structuredClone() | object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers call of a sibling function | destructuring | field access | method call .map() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | method call .get() | null | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | field access | global call String()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .get()
// formal-ai:blockers call of a sibling function | field access | method call .get() | null | nullish coalescing | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | call of an imported function | method call .indexOf() | null | nullish coalescing | undefined
