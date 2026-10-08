// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=d8ceac307c4bbd19c9f908d12cf19ada3f538336979c7a6e1caedf86a352aeed bytes=5072
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=88d7c2f27a7ae1e51566e790bd48dd70d9745566600048271e7ae4691001c386
// | const CACHE_FORMAT_VERSION = 'source-capture-v2';
pub const CACHE_FORMAT_VERSION: &str = "source-capture-v2";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=8941b16b237d5ea7af3be34a374ca45cbf365fc134fe1fd1132a38960aa1a36c
// | const LEGACY_CACHE_FORMAT_VERSION = 'source-capture-v1';
pub const LEGACY_CACHE_FORMAT_VERSION: &str = "source-capture-v1";

// meta-language:translated JavaScript export_statement items=1 sha256=8ef5d181cc87d07555e2afb2e02b7a9fa30ab1e2c421e5c538fd39d340c92c3f
// | /** Mirrors `DEFAULT_TTL_SECONDS`: sixty days. */
// | export const DEFAULT_TTL_SECONDS = 60 * 60 * 24 * 60;
pub static DEFAULT_TTL_SECONDS: std::sync::LazyLock<f64> = std::sync::LazyLock::new(|| (((60f64 * 60f64) * 24f64) * 60f64));

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers bitwise operator | method call .encode() | method call .padStart() | new TextEncoder

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice() | method call .test() | null | regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal rest parameter
// formal-ai:blockers arrow function | method call .join()

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers arrow function | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | method call .readBytes() | method call .readText() | method call .replace() | method call .slice() | method call .split() | method call .test() | null | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers method call .decode() | new TextDecoder | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .createDirAll()
// formal-ai:blockers call of a sibling function | field access | method call .createDirAll() | method call .encode() | method call .writeBytes() | new TextEncoder

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal destructured parameter
// formal-ai:blockers JSDoc type {…} | Number.parseInt | arrow function | call of a sibling function | call of an imported function | field access | global call String() | method call .get() | method call .now() | null | object without a $ tag | try statement
