// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=ad4bda370077b012151ddc2be5be82508f7e58fc36bb20f13925de1912553b37 bytes=27979
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

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript export_statement items=1 sha256=fc62867699c06b20229106914761a4f4ed172866c107b35a3bcd0b5c2e0d33ac
// | /** Mirrors `TEXT_FORMALIZATION_LINO` in rust/src/seed/embedded_registry.rs (the repository path it embeds). */
// | export const TEXT_FORMALIZATION_PATH = 'data/seed/text-formalization.lino';
pub const TEXT_FORMALIZATION_PATH: &str = "data/seed/text-formalization.lino";

// meta-language:translated JavaScript export_statement items=1 sha256=2eb8fa15a18deaee3bc3f51e8452da06becc8d37db0fd1eda541b204d7327304
// | /** Mirrors `FUNCTION_WORD_ROLES` in rust/src/formalization/text_statements.rs: the lexicon roles whose words carry no term. */
// | export const FUNCTION_WORD_ROLES = Object.freeze(['translation_stop_word', 'statement_function_word']);
pub static FUNCTION_WORD_ROLES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("translation_stop_word"), String::from("statement_function_word")]);

// meta-language:translated JavaScript export_statement items=1 sha256=1430fe4e91c33905042a3b7625881c2d8e81d87fb17874062700c464890fa76a
// | /** Mirrors `NEGATION_ROLE` in rust/src/formalization/text_statements.rs: the lexicon role whose words deny a statement. */
// | export const NEGATION_ROLE = 'statement_negation_cue';
pub const NEGATION_ROLE: &str = "statement_negation_cue";

// meta-language:translated JavaScript export_statement items=1 sha256=53865a1c7f28f922404629d81d819692bc7382af5456d2bd3a23dba17df0ef80
// | /** Mirrors `LONGEST_PHRASE` in rust/src/formalization/text_statements.rs: the most words one lexicon surface may span. */
// | export const LONGEST_PHRASE = 4;
pub const LONGEST_PHRASE: f64 = 4f64;

// meta-language:translated JavaScript export_statement items=1 sha256=305ce0bae8a45e999c19824397c4c057d6589f9a795f34bd962280ab6b577c94
// | /** Mirrors `LONGEST_HAN_WORD` in rust/src/formalization/text_statements.rs: the most characters one Han lexicon surface may span. */
// | export const LONGEST_HAN_WORD = 8;
pub const LONGEST_HAN_WORD: f64 = 8f64;

// meta-language:translated JavaScript export_statement items=1 sha256=882598a7cc8c6741222a394fa12dcaa9c2ed521c3df1e7a6ef01ba37b5fd71ac
// | /** Mirrors `SHORTEST_STEM` in rust/src/formalization/text_statements.rs: an inflected word keeps at least this many characters. */
// | export const SHORTEST_STEM = 3;
pub const SHORTEST_STEM: f64 = 3f64;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ecd1cd9da7f43652df502b7ec97a5116cf2cc84481d8dcdf100f56b5e0d88a5b
// | const HAN = 'han';
pub const HAN: &str = "han";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Array.from

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .flatMap()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .pop()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
