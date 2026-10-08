// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=7c3b0a092630f81211c628785546f29db5b35e3336f995f56f6ecca4a0af2c42 bytes=4921
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds string-methods items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround-prelude begin
pub fn wa_str_split(text: String, separator: String) -> Vec<String> {
    if separator.is_empty() {
        return text.encode_utf16().map(|unit| String::from_utf16_lossy(&[unit])).collect();
    }
    text.split(separator.as_str()).map(String::from).collect() }

pub fn wa_strs_join(items: Vec<String>, separator: String) -> String {
    items.join(separator.as_str()) }
// formal-ai:workaround-prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow callback of .reduce() | call of an imported function | field access | method call .find() | method call .join() | method call .reduce() | method call .split() | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | call of an imported function

// meta-language:translated JavaScript lexical_declaration items=1 sha256=b55651f3d28fe19c1b901064fdcb2a0d94b0a320dd4f50dc92081ee43d8d0f4f
// | const RECORD_STEP = 'meta_step';
pub const RECORD_STEP: &str = "meta_step";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=56fbf284f6d44a1a33c1779daee71c4bf4247bc7a38ba5d9eede370dd6044afa
// | const STEP_EDITOR_ANY = 'any';
pub const STEP_EDITOR_ANY: &str = "any";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=fa7daff26a6febfba4b6dc8f85b4ec72c63867593e4538e2c8b80e24dbe8bd9d
// | const TRACE_RECORD = 'repository_protocol_trace';
pub const TRACE_RECORD: &str = "repository_protocol_trace";

// meta-language:translated JavaScript lexical_declaration items=1 sha256=378bfeccdf36ff57076671e653d99aa98218555811ec6dc2b2ea768f645aad9a
// | const STAGE_FIELD = 'stage';
pub const STAGE_FIELD: &str = "stage";

// meta-language:translated JavaScript export_statement items=1 sha256=9818903b3f87d02f6225ffc10403a45554ef06f0bd335abb9b4c177bcf1dd72c
// | /** Mirrors `const EDITOR_STRUCTURAL` (rust/src/repository_workspace/trace.rs). */
// | export const EDITOR_STRUCTURAL = 'structural';
pub const EDITOR_STRUCTURAL: &str = "structural";

// meta-language:translated JavaScript export_statement items=1 sha256=3c9e898d8281b2bc72aa38d177135e624808ff4a30c5f757e59b697f9e02986f
// | /** Mirrors `const EDITOR_AGENT_SESSION`. */
// | export const EDITOR_AGENT_SESSION = 'agent_session';
pub const EDITOR_AGENT_SESSION: &str = "agent_session";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers Number.parseInt | arrow callback of .filter() | arrow callback of .map() | arrow callback of .sort() | call of an imported function | field access | method call .filter() | method call .map() | method call .sort() | object without a $ tag

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | call of an imported function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .map() | call of a sibling function | field access | method call .map() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | assignment of a field or element | field access | method call .find()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | assignment of a field or element | field access | method call .find() | method call .push()

// formal-ai:workaround string-methods JavaScript function_declaration items=1 sha256=f8f65ed4acbe9a32d2573730957136515b90214786e7716af4f46ea4b8de12a7
// | /** @param {string} value */
// | function quote(value) {
// |   return value.split('"').join('""');
// | }
// ~ /** @param {string} value */
// ~ function quote(value) {
// ~   return waStrsJoin(waStrSplit(value, '"'), '""');
// ~ }
pub fn quote(value: String) -> String {
    crate::wa_strs_join(crate::wa_str_split(value.clone(), String::from("\"")), String::from("\"\""))
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers destructuring | field access | method call .join() | method call .push()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()
// formal-ai:blockers method call .indexOf() | method call .push() | method call .replace() | method call .slice() | method call .split() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | method call .map()
