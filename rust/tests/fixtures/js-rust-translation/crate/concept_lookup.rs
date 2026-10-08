// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=0b341e282ecb0fba04c3487889cb0284a20738b049c1acb69af7e37d2a2f25e6 bytes=4419
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=4

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f74f728d8e1e2ea542f31cf8ce3bd16ac821397530aae7504396f29a5151a9ca
// | const PAIRS = [['"', '"'], ['«', '»'], ['“', '”'], ['‘', '’'], ['「', '」']];
pub static PAIRS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(|| vec![vec![String::from("\""), String::from("\"")], vec![String::from("«"), String::from("»")], vec![String::from("“"), String::from("”")], vec![String::from("‘"), String::from("’")], vec![String::from("「"), String::from("」")]]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .map() | arrow function | call of an imported function | method call .flatMap() | method call .map() | new Set

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers PAIRS.find | arrow callback of .find() | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | call of a sibling function | call of an imported function | imported value | method call .every() | method call .has()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal void operator
// formal-ai:blockers call of a sibling function | call of an imported function | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | PAIRS.find | arrow callback of .find() | arrow function | call of a sibling function | call of an imported function | destructuring | method call .push() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method in an object value
// formal-ai:blockers Object.freeze | object without a $ tag
