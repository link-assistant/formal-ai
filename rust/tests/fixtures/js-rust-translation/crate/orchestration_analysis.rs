// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=663b75bf70ed45ca7eddf54200f61f2f4b69c2892470c5ad3e3602ce07eec855 bytes=12145
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=10

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=6f1da19fb3dfedf1ac3f45c91dd832d73ab278a5c0569bfa437a962af3a6483f
// | import {
// |   checkedText, recheck, survivors, verdictIsPresentable, verdictSlug,
// | } from './summarization_recheck.mjs';
use crate::summarization_recheck::verdict_slug;

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=107157351c3c6da08f1853f86ad8648a9573d5181f1553f4b806de8bb63ea7e8
// | const SOURCES_PLACEHOLDER = '{sources}';
pub const SOURCES_PLACEHOLDER: &str = "{sources}";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=71baa6131130f05995da3559631d299231f39d530923124d1e2f83f816acd0e3
// | const PROBABILITY_PLACEHOLDER = '{probability}';
pub const PROBABILITY_PLACEHOLDER: &str = "{probability}";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers JSDoc type {…} | JSON.stringify | assignment of a field or element | class | export … | field access | method call .debugOf() | method call .display() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of an imported function | in operator | method call .slice() | method call .split() | undefined

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal in operator
// formal-ai:blockers arrow function | call of an imported function | in operator | undefined

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers arrow function | typeof operator | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers call of a sibling function | null | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers Array.isArray | arrow callback of .filter() | arrow callback of .map() | call of a sibling function | method call .filter() | method call .join() | method call .map() | null | object without a $ tag | typeof operator | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers arrow callback of .find() | arrow callback of .map() | call of a sibling function | method call .find() | method call .map() | null | nullish coalescing | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of a sibling function | field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .slice() | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .split() | null | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of an imported function | field access | method call .filter() | method call .join() | method call .lastIndexOf() | method call .push() | method call .slice() | method call .split() | method call .test() | method call .toFixed() | nullish coalescing | object without a $ tag | regular expression | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | async function | call of a sibling function | call of an imported function | destructuring | field access | imported value | method call .entries() | method call .map() | method call .push() | new AgentSynthesisError | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal throw new …
// formal-ai:blockers assignment of a field or element | call of an imported function | field access | new AgentSynthesisError | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal assignment
// formal-ai:blockers assignment of a field or element | call of an imported function | field access
