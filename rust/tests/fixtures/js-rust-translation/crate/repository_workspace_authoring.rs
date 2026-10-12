// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=6e3ea27566748703ab892d9abbfaf27d57d3be6cfaaa67bd710a9b15059fd570 bytes=2004
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=9c6874396309a41b38582b1ad095039f9b3e2080fed66b388d0809620d3843b4
// | // Authoring uses the same declared repository stages through caller-owned ports.
// | import { EDITOR_AGENT_SESSION, loadProtocol, newProtocolTrace, recordStage, renderProtocolTrace,
// |   stepAppliesTo } from './repository_workspace.mjs';
use crate::repository_workspace::EDITOR_AGENT_SESSION;

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal destructured parameter
// formal-ai:blockers arrow callback of .some() | arrow function | assignment of a field or element | async function | call of an imported function | field access | let without a value | method call .push() | method call .some() | null | nullish coalescing | object without a $ tag | try statement | typeof operator
