// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=2a1d655bf15b17abc8623448164ff182692e46353e7428661de262a8719ea072 bytes=14333
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

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

// meta-language:translated JavaScript lexical_declaration items=1 sha256=104342e6f12770ba3cef701729238ebda54225195799a1298582a0a63c39a13a
// | const FACT_CAPTURES_FILE = "data/seed/fact-captures.lino";
pub const FACT_CAPTURES_FILE: &str = "data/seed/fact-captures.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=eaea54c6314b6c7e5dbc5424e1fdfb6d17f98697e8e7866d67f5ec29e6f53a7c
// | const FACT_REALIZATION_FILE = "data/seed/fact-realization.lino";
pub const FACT_REALIZATION_FILE: &str = "data/seed/fact-realization.lino";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | call of an imported function | field access | method call .find() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | field access | method call .filter()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access | global call String()

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function | global call String()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | call of a sibling function | field access | method call .map() | method call .push() | method call .set() | method call .slice() | new Map | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | arrow callback of .sort() | arrow function | call of a sibling function | destructuring | field access | global call String() | method call .filter() | method call .indexOf() | method call .map() | method call .set() | method call .slice() | method call .sort() | new Map | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access | method call .get()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access | method call .get() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .every() | arrow callback of .filter() | method call .every() | method call .filter()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | assignment of a field or element | call of a sibling function | field access | method call .find() | method call .get() | method call .join() | method call .slice() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access | method call .matchAll() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access | method call .get() | method call .join() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow function | call of a sibling function | field access | global call String() | method call .get() | method call .keys() | method call .push() | method call .slice() | new Map | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow function | call of a sibling function | call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .get() | method call .has() | method call .keys() | method call .map() | method call .push() | method call .set() | new Map | null | nullish coalescing | object without a $ tag | optional chaining
