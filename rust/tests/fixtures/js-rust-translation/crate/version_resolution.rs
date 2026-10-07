// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=9f5b77c2b417fc1ea078a5ad88dfbe4571bf51076ee8bcd87fe3d1f1c02addaf bytes=12054
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=96d5c1c208b6092a1de30c865ab273d1e9f3ba2b8e3a916f97de2c437207482f
// | const TOOLCHAINS = 'data/seed/toolchains.lino';
pub const TOOLCHAINS: &str = "data/seed/toolchains.lino";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d13f659d5ba21cf75f8e21f12baf7494d8e95cc9ee421318dd4e207a1f148218
// | const PIN_IDS = [
// |   ['checkout', 'actions_checkout'],
// |   ['setup_java', 'actions_setup_java'],
// |   ['setup_kotlin', 'setup_kotlin_action'],
// |   ['setup_python', 'actions_setup_python'],
// |   ['python_interpreter', 'python_interpreter'],
// |   ['kotlin', 'kotlin_compiler'],
// |   ['java_lts', 'java_lts'],
// |   ['setup_coursier', 'setup_coursier_action'],
// |   ['scala', 'scala_compiler'],
// | ];
pub static PIN_IDS: std::sync::LazyLock<Vec<Vec<String>>> = std::sync::LazyLock::new(|| vec![vec![String::from("checkout"), String::from("actions_checkout")], vec![String::from("setup_java"), String::from("actions_setup_java")], vec![String::from("setup_kotlin"), String::from("setup_kotlin_action")], vec![String::from("setup_python"), String::from("actions_setup_python")], vec![String::from("python_interpreter"), String::from("python_interpreter")], vec![String::from("kotlin"), String::from("kotlin_compiler")], vec![String::from("java_lts"), String::from("java_lts")], vec![String::from("setup_coursier"), String::from("setup_coursier_action")], vec![String::from("scala"), String::from("scala_compiler")]]);

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object spread

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal call of a computed function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal call of a computed function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: @param needs a type and a name

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .reduce()

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: malformed number

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
