// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=be99f5259dc9fc6c1b1a4ee042e859ca6a566e821be825b830a1cd9fd428fec7 bytes=10578
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:translated JavaScript export_statement items=1 sha256=a352c99d140ed49b027e7695923a4339383b233523a9bdac690072abf69cd7a9
// | export const UNKNOWN = 'unknown';
pub const UNKNOWN: &str = "unknown";

// meta-language:translated JavaScript export_statement items=1 sha256=8f6f2ef506bd33e287fdbdbd8de2738f0b4f5467a7c3c450c74f57c50dc9eda4
// | export const ENGLISH = 'en';
pub const ENGLISH: &str = "en";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers arrow function | method call .test() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | method call .slice() | null

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number
// formal-ai:blockers global call parseInt() | method call .slice() | method call .test() | null | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers method call .indexOf() | method call .push() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .filter() | method call .push() | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers call of a sibling function | field access | method call .codePointAt()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | null | nullish coalescing | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .map()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .find() | arrow callback of .some() | call of a sibling function | field access | method call .filter() | method call .find() | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | nullish coalescing | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | nullish coalescing | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .find() | assignment of a field or element | call of a sibling function | field access | method call .find() | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow function | field access | method call .find() | nullish coalescing | optional chaining

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .reduce() | arrow function | field access | method call .filter() | method call .reduce()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | arrow function | field access | method call .reduce()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .slice() | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .some() | call of a sibling function | field access | method call .indexOf() | method call .pop() | method call .slice() | method call .some() | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | call of a sibling function | field access | method call .filter() | method call .some() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .filter() | arrow callback of .reduce() | call of a sibling function | field access | method call .filter() | method call .reduce() | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | global call String()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers Array.from | arrow callback of .every() | arrow callback of .find() | arrow callback of .replace() | call of a sibling function | field access | method call .encode() | method call .every() | method call .find() | method call .replace() | method call .slice() | new TextEncoder | null | object without a $ tag | regular expression | undefined
