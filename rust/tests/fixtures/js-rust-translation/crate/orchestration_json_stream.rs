// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=8e0c67a3c594cddbcc8500c8d318fe4837a96b2ba31e00e1d91f6f4546cc9ba8 bytes=4701
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:translated JavaScript lexical_declaration items=1 sha256=716f1457e8c9693d8bc4c647e6df42f5e7d82a3458a69524b2f2bfc57d4044d6
// | const LITERALS = ['true', 'false', 'null'];
pub static LITERALS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("true"), String::from("false"), String::from("null")]);

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers method call .has() | sibling value

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: one value is used as a string and as an array; declare the types of the function with JSDoc

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers LITERALS.find | arrow callback of .find() | assignment of a field or element | call of a sibling function | method call .exec() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | JSON.parse | call of a sibling function | method call .slice() | null | object without a $ tag | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal generator function
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers call of a sibling function | field access | optional chaining | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Object.keys
// formal-ai:blockers Buffer.compare | Buffer.from | Object.keys | arrow callback of .map() | arrow callback of .sort() | method call .map() | method call .sort()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.isArray | arrow function | null | typeof operator
