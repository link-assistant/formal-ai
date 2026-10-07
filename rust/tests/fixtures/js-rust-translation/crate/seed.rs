// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=a05e71dbd638d81b9f41aeae353ba2a205e45d09298ab60946105e9bdfb4797d bytes=5268
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=fa7b3ace1090ede01933fc6f5d06aa7d19a5703fbfd7fc6c492c89f376044c74
// | /** `RESPONSE_FILES`, in the order rust/src/seed/embedded_registry.rs lists them. */
// | const RESPONSE_SUFFIXES = [
// |   '', '-agentic', '-agentic-continuation', '-agentic-tools', '-client-config', '-code-tasks',
// |   '-concept-lookup', '-creative-tasks', '-decomposition', '-engine-reports', '-entities',
// |   '-external-benchmark', '-formalization', '-language-protocol', '-legality', '-memory-program',
// |   '-orchestration', '-parity', '-pattern', '-policy', '-procedure', '-product-search', '-quantities',
// |   '-repair', '-substitution-compiler', '-summarization', '-summarization-quality', '-symbolic',
// |   '-synthesis', '-text-transform', '-thinking', '-thinking-narrative', '-translate', '-triz',
// | ];
pub static RESPONSE_SUFFIXES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from(""), String::from("-agentic"), String::from("-agentic-continuation"), String::from("-agentic-tools"), String::from("-client-config"), String::from("-code-tasks"), String::from("-concept-lookup"), String::from("-creative-tasks"), String::from("-decomposition"), String::from("-engine-reports"), String::from("-entities"), String::from("-external-benchmark"), String::from("-formalization"), String::from("-language-protocol"), String::from("-legality"), String::from("-memory-program"), String::from("-orchestration"), String::from("-parity"), String::from("-pattern"), String::from("-policy"), String::from("-procedure"), String::from("-product-search"), String::from("-quantities"), String::from("-repair"), String::from("-substitution-compiler"), String::from("-summarization"), String::from("-summarization-quality"), String::from("-symbolic"), String::from("-synthesis"), String::from("-text-transform"), String::from("-thinking"), String::from("-thinking-narrative"), String::from("-translate"), String::from("-triz")]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
