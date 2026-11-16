// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1b1f93ce91c42d92df20868c58a733b1943f4999df679c4d00f127e3493fe4ea bytes=15209
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=6

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=7eb05a769b0f4d2503f8e558048665e66b888a22877c4f1a98a65448df0e9009
// | import { PROPOSITION_RULE, surfaceStructure } from './proposition_structure.mjs';
use crate::proposition_structure::PROPOSITION_RULE;

// meta-language:translated JavaScript import_statement items=1 sha256=e9510558b3d8c6f2baeb8831106f535421d27d9965de4c4c2f6d47c7ae50c6e1
// | import { DEFAULT_SPLIT_DEPTH_BOUND } from './task_decomposition.mjs';
use crate::task_decomposition::DEFAULT_SPLIT_DEPTH_BOUND;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:translated JavaScript export_statement items=1 sha256=9f731a8ba990f0aaafd31adcbc5fd27a1ad7477b6eaab5b066924d8d1c6c3566
// | /** Mirrors `OBLIGATION_GAP_KIND`. */
// | export const OBLIGATION_GAP_KIND = 'obligation_gap';
pub const OBLIGATION_GAP_KIND: &str = "obligation_gap";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=0e05780dcb58f06b253f38c62df6d40c08aaab9044bad36dba28f8eee76f5af9
// | /** Mirrors `WHEN_UNBOUND_OUTPUT`. */
// | const WHEN_UNBOUND_OUTPUT = 'unbound_output_literal';
pub const WHEN_UNBOUND_OUTPUT: &str = "unbound_output_literal";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .find() | call of an imported function | method call .find() | null | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | call of an imported function | field access | null | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow function | call of a sibling function | field access | method call .filter() | method call .find() | method call .push() | null | nullish coalescing | object without a $ tag | optional chaining | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | arrow function | field access | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | field access | method call .filter()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers Object.values | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | field access | method call .filter() | method call .some() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | field access | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | null | nullish coalescing | optional chaining | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | method call .filter() | method call .map() | null | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | field access | method call .filter()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | call of a sibling function | field access | method call .filter() | method call .map() | null | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | field access | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .find() | method call .map() | nullish coalescing | object without a $ tag | optional chaining | sibling value | undefined

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .flatMap() | arrow callback of .forEach() | arrow callback of .map() | arrow callback of .reduce() | arrow callback of .sort() | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | global call Array() | method call .fill() | method call .filter() | method call .find() | method call .flatMap() | method call .forEach() | method call .map() | method call .pop() | method call .push() | method call .reduce() | method call .slice() | method call .sort() | method call .split() | null | object without a $ tag | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of an imported function | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers Number.parseInt | arrow callback of .find() | arrow callback of .map() | arrow callback of .reduce() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .find() | method call .join() | method call .map() | method call .push() | method call .reduce() | method call .slice() | method call .split() | nullish coalescing | regular expression | undefined

// meta-language:translated JavaScript lexical_declaration items=1 sha256=e89ed6198c954102012e7e221fc4656a3ab67db185470046a97e0bd6ab330353
// | /** The structure part `surfaceStructure` lists explicit counts under. */
// | const REPETITION_PART = 'repetition=';
pub const REPETITION_PART: &str = "repetition=";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=70075855e8525635be7ba79ce355619460f80036e3d5d841f1850c9d56f9c90d
// | /** The suffix of the `proposition-structure` key holding a repetition kind's count. */
// | const COUNT_SUFFIX = '-count';
pub const COUNT_SUFFIX: &str = "-count";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .some() | arrow function | call of an imported function | field access | method call .some() | null | nullish coalescing | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of a sibling function | call of an imported function | method call .filter() | method call .join() | method call .push() | method call .split() | regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers method call .replace() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .findIndex() | arrow callback of .some() | call of a sibling function | call of an imported function | global value Boolean | method call .filter() | method call .findIndex() | method call .join() | method call .push() | method call .replace() | method call .slice() | method call .some() | method call .split() | method call .test() | null | regular expression | sibling value

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function
