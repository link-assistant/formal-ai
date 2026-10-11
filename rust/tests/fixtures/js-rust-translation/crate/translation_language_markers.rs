// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=8df9ff3b7c53dc8e0484b49b40c41e617c9963f7ccdcfa60e17f403999026339 bytes=1149
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript lexical_declaration items=1 sha256=3f3d195b0ca582df0a7996615693331de0ad1b93354081aa13402e1fa703a970
// | const ROLE_RESPONSE_LANGUAGE_MARKER = 'response_language_marker';
pub const ROLE_RESPONSE_LANGUAGE_MARKER: &str = "response_language_marker";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d13db6f21d885c6d8f898329f2d6644a70d1194a8f7bb078c7e73a10bc270c9c
// | const ROLE_COMPREHENSION_FAILURE_MARKER = 'comprehension_failure_marker';
pub const ROLE_COMPREHENSION_FAILURE_MARKER: &str = "comprehension_failure_marker";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of an imported function | field access | method call .some() | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: function value ROLE_RESPONSE_LANGUAGE_MARKER: functions and namespaces are only portable when a function is called
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .some() | call of an imported function | method call .some()
