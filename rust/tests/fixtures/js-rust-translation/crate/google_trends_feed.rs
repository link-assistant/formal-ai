// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=e14a7deea4551a8ad2db5653645223c2f8220a1424512b7378549112a3967ccf bytes=2608
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=1b70b62d6e9f140697ce62f72f1d0d36ee7a3aba9fb6befbea0f4f9798f3889d
// | export const GOOGLE_TRENDS_SOURCE = 'google_trends_rss';
pub const GOOGLE_TRENDS_SOURCE: &str = "google_trends_rss";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=c4a4a37b5c2f19fc36ed9ef27a72367a4084582ac48ac5f08443ba9cc905b018
// | const TOP_LIMIT = 10;
pub const TOP_LIMIT: f64 = 10f64;

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal namespace property …
// formal-ai:blockers object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers arrow function | global call String() | method call .replace() | nullish coalescing | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers Array.from | arrow callback of .from() | method call .matchAll() | new RegExp

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers arrow callback of .replace() | call of a sibling function | method call .replace() | method call .slice() | nullish coalescing | optional chaining | regular expression | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow callback of .find() | arrow callback of .map() | call of a sibling function | method call .exec() | method call .find() | method call .map() | method call .push() | object without a $ tag | optional chaining | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .findIndex() | global value Boolean | method call .filter() | method call .findIndex() | method call .slice() | method call .split() | method call .test() | null | nullish coalescing | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of an imported function | field access | method call .filter() | method call .map() | object without a $ tag
