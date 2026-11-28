// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=41d49060818e09fb563e5812f9f866d8f22913a31aea40a56bc1fa51bf6f964c bytes=22454
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=6

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

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

// meta-language:translated JavaScript lexical_declaration items=1 sha256=341aa995b9ecfe4f95651208c20f2fff1f45ac3f9279baccb9b64073831cae12
// | const RULE = 'task-decomposition';
pub const RULE: &str = "task-decomposition";

// meta-language:translated JavaScript export_statement items=1 sha256=29c343fed5e5f050462d60b04c9903930a37ab4e891f6658c3aa809579a28f00
// | /** The event kinds, in the order a level emits them. */
// | export const META_EXEC_EVENT_KINDS = Object.freeze([
// |   'meta_exec:bounds',
// |   'meta_exec:task',
// |   'meta_exec:test',
// |   'meta_exec:draft',
// |   'meta_exec:compose',
// |   'meta_exec:select',
// |   'meta_exec:unresolved',
// |   'meta_exec:result',
// | ]);
pub static META_EXEC_EVENT_KINDS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("meta_exec:bounds"), String::from("meta_exec:task"), String::from("meta_exec:test"), String::from("meta_exec:draft"), String::from("meta_exec:compose"), String::from("meta_exec:select"), String::from("meta_exec:unresolved"), String::from("meta_exec:result")]);

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: function value RULE: functions and namespaces are only portable when a function is called
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal throw of a non-error value
// formal-ai:blockers call of a sibling function | throw of a non-error value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers call of a sibling function | method call .test() | regular expression | throw of a non-error value

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | arrow function | call of a sibling function | object without a $ tag | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | method call .join() | method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | field access | method call .push() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers Array.from | arrow callback of .find() | arrow callback of .map() | arrow callback of .some() | call of a sibling function | call of an imported function | global call Boolean() | method call .find() | method call .map() | method call .push() | method call .some() | null | nullish coalescing | throw of a non-error value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers call of an imported function | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | method call .join()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.from | JSDoc type {…} | arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | call of an imported function | field access | global call String() | method call .filter() | method call .find() | method call .map() | method call .push() | method call .slice() | method call .sort() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | assignment of a field or element | call of a sibling function | field access | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers field access | method call .flatMap()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers field access | method call .join() | method call .map()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | field access | method call .filter()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .some() | call of a sibling function | call of an imported function | field access | global call Boolean() | global call String() | imported value | let without a value | method call .push() | method call .some() | null | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | global call String() | method call .join() | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .lastIndexOf()
// formal-ai:blockers global call String() | method call .indexOf() | method call .lastIndexOf() | method call .push() | method call .slice() | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow callback of .map() | arrow callback of .reduce() | call of a sibling function | field access | method call .filter() | method call .flatMap() | method call .join() | method call .map() | method call .reduce() | new Set | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.isArray | field access | global call String() | method call .slice() | null | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | assignment of a field or element | async function | call of a sibling function | call of an imported function | field access | method call .cached() | method call .draft() | method call .filter() | method call .find() | method call .get() | method call .has() | method call .join() | method call .map() | method call .push() | method call .set() | null | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | async function | call of a sibling function | field access | global call String() | new Map | new Set | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow function | assignment of a field or element | async function | call of a sibling function | call of an imported function | field access | method call .attempt() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | async function | call of a sibling function | call of an imported function | field access | global call String() | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (top-level statement)
// formal-ai:refusal top-level statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | async function | sibling value | try statement

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow function | field access | null | object without a $ tag

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers Array.isArray | JSDoc type {…} | arrow callback of .map() | arrow function | async function | call of a sibling function | field access | method call .concat() | method call .map() | null | nullish coalescing | object spread | object without a $ tag | optional chaining
