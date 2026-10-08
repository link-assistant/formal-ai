// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1390ee78a666b6bc4c9fbaa8a36b66a86aa7d69e474eb7e59d043a3441bc583f bytes=12666
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

// meta-language:translated JavaScript export_statement items=1 sha256=af4c04522de8630bf4ede8978b5718c494f97c55d4280728391ad205c7df9bd7
// | /** Mirrors `PRELUDE_METHOD_NAMES` in rust/src/solver_dispatch.rs. */
// | export const PRELUDE_METHOD_NAMES = Object.freeze(['diagnostic', 'nl_tool', 'behavior_rules', 'feature_capability', 'playwright_script']);
pub static PRELUDE_METHOD_NAMES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("diagnostic"), String::from("nl_tool"), String::from("behavior_rules"), String::from("feature_capability"), String::from("playwright_script")]);

// meta-language:translated JavaScript export_statement items=1 sha256=b630b6627ef9625c1562cfc3df9f73795f7e45fd27146fe92b4c7991b9eee570
// | /** Mirrors `CONTEXTUAL_HANDLER_NAMES` in rust/src/solver_dispatch.rs. */
// | export const CONTEXTUAL_HANDLER_NAMES = Object.freeze([
// |   'http_fetch', 'proof_request', 'meta_explanation', 'numeric_list', 'program_synthesis',
// |   'shell_command_transform', 'text_manipulation', 'task_decomposition', 'response_language_followup',
// |   'fact_checking', 'world_state', 'web_search', 'procedural_how_to', 'fact_lookup',
// | ]);
pub static CONTEXTUAL_HANDLER_NAMES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("http_fetch"), String::from("proof_request"), String::from("meta_explanation"), String::from("numeric_list"), String::from("program_synthesis"), String::from("shell_command_transform"), String::from("text_manipulation"), String::from("task_decomposition"), String::from("response_language_followup"), String::from("fact_checking"), String::from("world_state"), String::from("web_search"), String::from("procedural_how_to"), String::from("fact_lookup")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=83968f47c4e6fd50ca2edd074533d2b4dbe5b43ba66a60998c7db9200d9cecd4
// | const HANDLER_PRECEDENCE_PATH = 'data/seed/handler-precedence.lino';
pub const HANDLER_PRECEDENCE_PATH: &str = "data/seed/handler-precedence.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a185cbdc4ccf948b5ce418d22ce8d9d2a84e3f95aee7ae1c72e23345c0fed626
// | const METHOD_EXECUTION_PATH = 'data/seed/method-execution.lino';
pub const METHOD_EXECUTION_PATH: &str = "data/seed/method-execution.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=108f2fa89be2ca4eaee3734b8fb847bc96951d816e6aaad03e725a6e6874edd1
// | const LEARNED_METHODS_PATH = 'data/seed/learned-methods.lino';
pub const LEARNED_METHODS_PATH: &str = "data/seed/learned-methods.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=1d5db4077effdfbda6a0bb34292c0fb692ab766cd768c7510710d06b4847185a
// | const RECIPE_PATH = 'data/meta/recursive-core-recipe.lino';
pub const RECIPE_PATH: &str = "data/meta/recursive-core-recipe.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=e5c5ed652a426650504bb00ad42473dd286c9c28f61c978e424a6c301d8ad6e5
// | const HEURISTICS_PATH = 'data/meta/selection-heuristics.lino';
pub const HEURISTICS_PATH: &str = "data/meta/selection-heuristics.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=3d6550a7967460d8e6aa54275b0ff5620aa844b4dbd11c3597559a70e1b0167d
// | const ALIASES_PATH = 'data/meta/route-method-aliases.lino';
pub const ALIASES_PATH: &str = "data/meta/route-method-aliases.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ff1ec1c2252b4d792caa9a828c854c85bcef3f60b3c27e19eac4107355e42001
// | const HEURISTIC_RECORD_TYPE = 'selection_heuristic';
pub const HEURISTIC_RECORD_TYPE: &str = "selection_heuristic";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
