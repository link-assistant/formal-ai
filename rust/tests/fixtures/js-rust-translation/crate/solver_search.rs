// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=7d69c25ef17668b01b077c67cec20db145016c3209108ccdded29f89f10c7340 bytes=1438
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d049ac7adde1e028334ad98cd427f347780eca1d049abb7d818a178145a1cd43
// | const MAX_OPERANDS = 6;
pub const MAX_OPERANDS: f64 = 6f64;

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .match()
// formal-ai:blockers call of an imported function | method call .match() | method call .push() | null | regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | arrow function | call of an imported function | method call .some()

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: function value MAX_OPERANDS: functions and namespaces are only portable when a function is called
// formal-ai:blockers call of a sibling function | call of an imported function
