// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=0d374034d1a564f6e0bf95450585ff7d2c4a3d788910f690e044492886a2660c bytes=13803
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:translated JavaScript import_statement items=1 sha256=ea67ac58c941251801bc82c22574a8e03c0fd9db18bf89015315cef5823b17a3
// | import { lower as canonicalUnaryLower } from './rust_str.mjs';
use crate::rust_str::lower as canonical_unary_lower;

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:translated JavaScript lexical_declaration items=1 sha256=37673c539c0cf97d3f8ae5e02a7492857b945c537b2a3b44c10babfe88eb7e21
// | const FACT_CAPTURES_ROOT = "fact_captures";
pub const FACT_CAPTURES_ROOT: &str = "fact_captures";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=079a5ec2ebf94506356ed087fdfdf8428188e5bc237f0dd1c8b64dca7d777e73
// | const FACT_REALIZATION_ROOT = "fact_realization";
pub const FACT_REALIZATION_ROOT: &str = "fact_realization";

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
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of a sibling function | field access | method call .filter() | method call .get() | method call .has() | method call .keys() | method call .map() | method call .push() | method call .set() | new Map | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal export …
// formal-ai:blockers export … | sibling value
