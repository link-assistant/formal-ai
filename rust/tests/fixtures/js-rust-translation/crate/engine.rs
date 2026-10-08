// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=719e9c0d262fe554fb30c37a52e7bb64eaaf7597a6b7f341d7bd81194ce1fb07 bytes=2738
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .codePointAt()
// formal-ai:blockers method call .codePointAt()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .from() | call of a sibling function | global call String() | global value Boolean | method call .filter() | method call .join() | method call .replaceAll() | method call .split() | method call .test() | regular expression | sibling value

// meta-language:translated JavaScript lexical_declaration items=1 sha256=cb9fdcb64553fc365383881e4f579ec72c03aaf69efbc625b7165184b32a7a77
// | const LANGUAGES_FILE = 'data/seed/languages.lino';
pub const LANGUAGES_FILE: &str = "data/seed/languages.lino";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of an imported function | global value Boolean | method call .filter() | method call .indexOf() | method call .push() | method call .replace() | method call .slice() | method call .split() | null | object without a $ tag | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .every() | arrow callback of .find() | call of a sibling function | method call .every() | method call .find() | method call .join() | method call .push() | method call .split()
