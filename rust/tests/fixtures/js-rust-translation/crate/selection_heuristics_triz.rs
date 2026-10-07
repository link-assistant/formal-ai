// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=6ba4ef4f3011a51250d4e06b8cfe02c7422ec16ede2612f465200cec178693f6 bytes=11683
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

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d73819e62b1c043c7eba6681eac268202f9d883e4b5704bc7b208eb8d522bcff
// | /** Mirrors `DIMENSIONS` in rust/src/selection_heuristics/triz.rs. */
// | const DIMENSIONS = ['code_size', 'steps', 'resource_units', 'leaf_count'];
pub static DIMENSIONS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("code_size"), String::from("steps"), String::from("resource_units"), String::from("leaf_count")]);

// meta-language:translated JavaScript function_declaration items=1 sha256=47fe22bf9a10cac6d074992c0e5b00a677f480a4e5296c77827e17c95854b0ad
// | /** Mirrors `fn criterion_for` in rust/src/selection_heuristics/triz.rs. */
// | function criterionFor(dimension) {
// |   if (dimension === 'code_size') return 'answer_brevity';
// |   if (dimension === 'steps') return 'answer_completeness';
// |   if (dimension === 'resource_units') return 'resource_economy';
// |   return 'decomposition_depth';
// | }
pub fn criterion_for(dimension: String) -> String {
    if (dimension == "code_size") {
        String::from("answer_brevity")
    } else {
        if (dimension == "steps") {
            String::from("answer_completeness")
        } else {
            if (dimension == "resource_units") {
                String::from("resource_economy")
            } else {
                String::from("decomposition_depth")
            }
        }
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .codePointAt()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:translated JavaScript lexical_declaration items=1 sha256=065ebb8a1bcd1a16017107e6367583b7afa4b7487be6c561197bdc75431fdbe5
// | /** Mirrors `NO_STATED_TRADE_OFF` in rust/src/selection_heuristics/triz.rs. */
// | const NO_STATED_TRADE_OFF = 'no_clause_demands_either_criterion';
pub const NO_STATED_TRADE_OFF: &str = "no_clause_demands_either_criterion";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:translated JavaScript lexical_declaration items=1 sha256=e5c5ed652a426650504bb00ad42473dd286c9c28f61c978e424a6c301d8ad6e5
// | const HEURISTICS_PATH = 'data/meta/selection-heuristics.lino';
pub const HEURISTICS_PATH: &str = "data/meta/selection-heuristics.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ff1ec1c2252b4d792caa9a828c854c85bcef3f60b3c27e19eac4107355e42001
// | const HEURISTIC_RECORD_TYPE = 'selection_heuristic';
pub const HEURISTIC_RECORD_TYPE: &str = "selection_heuristic";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
