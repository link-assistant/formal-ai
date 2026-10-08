// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1d60a0ba364bdd16e641893ad789180c3e26b43ca7b1e38ea633a99cc80ed41f bytes=5986
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript export_statement items=1 sha256=19ef29f356e80ed76812a3f4fb77fa2181e1c7f60d3e159a85c2a565a7224ec6
// | /** Mirrors `ROLE_STATEMENT_FUNCTION_WORD` in rust/src/seed/roles/tooling.rs. */
// | export const ROLE_STATEMENT_FUNCTION_WORD = 'statement_function_word';
pub const ROLE_STATEMENT_FUNCTION_WORD: &str = "statement_function_word";

// meta-language:translated JavaScript export_statement items=1 sha256=5d868d2a88626de99b2a320a2a58a82d6ab63dfa69f449875450184d67b5499e
// | /** Mirrors `ROLE_STATEMENT_NEGATION_CUE` in rust/src/seed/roles/tooling.rs. */
// | export const ROLE_STATEMENT_NEGATION_CUE = 'statement_negation_cue';
pub const ROLE_STATEMENT_NEGATION_CUE: &str = "statement_negation_cue";

// meta-language:translated JavaScript export_statement items=1 sha256=e20c1375539f77cf2aceab1b7274876d45cd9f112d26f7a2d9bb5792720ae550
// | /** Mirrors `ROLE_IDENTIFIER_RESERVED_WORD` in rust/src/seed/roles/program.rs. */
// | export const ROLE_IDENTIFIER_RESERVED_WORD = 'identifier_reserved_word';
pub const ROLE_IDENTIFIER_RESERVED_WORD: &str = "identifier_reserved_word";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of an imported function | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .some() | call of an imported function | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .push() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers call of a sibling function | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .map() | arrow callback of .sort() | call of an imported function | field access | method call .filter() | method call .map() | method call .sort() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | method call .join() | method call .push() | method call .some() | method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | call of an imported function | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of an imported function | field access | method call .find() | null | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .reduce() | call of a sibling function | method call .join() | method call .reduce() | method call .split() | nullish coalescing
