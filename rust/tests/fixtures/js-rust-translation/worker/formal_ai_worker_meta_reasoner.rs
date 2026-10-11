// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=24fc6dac3590112b0e8354846ab1817717e3bb5b3ebb9563ef3db559470100d3 bytes=58269
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:translated JavaScript lexical_declaration items=1 sha256=515488973ba47ad0a2df22fb5d865e4c76f068bbb055c5dd75aad252c53cf63b
// | const META_REASONING_FILE = "meta-reasoning.lino";
pub const META_REASONING_FILE: &str = "meta-reasoning.lino";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers null

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Map

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Map

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Map

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow function | assignment of a field or element | field access | global call String() | global value Boolean | method call .concat() | method call .filter() | method call .find() | method call .map() | method call .push() | method call .split() | new Set | object without a $ tag | regular expression | sibling value | typeof operator | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | Object.keys | call of a sibling function | field access | global call String() | method call .join() | method call .split() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | Object.keys | call of a sibling function | field access | global call String() | method call .join() | method call .split() | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | global call String() | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .match()
// formal-ai:blockers global call String() | method call .match() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of a sibling function | field access | method call .push() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Object.keys
// formal-ai:blockers Object.keys | call of a sibling function | field access | method call .has() | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.keys | call of a sibling function | field access | method call .push() | object without a $ tag

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unsupported string escape \p
// formal-ai:blockers JSDoc type {…} | JSON.parse | arrow callback of .every() | call of a sibling function | field access | global call String() | global value Boolean | let without a value | method call .every() | method call .filter() | method call .matchAll() | method call .push() | method call .replace() | method call .split() | nullish coalescing | object without a $ tag | regular expression | try statement | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | call of a sibling function | field access | method call .find() | method call .push() | method call .replace() | method call .slice() | method call .test() | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | call of a sibling function | global call String() | method call .filter() | method call .indexOf() | method call .push() | method call .slice() | method call .split() | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | assignment of a field or element | call of a sibling function | field access | method call .add() | method call .concat() | method call .get() | method call .map() | method call .replace() | method call .set() | new Map | new Set | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.from | JSDoc type {…} | arrow callback of .from() | arrow callback of .some() | arrow callback of .sort() | call of a sibling function | field access | method call .get() | method call .has() | method call .localeCompare() | method call .set() | method call .some() | method call .sort() | method call .split() | new Map | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.from | JSDoc type {…} | JSON.stringify | arrow callback of .filter() | arrow callback of .find() | arrow callback of .from() | arrow callback of .map() | arrow callback of .some() | arrow callback of .sort() | call of a sibling function | destructuring | field access | method call .concat() | method call .emit() | method call .filter() | method call .find() | method call .get() | method call .has() | method call .join() | method call .localeCompare() | method call .map() | method call .push() | method call .repeat() | method call .set() | method call .slice() | method call .some() | method call .sort() | method call .toFixed() | method call .values() | new Map | new Set | null | object spread | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unsupported string escape \]
// formal-ai:blockers Array.from | JSDoc type {…} | field access | global call String() | method call .replace() | method call .slice() | method call .split() | method call .test() | null | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | call of a sibling function | destructuring | field access | method call .concat() | method call .emit() | method call .find() | method call .push() | method call .slice() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | global call Boolean() | method call .find()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.from | JSDoc type {…} | arrow callback of .filter() | arrow callback of .from() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | field access | method call .concat() | method call .emit() | method call .filter() | method call .get() | method call .has() | method call .join() | method call .localeCompare() | method call .map() | method call .set() | method call .slice() | method call .sort() | new Map | new Set | null | object spread | object without a $ tag

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers Array.from | Array.isArray | JSDoc type {…} | arrow callback of .sort() | call of a sibling function | field access | method call .get() | method call .localeCompare() | method call .push() | method call .set() | method call .sort() | method call .values() | new Map

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | method call .join() | method call .sort()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .find() | arrow callback of .findIndex() | arrow callback of .flatMap() | arrow callback of .map() | arrow callback of .some() | arrow function | call of a sibling function | field access | global call String() | method call .emit() | method call .every() | method call .filter() | method call .find() | method call .findIndex() | method call .flatMap() | method call .get() | method call .join() | method call .map() | method call .push() | method call .slice() | method call .some() | method call .split() | new Map | null | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.from | JSDoc type {…} | arrow callback of .every() | arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow callback of .some() | arrow callback of .sort() | assignment of a field or element | call of a sibling function | field access | global call Boolean() | global call String() | global value Boolean | method call .concat() | method call .emit() | method call .every() | method call .filter() | method call .find() | method call .get() | method call .indexOf() | method call .join() | method call .map() | method call .parse() | method call .push() | method call .reverse() | method call .set() | method call .slice() | method call .some() | method call .sort() | method call .split() | method call .stringify() | method call .unshift() | new Function | new Map | new Set | null | object without a $ tag | sibling value | try statement | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .every() | arrow callback of .find() | arrow callback of .map() | arrow callback of .some() | call of a sibling function | field access | global value Boolean | method call .every() | method call .filter() | method call .find() | method call .map() | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .some() | arrow callback of .sort() | call of a sibling function | field access | global call String() | method call .exec() | method call .indexOf() | method call .push() | method call .slice() | method call .some() | method call .sort() | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers field access | global call String() | method call .exec() | new RegExp

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .find() | field access | method call .emit() | method call .find() | method call .flatMap() | method call .has() | method call .set() | new Set | object without a $ tag | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers Array.from | JSON.stringify | arrow callback of .sort() | destructuring | field access | method call .join() | method call .localeCompare() | method call .push() | method call .sort()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | JSON.stringify | destructuring | field access | method call .entries() | method call .join() | method call .push() | method call .slice() | method call .split()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | assignment of a field or element | async function | call of a sibling function | field access | global value Map | instanceof operator | method call .join() | method call .lookup() | method call .push() | method call .set() | method call .slice() | new Map | object spread | object without a $ tag | try statement | typeof operator

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: @param needs a type and a name
// formal-ai:blockers JSDoc type {…} | JSON.stringify | arrow callback of .filter() | arrow callback of .map() | call of a sibling function | field access | method call .filter() | method call .join() | method call .map() | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | arrow function | async function | call of a sibling function | field access | method call .map() | null | object without a $ tag | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | field access | global call String() | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | String.fromCharCode | arrow callback of .filter() | arrow callback of .some() | async function | call of a sibling function | field access | method call .concat() | method call .filter() | method call .pop() | method call .push() | method call .slice() | method call .some() | method call .split() | null | object spread | object without a $ tag | typeof operator
