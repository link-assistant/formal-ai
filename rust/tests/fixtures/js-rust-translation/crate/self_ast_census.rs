// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=49b6c4b81ea62926605458d15b0d4afc6d59b4f30058e2f33f2cecf91697dc60 bytes=4949
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:translated JavaScript export_statement items=1 sha256=af811582039fec5a465c406c135d6414fda571ad0d4a39d8a31f4f668cf052d8
// | /** Mirrors `const CENSUS_DIR`. */
// | export const CENSUS_DIR = 'data/meta/self-ast';
pub const CENSUS_DIR: &str = "data/meta/self-ast";

// meta-language:translated JavaScript export_statement items=1 sha256=18fd0f789c47df983e8e9588b65c7183ceb78acaeb1ebb8f1604fc4431ed6058
// | /** Mirrors `const FULL_FIDELITY_PREFIX`. */
// | export const FULL_FIDELITY_PREFIX = 'src/agentic_coding/';
pub const FULL_FIDELITY_PREFIX: &str = "src/agentic_coding/";

// meta-language:translated JavaScript export_statement items=1 sha256=3aa57cc31aad11e30e30be83974d994672aae8bb034e26bad43db0e4f29686ed
// | /** Mirrors `fn fidelity_for`: 'full_ast' | 'signature'. @param {string} modulePath @returns {string} */
// | export function fidelityFor(modulePath) {
// |   return modulePath.startsWith(FULL_FIDELITY_PREFIX) ? 'full_ast' : 'signature';
// | }
pub fn fidelity_for(module_path: String) -> String {
    if module_path.starts_with((String::from(FULL_FIDELITY_PREFIX)).as_str()) {
        String::from("full_ast")
    } else {
        String::from("signature")
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers assignment of a field or element | call of an imported function | destructuring | method call .push() | method call .slice() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .sort()
// formal-ai:blockers arrow callback of .sort() | field access | method call .sort() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | global value Boolean | method call .censusDocuments() | method call .filter() | method call .map() | object without a $ tag | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | field access | method call .filter() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | call of a sibling function | field access | method call .filter() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | field access | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | method call .lastIndexOf() | method call .slice() | null
