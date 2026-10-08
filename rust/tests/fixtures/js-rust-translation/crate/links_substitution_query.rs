// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=d62b0641c5dfd26054136733b3a3a5c2adc42b38bb968bb31c12d59365ec4080 bytes=5397
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

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers field access

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: one value is used as a string and as an array; declare the types of the function with JSDoc

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | arrow callback of .map() | call of a sibling function | field access | method call .every() | method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | method call .join() | method call .map()

// meta-language:carried JavaScript class_declaration (unsupported)
// formal-ai:refusal top-level class statement
// formal-ai:blockers class

// meta-language:carried JavaScript class_declaration (unsupported)
// formal-ai:refusal top-level class statement
// formal-ai:blockers Array.from | assignment of a field or element | call of an imported function | class | field access | method call .bump() | method call .eat() | method call .join() | method call .parseOperand() | method call .parseOperandName() | method call .parseString() | method call .peek() | method call .push() | method call .skipWhitespace() | method call .slice() | new Failure | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers call of a sibling function | call of an imported function | instanceof operator | method call .atEnd() | method call .parseSide() | method call .skipWhitespace() | new Parser | null | sibling value | throw of a non-error value | try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of an imported function | field access | method call .map() | null
