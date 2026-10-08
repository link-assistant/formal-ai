// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=c4e2a454ea3859ae5dd3df0d9282716070e4a8fb61fb70e7b0d9e9e6a37be7ab bytes=6363
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a8291c230c7291bf0fcf89f4d18e785a32a1c69d22d8768465a40ebe39d63d28
// | /** The role whose words mark a unit as stating an obligation. */
// | const OBLIGATION_ROLE = 'requirement_obligation_cue';
pub const OBLIGATION_ROLE: &str = "requirement_obligation_cue";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=388834374cbf3146728c4fb0786b1f342dacdcff78d759b55468c04eb819a4e5
// | /** The role whose words, opening a unit, mark it as an instruction. */
// | const DIRECTIVE_ROLE = 'requirement_directive_verb';
pub const DIRECTIVE_ROLE: &str = "requirement_directive_verb";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=55e89e11d93dd1e8282a9db8a18368cefd97af6e9f6aa8ced7f3fc4ff774b7e3
// | /** The role whose words, in a heading, mark the section below as requirements. */
// | const SECTION_ROLE = 'requirement_section_heading';
pub const SECTION_ROLE: &str = "requirement_section_heading";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=51555a549582cf7e83d20aab6fbe9f78bd063f398f3d369ee8c57df01fedf27f
// | /** The role of words that carry no content of their own. */
// | const FUNCTION_WORD_ROLE = 'statement_function_word';
pub const FUNCTION_WORD_ROLE: &str = "statement_function_word";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=7def4a3fb1184811ff4c5d099abd8d1ed35cc645fd1210f5039e8e7d9ff4584d
// | /** A content word has at least this many letters. */
// | const CONTENT_WORD_LENGTH = 3;
pub const CONTENT_WORD_LENGTH: f64 = 3f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d8dfa0a073fbcab141c8dc9b471819cacbb1fbd19efe30f0e65078c0f202151c
// | /** Words are compared by this many leading letters, so inflections meet. */
// | const STEM_LENGTH = 6;
pub const STEM_LENGTH: f64 = 6f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a119825a5c52f6fb16d074f3235262447096a5b8fefa3f14007222ec684730d9
// | /** Two units whose content words overlap this much say the same thing. */
// | const RESTATEMENT_OVERLAP = 0.8;
pub const RESTATEMENT_OVERLAP: f64 = 0.8f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=76d352d98291cc2445db7f443eddc7c57885ccff75eab19f5ace8c9bbc924a9d
// | /** The Markdown fences that open and close a code block. */
// | const FENCES = ['```', '~~~'];
pub static FENCES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from("```"), String::from("~~~")]);

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unsupported string escape \s

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .has()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
