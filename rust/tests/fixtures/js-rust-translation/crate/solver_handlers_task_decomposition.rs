// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=674a812d1492936aa51b253d5901b527bafb4d49f347e56336aee86d7e7425cc bytes=5124
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=6

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

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

// meta-language:translated JavaScript import_statement items=1 sha256=e9510558b3d8c6f2baeb8831106f535421d27d9965de4c4c2f6d47c7ae50c6e1
// | import {DEFAULT_SPLIT_DEPTH_BOUND} from './task_decomposition.mjs';
use crate::task_decomposition::DEFAULT_SPLIT_DEPTH_BOUND;

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:translated JavaScript lexical_declaration items=1 sha256=a2a7ebc608b0575f2e42c804b1a30fb701db71db896b923024256b427db7115d
// | const ROLE_DECOMPOSABLE_TASK_NOUN = 'decomposable_task_noun';
pub const ROLE_DECOMPOSABLE_TASK_NOUN: &str = "decomposable_task_noun";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=aa3c1025e446fa25b1a78ab2f2c32ef49dd7bf3160a9aa01dfa836924f5fa5a5
// | const ROLE_SUBTASK_UNIT_NOUN = 'subtask_unit_noun';
pub const ROLE_SUBTASK_UNIT_NOUN: &str = "subtask_unit_noun";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=af6a990005de1888463bd71721df43b6c6863fb04f5a5d9697a0ca0c0d8dc300
// | const ROLE_TASK_ATOMICITY_PREDICATE = 'task_atomicity_predicate';
pub const ROLE_TASK_ATOMICITY_PREDICATE: &str = "task_atomicity_predicate";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=f026f0f7355a5e024cf8b0e1780bffa558ea6213ebb0332017890df35a125281
// | const ROLE_TASK_FIRST_STEP_CUE = 'task_first_step_cue';
pub const ROLE_TASK_FIRST_STEP_CUE: &str = "task_first_step_cue";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=31deb65be1cab3dc62f97770473f0d34034166435f59122b47a9bd35b081fc38
// | const ROLE_TASK_DECOMPOSITION_ACTION = 'task_decomposition_action';
pub const ROLE_TASK_DECOMPOSITION_ACTION: &str = "task_decomposition_action";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=fb3e44daaac62f00c7dc33285dd282a1a609ef8d62a9a64293eac8949cf72717
// | const ROLE_SUBTASK_ENUMERATION_CUE = 'subtask_enumeration_cue';
pub const ROLE_SUBTASK_ENUMERATION_CUE: &str = "subtask_enumeration_cue";

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function statement
// formal-ai:blockers field access | function value … | method call .push()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of an imported function | nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of a sibling function | field access | method call .join() | method call .push() | method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | field access | let without a value | null | nullish coalescing | object without a $ tag | optional chaining
