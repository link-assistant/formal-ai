// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=317a24c0ca9f5f0266dce6045f7c909d7e0e8c169b1cd8703b0d9604153c9494 bytes=21028
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=4

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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript export_statement items=1 sha256=a33eb4ecfbce26356d2247678cfe9f8597d97098ee65bd070c2ed122e38346d1
// | /** Mirrors `PROPOSITION_RULE` in rust/src/formalization/proposition_structure.rs. */
// | export const PROPOSITION_RULE = 'proposition-structure';
pub const PROPOSITION_RULE: &str = "proposition-structure";

// meta-language:translated JavaScript export_statement items=1 sha256=46eb3c0fe3dbb8c89e1d5f6d4f2e9b58d4286df1ed0f10be93c34b755b0b0dfa
// | /** Mirrors `COMPARISON_RULE` in rust/src/formalization/proposition_structure.rs. */
// | export const COMPARISON_RULE = 'comparison-relation';
pub const COMPARISON_RULE: &str = "comparison-relation";

// meta-language:translated JavaScript export_statement items=1 sha256=49088e901f07da641d41851ce828857b5a27c96cd1f34fdac37087533aba1fb5
// | /** Mirrors `OPERATOR_RULE` in rust/src/formalization/proposition_structure.rs. */
// | export const OPERATOR_RULE = 'symbolic-operator';
pub const OPERATOR_RULE: &str = "symbolic-operator";

// meta-language:translated JavaScript export_statement items=1 sha256=a1fa7b7b7008ba758d403a18692e70a1204ae8a8d4425e26e81c148de32e5e14
// | /** Mirrors `ORDERED_RELATIONS` in rust/src/formalization/proposition_structure.rs: relations with a converse wording. */
// | export const ORDERED_RELATIONS = Object.freeze(['less', 'less-or-equal']);
pub static ORDERED_RELATIONS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("less"), String::from("less-or-equal")]);

// meta-language:translated JavaScript export_statement items=1 sha256=3e4757fe81e0757c04ab1820fc84a957276f05a49e20bdde4b114c6ff09246b7
// | /** Mirrors `PLAIN_OPERATORS` in rust/src/formalization/proposition_structure.rs. */
// | export const PLAIN_OPERATORS = Object.freeze(['equal', 'not-equal', 'minus', 'plus', 'times', 'divide']);
pub static PLAIN_OPERATORS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("equal"), String::from("not-equal"), String::from("minus"), String::from("plus"), String::from("times"), String::from("divide")]);

// meta-language:translated JavaScript export_statement items=1 sha256=4b412aaa1a22651863cd087d047fb25db0513290173c226bd9cd29f7f5b8e3f7
// | /** Mirrors `CONVERSE_SUFFIX` in rust/src/formalization/proposition_structure.rs. */
// | export const CONVERSE_SUFFIX = '-converse';
pub const CONVERSE_SUFFIX: &str = "-converse";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers Object.freeze | sibling value

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f723135cfa18b17e66b7f0979721686a47054673f6746e43048c8b43fa0d6984
// | const REPETITION = 'repetition';
pub const REPETITION: &str = "repetition";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .map() | arrow callback of .sort() | arrow function | call of an imported function | destructuring | field access | method call .map() | method call .push() | method call .sort() | object spread | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of an imported function | null | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers ORDERED_RELATIONS.includes | call of an imported function | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .map() | arrow callback of .sort() | arrow function | call of an imported function | field access | method call .map() | method call .push() | method call .sort() | object spread | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal array destructuring
// formal-ai:blockers assignment of a field or element | call of an imported function | destructuring | object without a $ tag

// meta-language:translated JavaScript lexical_declaration items=1 sha256=9a1c312a9ec28b94f04f6ee90314138562870e1811ace72708042f3ddf8d3ba3
// | const lower = (text) => text.toLowerCase();
pub fn lower(text: String) -> String {
    text.to_lowercase()
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Array.from | field access | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .every() | arrow callback of .find() | call of a sibling function | call of an imported function | field access | method call .every() | method call .find() | method call .join() | method call .push() | method call .slice() | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers Array.from | arrow function | call of an imported function | field access | imported value | method call .every()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .forEach() | call of a sibling function | field access | method call .forEach() | method call .push() | null | object without a $ tag | optional chaining | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .forEach() | arrow callback of .map() | arrow callback of .some() | arrow function | call of a sibling function | destructuring | field access | imported value | method call .flatMap() | method call .forEach() | method call .isTerm() | method call .join() | method call .map() | method call .pop() | method call .push() | method call .shift() | method call .some() | method call .sort() | new Set | null | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers field access | method call .join() | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers arrow callback of .every() | field access | method call .every() | method call .split() | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | call of an imported function | field access | method call .filter() | method call .map() | nullish coalescing | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers Array.from | arrow callback of .some() | arrow callback of .sort() | arrow function | call of a sibling function | call of an imported function | destructuring | field access | method call .push() | method call .some() | method call .sort() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers field access | method call .push() | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers Array.from | arrow callback of .find() | arrow callback of .sort() | arrow function | call of a sibling function | call of an imported function | field access | imported value | method call .filter() | method call .find() | method call .join() | method call .push() | method call .slice() | method call .sort() | object without a $ tag | sibling value | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .push() | null | object without a $ tag | sibling value

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow function | method call .push() | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers arrow callback of .find() | arrow callback of .forEach() | arrow callback of .map() | arrow callback of .some() | arrow function | call of a sibling function | destructuring | field access | imported value | method call .find() | method call .forEach() | method call .join() | method call .map() | method call .pop() | method call .push() | method call .reverse() | method call .shift() | method call .slice() | method call .some() | method call .sort() | new Set | nullish coalescing | object without a $ tag | sibling value | undefined
