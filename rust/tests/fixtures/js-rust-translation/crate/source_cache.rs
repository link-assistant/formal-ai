// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=d8ceac307c4bbd19c9f908d12cf19ada3f538336979c7a6e1caedf86a352aeed bytes=5072
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

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

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal rest parameter

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .createDirAll()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal destructured parameter
