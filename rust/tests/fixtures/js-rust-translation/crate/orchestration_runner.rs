// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=abd403edc709e3ef17da09d39e9e0b108000f681e1eed274f656f2f69abe76a6 bytes=21864
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=6

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from '…'

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from '…'

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=67031384e051662ec070a331d7e8d4fbe6283b1a5097a9fce7e5d002513b1dd4
// | import { sessionSha256 as replaySessionSha256, SESSION_SCHEMA } from './orchestration_replay.mjs';
use crate::orchestration_replay::SESSION_SCHEMA;

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

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
// formal-ai:blockers JSDoc type {…} | JSON.stringify | arrow callback of .map() | assignment of a field or element | class | export … | field access | imported value | instanceof operator | method call .debugOf() | method call .display() | method call .join() | method call .map() | method call .slice() | method call .split() | null

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | global call String() | object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers global call String() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of an imported function | global call String() | new Set | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread
// formal-ai:blockers object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object spread
// formal-ai:blockers object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | field access | method call .every()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | field access | method call .reduce()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unsupported template escape
// formal-ai:blockers call of an imported function | field access | method call .digest() | method call .push() | method call .repeat() | method call .update() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .has()
// formal-ai:blockers field access | method call .has() | new AgentRunError

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal let …
// formal-ai:blockers call of an imported function | field access | imported value | let without a value | method call .isDirectory() | method call .realpathSync() | method call .statSync() | new AgentRunError | try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .join() | method call .map() | method call .push() | method call .slice() | method call .split() | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of a sibling function | field access | imported value | method call .join() | method call .push() | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers Array.isArray | call of an imported function | null | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal let …
// formal-ai:blockers JSON.parse | call of an imported function | field access | let without a value | null | object without a $ tag | try statement | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .join() | method call .slice() | method call .split() | null | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement
// formal-ai:blockers field access | imported value | method call .accessSync() | method call .isFile() | method call .statSync() | try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers arrow callback of .map() | arrow callback of .some() | call of a sibling function | call of an imported function | field access | global value Boolean | imported value | method call .filter() | method call .map() | method call .resolve() | method call .some() | method call .split() | new AgentRunError | nullish coalescing

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow function | field access | imported value | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Buffer.concat | JSDoc type {…} | arrow callback of .on() | arrow function | async function | call of a sibling function | call of an imported function | field access | global call setTimeout() | method call .destroy() | method call .kill() | method call .on() | method call .push() | new AgentRunError | new Promise | null | nullish coalescing | object spread | object without a $ tag | try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers async function | call of a sibling function | field access | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Date.now | JSDoc type {…} | arrow callback of .find() | arrow callback of .map() | assignment of a field or element | async function | call of a sibling function | call of an imported function | field access | let without a value | method call .find() | method call .has() | method call .join() | method call .map() | method call .push() | method call .split() | new AgentRunError | null | nullish coalescing | object spread | object without a $ tag | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Date.now | JSDoc type {…} | arrow callback of .every() | async function | call of a sibling function | call of an imported function | field access | method call .every() | method call .push() | new AgentRunError | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | async function | call of a sibling function | call of an imported function | field access | method call .join() | method call .split() | new AgentRunError | null | object spread | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal rest property
// formal-ai:blockers destructuring | object spread | object without a $ tag | undefined
