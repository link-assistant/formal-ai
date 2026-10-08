// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=97d6c3fab208165fee5c49e4f3a51061905063356558891bd859ef2142059a5a bytes=17658
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=9

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=2e06e6b009579075745e00f96caab10f68055b9fd608312b5c574609e68c051f
// | const ROLE_CLAUSE_CONTINUATION_MARKER = 'clause_continuation_marker';
pub const ROLE_CLAUSE_CONTINUATION_MARKER: &str = "clause_continuation_marker";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=d8d7a04e3356aa27e5fed69741702f6f34d858d673164224542d4c9315a0e4ab
// | const ROLE_OBSERVABLE_TASK_ACTION = 'observable_task_action';
pub const ROLE_OBSERVABLE_TASK_ACTION: &str = "observable_task_action";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=fa7173bc88e3889c4b281a0eef7396a69da6a478f17cfb4a3e39d09b048edf34
// | const ROLE_SOFTWARE_AUTHORING_ACTION = 'software_authoring_action';
pub const ROLE_SOFTWARE_AUTHORING_ACTION: &str = "software_authoring_action";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=cc1574e316bb53046bdb5bdf6432f2324921c02b757f421aad1077b9b3f4ccff
// | const ROLE_FOLLOWUP_INSTRUCTION_VERB = 'followup_instruction_verb';
pub const ROLE_FOLLOWUP_INSTRUCTION_VERB: &str = "followup_instruction_verb";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=7daeb8bd278c286ce1ca5d0e5991bfb9b7892838673c850f2b0d6d5b07931a81
// | const ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR = 'skill_procedure_clause_separator';
pub const ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR: &str = "skill_procedure_clause_separator";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=aa538a57021876eea256d966db9def269ada726a1a6506ab51ea64bcb07ea24d
// | const OBLIGATION_ROLES = [ROLE_OBSERVABLE_TASK_ACTION, ROLE_SOFTWARE_AUTHORING_ACTION, ROLE_FOLLOWUP_INSTRUCTION_VERB];
pub static OBLIGATION_ROLES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| vec![String::from(ROLE_OBSERVABLE_TASK_ACTION), String::from(ROLE_SOFTWARE_AUTHORING_ACTION), String::from(ROLE_FOLLOWUP_INSTRUCTION_VERB)]);

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal undefined
// formal-ai:blockers arrow function | method call .test() | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers null

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .filter() | arrow callback of .forEach() | call of a sibling function | method call .filter() | method call .forEach() | method call .has() | method call .push() | method call .slice() | object without a $ tag | sibling value | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers arrow function | global value Boolean | method call .filter() | method call .slice() | method call .split() | method call .test() | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()
// formal-ai:blockers Array.from | call of a sibling function | call of an imported function | method call .indexOf() | method call .pop() | method call .push() | method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal destructured parameter
// formal-ai:blockers arrow callback of .sort() | call of a sibling function | call of an imported function | method call .has() | method call .push() | method call .slice() | method call .sort() | new Set | sibling value

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | global value Boolean | method call .filter() | method call .split() | null | regular expression | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers OBLIGATION_ROLES.some | arrow callback of .some() | call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal destructured parameter
// formal-ai:blockers call of a sibling function | call of an imported function | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal array destructuring
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | method call .push() | method call .slice() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | method call .slice() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Math.log2
// formal-ai:blockers Math.log2 | call of a sibling function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Array.from
// formal-ai:blockers Array.from | arrow callback of .forEach() | arrow callback of .some() | call of a sibling function | call of an imported function | method call .forEach() | method call .has() | method call .push() | method call .some() | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .map() | call of an imported function | field access | method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | global value Boolean | method call .filter() | method call .flatMap() | method call .map() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .map() | object spread | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | field access | global call String() | method call .push() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | field access | global call String() | method call .push() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | null

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | arrow function | field access | method call .reduce()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .reduce()
// formal-ai:blockers arrow callback of .reduce() | arrow function | field access | method call .reduce()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers field access | method call .push()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | field access | global call String() | method call .push() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers field access | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | global call String() | method call .push() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow function | call of a sibling function | field access | method call .encode() | method call .filter() | method call .find() | new TextEncoder

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow function | field access | method call .filter()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers call of a sibling function | call of an imported function | field access | global call String() | method call .push() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .push()
// formal-ai:blockers arrow callback of .every() | call of a sibling function | field access | global call String() | method call .every() | method call .push() | object without a $ tag
