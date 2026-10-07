// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=abd403edc709e3ef17da09d39e9e0b108000f681e1eed274f656f2f69abe76a6 bytes=21864
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript export_statement items=1 sha256=265a118f9ae88c73ba37b8d348344eeb9dd0b8171f796db4f986ae4821882891
// | export const DEFAULT_MODEL = 'formal-ai';
pub const DEFAULT_MODEL: &str = "formal-ai";

// meta-language:translated JavaScript export_statement items=1 sha256=0f916e1df526977fba14dc751d6bc178a4f524dba164cb06fc1d63b2c83d4b52
// | export const DEFAULT_BASE_URL = 'http://127.0.0.1:8080';
pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8080";

// meta-language:translated JavaScript export_statement items=1 sha256=57ece8e01de29956448ccace6659cdfeea557c43101212170791a2a563103b0a
// | /** `crate::research_learning::DEFAULT_RESEARCH_TIME_LIMIT_SECONDS`. */
// | export const DEFAULT_TIME_LIMIT_SECONDS = 60 * 60;
pub static DEFAULT_TIME_LIMIT_SECONDS: std::sync::LazyLock<f64> = std::sync::LazyLock::new(|| (60f64 * 60f64));

// meta-language:translated JavaScript lexical_declaration items=1 sha256=41937a7ef4e8ba30ab337995747e8951d1a11d76e762268c40536412c89d07dd
// | const VERIFICATION_TASK_ENV = 'FORMAL_AI_VERIFICATION_TASK';
pub const VERIFICATION_TASK_ENV: &str = "FORMAL_AI_VERIFICATION_TASK";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f30b5c4de10bbe88b39c8e4f0dfbc9ec7a184cc8badce294fc149fe5b281fcff
// | const NATIVE_SESSION_PREFIX = 'formal-ai: orchestration-session-json:';
pub const NATIVE_SESSION_PREFIX: &str = "formal-ai: orchestration-session-json:";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a2f04c4ed4a732aff5e7bf6be13df61a8a8bb994749c93f85a4fa31d227e235b
// | const PIPE_DRAIN_GRACE_MS = 250;
pub const PIPE_DRAIN_GRACE_MS: f64 = 250f64;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .reduce()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unsupported template escape

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .has()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal let …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal let …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal rest property
