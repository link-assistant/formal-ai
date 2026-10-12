// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=5ea9cd462c17d3a1f3d9bb2d008119a9a3afb00be0247a3bb0c46da5341c801e bytes=11645
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

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:translated JavaScript lexical_declaration items=1 sha256=16408cae188b9968165fdd8913a0ee83257567bafd2af291910d643cd7cf7940
// | /** Head of the record `links_notation` emits. */
// | const RECORD_TYPE = 'computer_use_learned_schemas';
pub const RECORD_TYPE: &str = "computer_use_learned_schemas";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=1aeaca7b71cc3729eed2ff4fec94f5edca19a5c54d5284c97b2b072affe91d4e
// | /** The corpus the schemas are induced from. */
// | const CORPUS_PATH = 'data/seed/computer-use-tasks.lino';
pub const CORPUS_PATH: &str = "data/seed/computer-use-tasks.lino";

// meta-language:translated JavaScript export_statement items=1 sha256=29f9e37910e5d8ad30aebd94edb0984a671489dbe268c4cf335a865ad9575405
// | /** `FETCH_OPERATION`: the operation realised by a materialisation prefix. */
// | export const FETCH_OPERATION = 'computer_use_fetch';
pub const FETCH_OPERATION: &str = "computer_use_fetch";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .keys()
// formal-ai:blockers arrow function | imported value | method call .keys() | method call .sort()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .sort()
// formal-ai:blockers arrow function | imported value | method call .sort()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers field access | null | object without a $ tag | optional chaining | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers COMPUTER_USE_PRIMITIVES.indexOf | call of an imported function | field access | imported value | null

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers Array.isArray | arrow function | global call Boolean() | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .has()
// formal-ai:blockers method call .get() | method call .has() | method call .push() | method call .set()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .every() | arrow callback of .filter() | arrow callback of .find() | arrow callback of .forEach() | arrow callback of .map() | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | imported value | method call .every() | method call .filter() | method call .find() | method call .forEach() | method call .get() | method call .has() | method call .map() | method call .push() | method call .set() | method call .slice() | method call .sort() | new Map | new Set | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | method call .push() | method call .some() | method call .sort() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow callback of .reduce() | assignment of a field or element | call of an imported function | destructuring | field access | method call .filter() | method call .has() | method call .map() | method call .reduce() | new Set

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers call of a sibling function | call of an imported function | destructuring | field access | method call .get() | method call .set() | new Map | null | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers Object.keys | arrow callback of .map() | call of a sibling function | destructuring | field access | global call structuredClone() | method call .add() | method call .delete() | method call .get() | method call .has() | method call .map() | method call .set() | method call .sort() | new Map | new Set | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .every() | arrow callback of .map() | arrow callback of .some() | arrow function | call of a sibling function | method call .every() | method call .map() | method call .some() | new Map | new Set | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=35d160f867903fc8744207ced3432c46856b93e2e06b2fdb7371b4c1d20a30f4
// | const PARAMETER_FIELDS = ['selector', 'pointer', 'column', 'equals', 'body'];
pub static PARAMETER_FIELDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("selector"), String::from("pointer"), String::from("column"), String::from("equals"), String::from("body")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers Object.prototype | call of a sibling function | field access | global call structuredClone() | method call .call() | method call .has() | method call .set() | new Map

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .get() | method call .join() | method call .push()
