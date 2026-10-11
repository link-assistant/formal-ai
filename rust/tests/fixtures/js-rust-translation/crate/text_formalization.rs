// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=8b9a74fde44da0e6fb4c3d256e2d79cd3f8509d06ba117f0db7383fa80ce918b bytes=27948
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=3

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

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
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ecd1cd9da7f43652df502b7ec97a5116cf2cc84481d8dcdf100f56b5e0d88a5b
// | const HAN = 'han';
pub const HAN: &str = "han";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow function

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | method call .test() | sibling value

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: function value HAN: functions and namespaces are only portable when a function is called
// formal-ai:blockers arrow function | call of an imported function

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers arrow function | method call .test() | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow function | call of an imported function | field access | method call .filter() | method call .find() | method call .map() | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow callback of .sort() | arrow function | call of a sibling function | call of an imported function | field access | method call .map() | method call .sort() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow callback of .map() | arrow function | call of an imported function | field access | method call .filter() | method call .flatMap() | method call .map() | new Set

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | method call .add() | new Set | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .forEach() | arrow function | call of a sibling function | method call .forEach() | method call .has() | method call .push() | object without a $ tag | sibling value | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .map() | arrow callback of .some() | call of a sibling function | field access | global value Boolean | method call .filter() | method call .join() | method call .map() | method call .some() | method call .split() | method call .test() | null | regular expression | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers FUNCTION_WORD_ROLES.some | JSDoc type {…} | arrow callback of .find() | arrow callback of .forEach() | arrow callback of .map() | arrow callback of .reduce() | arrow callback of .some() | arrow function | call of a sibling function | call of an imported function | field access | method call .entries() | method call .find() | method call .forEach() | method call .get() | method call .map() | method call .reduce() | method call .set() | new Map | new Set | null | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | method call .get() | method call .has() | method call .slice() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | method call .has() | null | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | method call .push() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .map() | field access | method call .get() | method call .has() | method call .join() | method call .map() | method call .min() | method call .slice() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | method call .get() | method call .has() | method call .test() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | field access | method call .push() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | field access | method call .push() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | method call .get() | method call .has() | method call .join() | method call .min() | method call .slice() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | field access | method call .push() | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow function | field access | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow callback of .some() | call of a sibling function | field access | method call .filter() | method call .find() | method call .map() | method call .slice() | method call .some() | null | nullish coalescing | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .pop() | method call .push() | null | object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .forEach() | call of a sibling function | call of an imported function | field access | method call .forEach() | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | call of a sibling function | field access | method call .add() | method call .filter() | method call .has() | method call .map() | method call .test() | new Set | null | object spread | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .flatMap() | call of a sibling function | field access | method call .flatMap()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers field access | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | imported value | method call .map() | method call .sort() | new Set

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | field access | imported value | method call .filter() | method call .map() | method call .sort() | new Set

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | field access | imported value | method call .filter() | method call .join() | method call .map() | method call .sort() | new Set | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | call of a sibling function | field access | method call .every()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .find() | method call .map() | null | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | method call .get() | method call .has() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .find() | call of a sibling function | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | field access | nullish coalescing | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .pop()
// formal-ai:blockers call of a sibling function | method call .codePointAt() | method call .pop() | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | method call .push() | null | sibling value
