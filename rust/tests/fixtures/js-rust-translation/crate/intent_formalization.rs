// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=e7dbdfa1df9ef9e03fc90c2555792bdff1c351a8cbfb99d5515a763e3b85a73b bytes=10108
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

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=344b43fb7b585894fce9c383a58d1c8a44b96a2b1047eff671b7adf460232bf4
// | const ROLE_INTERROGATIVE_OPENER = 'interrogative_opener';
pub const ROLE_INTERROGATIVE_OPENER: &str = "interrogative_opener";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a78be79804c881d1448d9c35e70594697a82abd084d2f0b089143b94bb6816e3
// | const TRANSLATION_PREDICATE = 'wikidata:P5972';
pub const TRANSLATION_PREDICATE: &str = "wikidata:P5972";

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f00d29ff5b66242f5a6b5502ad494bf9dfa08ca7289c4604fe486565a2976f60
// | const REQUIREMENT_TOKENS = ['must', 'should', 'require', 'requires'];
pub static REQUIREMENT_TOKENS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("must"), String::from("should"), String::from("require"), String::from("requires")]);

// meta-language:translated JavaScript lexical_declaration items=1 sha256=9bfb0c05f5bc2184c80fa17acc6b4492373e615828754d291162527f1f697a50
// | const TASK_TOKENS = ['translate', 'write', 'calculate', 'search', 'find', 'prove', 'define'];
pub static TASK_TOKENS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("translate"), String::from("write"), String::from("calculate"), String::from("search"), String::from("find"), String::from("prove"), String::from("define")]);

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression

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

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
