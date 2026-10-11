// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=f4161079a1e37708915bef76e0ddab96a4104745668c051a3d0b0f7330db822b bytes=20319
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=4

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

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

// meta-language:translated JavaScript export_statement items=1 sha256=c6ad40a9c829836fb71710ae6ff89a5ccb788d48aafc8fbc57919b1154e72856
// | /** Mirrors `ROLE_WIKIDATA_ENTITY_ANCHOR` in rust/src/seed/roles/tooling.rs. */
// | export const ROLE_WIKIDATA_ENTITY_ANCHOR = 'wikidata_entity_anchor';
pub const ROLE_WIKIDATA_ENTITY_ANCHOR: &str = "wikidata_entity_anchor";

// meta-language:translated JavaScript export_statement items=1 sha256=7a8e639dd47d602de8eb9acbfc3f51a82faa51c1d9bece338e743a3338122e11
// | /** Mirrors `ROLE_BINARY_RELATION_PROPERTY` in rust/src/seed/roles/tooling.rs. */
// | export const ROLE_BINARY_RELATION_PROPERTY = 'binary_relation_property';
pub const ROLE_BINARY_RELATION_PROPERTY: &str = "binary_relation_property";

// meta-language:translated JavaScript export_statement items=1 sha256=f336fc30657d23ee3dcd351cbc606b841d37e2b4af042d8abd3ac66d3320a326
// | /** Mirrors `ROLE_TRANSLATION_PROPERTY` in rust/src/seed/roles/tooling.rs. */
// | export const ROLE_TRANSLATION_PROPERTY = 'translation_property';
pub const ROLE_TRANSLATION_PROPERTY: &str = "translation_property";

// meta-language:translated JavaScript export_statement items=1 sha256=3b730c78270a1a9cbd4adeee2654ac1e9aa1eb5614278df8225ebd117438316b
// | /** Mirrors `ROLE_TRANSLATION_ACTION` in rust/src/seed/roles/language.rs. */
// | export const ROLE_TRANSLATION_ACTION = 'translation_action';
pub const ROLE_TRANSLATION_ACTION: &str = "translation_action";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextEncoder

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new TextDecoder

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .replace()
// formal-ai:blockers arrow function | method call .replace() | sibling value

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | method call .encode() | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow function | global value Boolean | method call .filter() | method call .split() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow function | bitwise operator | method call .decode() | method call .encode() | method call .subarray() | null | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | new Error | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers call of a sibling function | method call .indexOf() | method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | call of a sibling function | method call .has() | method call .join() | method call .pop() | method call .shift() | method call .slice() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of a sibling function | global value Boolean | method call .filter() | method call .join() | method call .split() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal case test
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal case test

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function | field access | global call String()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | field access | method call .join() | method call .map() | method call .push()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of an imported function | field access | method call .map() | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of an imported function | destructuring | field access | method call .map() | method call .push() | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers Array.from | arrow callback of .map() | arrow callback of .sort() | field access | method call .map() | method call .sort() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers field access | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .every() | arrow callback of .filter() | arrow function | method call .codePointAt() | method call .every() | method call .filter() | method call .some() | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .some() | method call .some() | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers call of a sibling function | global value Boolean | method call .filter() | method call .join() | method call .split() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | call of an imported function | field access | method call .push() | method call .replaceAll() | method call .slice() | method call .some() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow callback of .some() | call of a sibling function | field access | method call .find() | method call .some() | null | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | method call .push() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow callback of .reduce() | field access | method call .filter() | method call .map() | method call .reduce() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | field access | method call .push() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .some() | call of a sibling function | call of an imported function | method call .extractUnquotedTranslationSurface() | method call .slice() | method call .some() | null | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | method call .slice() | method call .test() | null | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | assignment of a field or element | call of a sibling function | destructuring | field access | method call .filter() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .sort()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .sort() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .sort()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .some() | call of a sibling function | method call .some() | method call .test() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | call of an imported function | destructuring | field access | method call .extractConceptQuery() | method call .push() | null | object without a $ tag | typeof operator | undefined

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function
