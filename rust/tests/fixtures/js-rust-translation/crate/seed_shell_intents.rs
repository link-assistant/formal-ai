// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1d8700394e11ceff860dd11613677bd82f5ed3a48de503742b3fe699ebe45d32 bytes=5013
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow function | field access | method call .filter()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .map() | arrow function | call of a sibling function | field access | method call .flatMap() | method call .map()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | arrow function | method call .map()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=44f1c840798f1cf263e63276d83cf74e47f35234de148a5d8ac1c20251e389f9
// | const ARGUMENTS = ['path', 'name_lead', 'one_path', 'two_paths', 'remainder', 'search_query'];
pub static ARGUMENTS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("path"), String::from("name_lead"), String::from("one_path"), String::from("two_paths"), String::from("remainder"), String::from("search_query")]);

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers ARGUMENTS.includes | arrow callback of .find() | arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .find() | method call .map() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .find() | arrow callback of .map() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .find() | method call .map() | object without a $ tag | sibling value
