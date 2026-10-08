// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=0dabc46fb9743c53254019ea7df907d267bebe084acd53571c3abc5fe11354b3 bytes=16695
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=0 carried=8

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
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

// meta-language:translated JavaScript export_statement items=1 sha256=7a77f33350e11dba3ee890d6fd9474863eaef5a9aaea84e7f298c94af4bb0841
// | /** Mirrors `MINIMUM_STEPS` in rust/src/skill_procedure.rs. */
// | export const MINIMUM_STEPS = 2;
pub const MINIMUM_STEPS: f64 = 2f64;

// meta-language:translated JavaScript export_statement items=1 sha256=81fc55670934d268b96f34ddeadb9cb3b1678572fba528f51ba9226e95d733dc
// | /** Mirrors `PROCEDURE_CONFORMANCE_TRIGGER` in rust/src/skill_procedure.rs. */
// | export const PROCEDURE_CONFORMANCE_TRIGGER = 'https://example.com/article';
pub const PROCEDURE_CONFORMANCE_TRIGGER: &str = "https://example.com/article";

// meta-language:translated JavaScript export_statement items=1 sha256=6244994c937aead272143f9383eaa05e372c86037d1219af2bf1f2d602e2054a
// | /** Mirrors `crate::engine::KNOWLEDGE_SCHEMA_VERSION`. */
// | export const KNOWLEDGE_SCHEMA_VERSION = '0.2.0';
pub const KNOWLEDGE_SCHEMA_VERSION: &str = "0.2.0";

// meta-language:translated JavaScript export_statement items=1 sha256=cde0402871f3e78fa3554190ba2e80ce8f7de1547bcec204b92356ede29d389a
// | /** `crate::seed::ROLE_SKILL_PROCEDURE_*` and `ROLE_TRANSLATION_LANGUAGE` (rust/src/seed/roles/). */
// | export const ROLE_SKILL_PROCEDURE_TRIGGER_LEAD = 'skill_procedure_trigger_lead';
pub const ROLE_SKILL_PROCEDURE_TRIGGER_LEAD: &str = "skill_procedure_trigger_lead";

// meta-language:translated JavaScript export_statement items=1 sha256=7daeb8bd278c286ce1ca5d0e5991bfb9b7892838673c850f2b0d6d5b07931a81
// | export const ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR = 'skill_procedure_clause_separator';
pub const ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR: &str = "skill_procedure_clause_separator";

// meta-language:translated JavaScript export_statement items=1 sha256=55d561d00aab20c4bae0bdd763e41b853eca9d39cc839f395825d4b97bf1b73a
// | export const ROLE_SKILL_PROCEDURE_STEP_VERB = 'skill_procedure_step_verb';
pub const ROLE_SKILL_PROCEDURE_STEP_VERB: &str = "skill_procedure_step_verb";

// meta-language:translated JavaScript export_statement items=1 sha256=cbabfb11e4e66140b233cfa387007ce6b01430d719add2bd36d3771bb0eff2ae
// | export const ROLE_SKILL_PROCEDURE_STEP_OBJECT = 'skill_procedure_step_object';
pub const ROLE_SKILL_PROCEDURE_STEP_OBJECT: &str = "skill_procedure_step_object";

// meta-language:translated JavaScript export_statement items=1 sha256=8d158b2c9f106d2f5236c7a25e0789642d676b4e9c26e6413970fdb081e915e8
// | export const ROLE_TRANSLATION_LANGUAGE = 'translation_language';
pub const ROLE_TRANSLATION_LANGUAGE: &str = "translation_language";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=cd3ad739916aad33df9c316370435a872ac055e848010a379f51f9896965a18c
// | const PROCEDURE_MEANINGS_FILE = 'data/seed/meanings-skill-procedure.lino';
pub const PROCEDURE_MEANINGS_FILE: &str = "data/seed/meanings-skill-procedure.lino";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=dd52737b655eb80d6b1da1639551550c219014da4bcf32147c4d6935cc65ad94
// | const CAPABILITY_LEDGER_FILE = 'data/meta/procedure-capability-ledger.lino';
pub const CAPABILITY_LEDGER_FILE: &str = "data/meta/procedure-capability-ledger.lino";

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers String.fromCodePoint | global call parseInt() | method call .filter() | method call .isFinite() | method call .slice() | method call .split() | regular expression

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function | field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | field access | method call .filter() | method call .map() | method call .push() | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow function | call of a sibling function | field access | method call .filter()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .flatMap() | arrow callback of .map() | arrow function | field access | method call .filter() | method call .flatMap() | method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .flatMap()
// formal-ai:blockers call of a sibling function | method call .flatMap() | sibling value

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()
// formal-ai:blockers arrow callback of .some() | call of an imported function | field access | method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow callback of .sort() | arrow function | call of an imported function | field access | method call .entries() | method call .filter() | method call .find() | method call .map() | method call .set() | method call .sort() | new Map | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow function | field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers arrow function | method call .test() | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers Array.from | arrow function | call of an imported function | method call .pop() | method call .slice() | method call .some() | sibling value | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers call of a sibling function | call of an imported function | field access | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .map() | arrow callback of .sort() | call of a sibling function | call of an imported function | field access | method call .map() | method call .push() | method call .sort() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing
// formal-ai:blockers call of an imported function | method call .join() | method call .split() | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers field access | null

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access
// formal-ai:blockers call of a sibling function | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers call of a sibling function | call of an imported function | field access | method call .join()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .filter() | arrow callback of .findIndex() | arrow callback of .map() | assignment of a field or element | call of a sibling function | call of an imported function | field access | method call .filter() | method call .findIndex() | method call .map() | method call .slice() | null | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | field access | null | nullish coalescing

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .forEach() | arrow callback of .map() | call of an imported function | field access | global call String() | method call .forEach() | method call .map() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers call of a sibling function | field access | method call .join()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | call of an imported function | field access | global call String() | null
