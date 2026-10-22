// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=6917e99549cb876345f279918c7bdb4c5b30020e2006fe99b42f07ab6177f2a5 bytes=6588
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
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow callback of .map() | call of an imported function | field access | method call .filter() | method call .flatMap() | method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .map() | arrow callback of .sort() | arrow function | call of a sibling function | call of an imported function | field access | method call .flatMap() | method call .map() | method call .sort() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow callback of .sort() | arrow function | call of a sibling function | call of an imported function | method call .flatMap() | method call .map() | method call .sort() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers method call .set() | new Map | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .from() | call of a sibling function | call of an imported function | destructuring | method call .at() | method call .get() | method call .has() | method call .indexOf() | method call .join() | method call .slice() | null | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers call of an imported function | method call .replace() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | call of an imported function | destructuring | field access | method call .push() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers call of a sibling function | destructuring | field access | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal several declarators
// formal-ai:blockers arrow callback of .map() | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | field access | method call .flatMap() | method call .map() | method call .sort() | object without a $ tag

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unsupported string escape \x
// formal-ai:blockers arrow callback of .some() | arrow callback of .sort() | call of a sibling function | call of an imported function | destructuring | field access | method call .at() | method call .join() | method call .push() | method call .slice() | method call .some() | method call .sort() | object without a $ tag

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unsupported string escape \x
// formal-ai:blockers call of an imported function | field access | method call .join()
