// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=0028f1e05a2e6271b17a9fae9cdf2daf6b8a4e5f38984cb4c16e215967a01c34 bytes=8275
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=06b879771f1d500c4d96ab5ad1ace40c3587a533181ab8d0edfb12b43f1cd12a
// | const CONFIG_FORMATS = ['toml', 'json', 'shell_env'];
pub static CONFIG_FORMATS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("toml"), String::from("json"), String::from("shell_env")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=7be59bf4791978b16891ab6eacae3e89fd3ad204577e566e3de07e168b232fcb
// | const MODE_ARG_POSITIONS = ['before_invocation', 'before_user_args'];
pub static MODE_ARG_POSITIONS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("before_invocation"), String::from("before_user_args")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=622bf2511216eeb40a438e9f953bb64a1f5ba5926da4df408bf6e18c81e16caa
// | const MODEL_ARG_POSITIONS = ['before_args', 'after_first_arg'];
pub static MODEL_ARG_POSITIONS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("before_args"), String::from("after_first_arg")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
