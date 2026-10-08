// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=eb278260b48a33cca02331d2232063d4c8bcdd117ecbf1e8bfea84f9f336c6ef bytes=6869
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextEncoder

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .map() | method call .map() | method call .pop() | method call .slice() | method call .split() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers call of a sibling function | method call .join() | method call .push() | method call .replace() | method call .slice() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .every() | assignment of a field or element | bitwise operator | call of an imported function | method call .encode() | method call .every() | method call .join() | method call .push() | new Error | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers Array.from | call of a sibling function | field access | method call .replace() | method call .test() | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal assignment
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal assignment
// formal-ai:blockers field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers method call .join() | method call .push()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | call of an imported function | field access
