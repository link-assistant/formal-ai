// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1765f2987d67001b6a415c26d09cedc3aba579e0d2806a089af71339fb63bc19 bytes=10116
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=5

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript lexical_declaration items=1 sha256=344b43fb7b585894fce9c383a58d1c8a44b96a2b1047eff671b7adf460232bf4
// | const ROLE_INTERROGATIVE_OPENER = 'interrogative_opener';
pub const ROLE_INTERROGATIVE_OPENER: &str = "interrogative_opener";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a78be79804c881d1448d9c35e70594697a82abd084d2f0b089143b94bb6816e3
// | const TRANSLATION_PREDICATE = 'wikidata:P5972';
pub const TRANSLATION_PREDICATE: &str = "wikidata:P5972";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f00d29ff5b66242f5a6b5502ad494bf9dfa08ca7289c4604fe486565a2976f60
// | const REQUIREMENT_TOKENS = ['must', 'should', 'require', 'requires'];
pub static REQUIREMENT_TOKENS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("must"), String::from("should"), String::from("require"), String::from("requires")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=9bfb0c05f5bc2184c80fa17acc6b4492373e615828754d291162527f1f697a50
// | const TASK_TOKENS = ['translate', 'write', 'calculate', 'search', 'find', 'prove', 'define'];
pub static TASK_TOKENS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("translate"), String::from("write"), String::from("calculate"), String::from("search"), String::from("find"), String::from("prove"), String::from("define")]);

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | assignment of a field or element | call of an imported function | field access | method call .map() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers method call .push()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .split() | method call .test() | regular expression

// meta-language:translated JavaScript function_declaration items=1 sha256=f76ca3c775b9cf828242e4db3893e28365753890ec3405c34c0a2c2e67e6413b
// | /**
// |  * Mirrors `fn slot_known_link`.
// |  * @param {string} role
// |  * @param {string} kind
// |  * @param {string} id
// |  * @returns {string}
// |  */
// | function slotKnownLink(role, kind, id) {
// |   if (kind === 'wikidata_item') {
// |     if (role === 'subject') return `formalization:subject_q:${id}`;
// |     if (role === 'object') return `formalization:object_q:${id}`;
// |     return `formalization:item_q:${id}`;
// |   }
// |   if (kind === 'wikidata_property') return role === 'predicate' ? `formalization:predicate_p:${id}` : `formalization:property_p:${id}`;
// |   if (kind === 'wikipedia_article' || kind === 'wiktionary_entry') return `formalization:fallback:${id}`;
// |   return `formalization:raw:${id}`;
// | }
pub fn slot_known_link(role: String, kind: String, id: String) -> String {
    if (kind == "wikidata_item") {
        if (role == "subject") {
            format!("{}{}", (String::from("formalization:subject_q:")), id)
        } else {
            if (role == "object") {
                format!("{}{}", (String::from("formalization:object_q:")), id)
            } else {
                format!("{}{}", (String::from("formalization:item_q:")), id)
            }
        }
    } else {
        if (kind == "wikidata_property") {
            if (role == "predicate") {
                format!("{}{}", (String::from("formalization:predicate_p:")), id)
            } else {
                format!("{}{}", (String::from("formalization:property_p:")), id)
            }
        } else {
            if ((kind == "wikipedia_article") || (kind == "wiktionary_entry")) {
                format!("{}{}", (String::from("formalization:fallback:")), id)
            } else {
                format!("{}{}", (String::from("formalization:raw:")), id)
            }
        }
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers call of an imported function | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .some() | call of an imported function | method call .slice() | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers REQUIREMENT_TOKENS.some | TASK_TOKENS.some | arrow callback of .some() | call of a sibling function | field access | method call .has() | sibling value

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers call of an imported function | field access | global call String() | method call .push() | method call .writeProgramParameters() | null | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers Array.from | JSDoc type {…} | call of a sibling function | call of an imported function | destructuring | field access | global call String() | method call .solverPromotedHandlers() | method call .solverRouteForPrompt() | null | object without a $ tag | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | destructuring | field access | method call .push() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | field access | method call .push() | null | object without a $ tag
