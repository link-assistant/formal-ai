// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=f659b8722ffe22b823913965ba5e2cfae3a8665159a5fd318f80935edb815415 bytes=5030
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=2

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=5caa9eb61c7cdb4002d46299bf968505ebf6c9ecb5379245fc97bfbd075f0ce8
// | import { COMPARISON_RULE, conjunctiveShape, relationOf } from './proposition_structure.mjs';
use crate::proposition_structure::COMPARISON_RULE;

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers Object.freeze | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | arrow function | method call .every()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | arrow function | method call .every()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | call of an imported function | field access | method call .every() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | field access | imported value | object without a $ tag | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers field access | sibling value
