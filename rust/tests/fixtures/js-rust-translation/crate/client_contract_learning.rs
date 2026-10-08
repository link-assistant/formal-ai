// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=14280817883388e730081eaec48ae164ec9896b8dedcd4969f6ad751588f87a3 bytes=9707
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers Buffer.compare | Buffer.from | arrow function

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .sort()
// formal-ai:blockers arrow function | method call .sort() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers global value Boolean | method call .filter() | method call .join() | method call .split() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replaceAll()
// formal-ai:blockers method call .replaceAll()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .repeat()
// formal-ai:blockers call of a sibling function | method call .repeat()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .every() | field access | method call .every() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers field access | method call .delete() | method call .has() | method call .slice() | new Set

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | field access | method call .filter() | method call .keys() | method call .map() | method call .set() | new Map | new Set

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers call of a sibling function | destructuring | field access | method call .delete() | method call .get() | method call .has() | method call .slice() | new Map

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unsupported template escape
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | call of an imported function | destructuring | field access | method call .get() | method call .has() | method call .keys() | method call .map() | method call .push() | method call .set() | method call .sort() | method call .values() | new Map | new Set | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | field access | nullish coalescing
