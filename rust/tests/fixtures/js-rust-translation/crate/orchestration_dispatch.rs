// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=cbe784df81ebf8fb786543a275f8fa436f33a49de50cceed447a17527c1a91c9 bytes=12697
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=9

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers default or namespace import

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers default or namespace import

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
// formal-ai:refusal new expression
// formal-ai:blockers call of an imported function | field access | global call String() | imported value | method call .join() | new Map | new Set | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Buffer.compare | Buffer.from | arrow callback of .filter() | arrow callback of .sort() | arrow function | field access | method call .filter() | method call .sort() | null

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers Array.from | arrow callback of .from() | method call .join() | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .padStart()
// formal-ai:blockers arrow function | global call String() | method call .padStart()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .filter() | arrow function | field access | imported value | method call .filter() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of an imported function | field access | imported value | let without a value | method call .basename() | method call .cwd() | method call .dirname() | method call .existsSync() | method call .isAbsolute() | method call .join() | method call .push() | method call .realpathSync() | method call .reverse() | method call .split() | new DispatchError | try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers call of an imported function | imported value | method call .existsSync() | method call .join() | method call .mkdirSync() | method call .readdirSync() | new DispatchError | object without a $ tag | throw of a non-error value | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal assignment
// formal-ai:blockers assignment of a field or element | call of an imported function | field access | method call .get() | new Set | null | nullish coalescing

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers arrow function | field access

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers Buffer.compare | Buffer.from | arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .get() | method call .keys() | method call .map() | method call .set() | method call .sort() | new DispatchError | new Map | object without a $ tag | sibling value | try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | call of an imported function | field access | method call .find() | new DispatchError | try statement

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal instanceof operator
// formal-ai:blockers arrow function | imported value | instanceof operator | new DispatchError

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | Promise.allSettled | arrow callback of .every() | arrow callback of .map() | arrow callback of .sort() | async function | call of a sibling function | call of an imported function | destructuring | field access | imported value | let without a value | method call .add() | method call .entries() | method call .every() | method call .has() | method call .join() | method call .map() | method call .push() | method call .realpathSync() | method call .sort() | method call .writeFileSync() | new DispatchError | new Set | null | object without a $ tag | throw of a non-error value | try statement
