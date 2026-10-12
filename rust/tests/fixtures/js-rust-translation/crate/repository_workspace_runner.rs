// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1bb87b92c289364710e76b0928c9511b8aa54e1af90f392a10f9d97da490b227 bytes=8037
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=3

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import from '…'
// formal-ai:blockers import from outside the module directory

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=14e197d033419ac03aed1194808e40ef0734b4f737dcaf096e8dff83f1c3e243
// | import { EDITOR_STRUCTURAL, loadProtocol, newProtocolTrace, recordStage, renderProtocolTrace,
// |   renderProtocolTemplate, stepAppliesTo } from './repository_workspace.mjs';
use crate::repository_workspace::EDITOR_STRUCTURAL;

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .find() | assignment of a field or element | call of an imported function | field access | imported value | method call .encode() | method call .find() | method call .push() | new TextEncoder | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of an imported function | field access | method call .filter() | method call .map() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal destructured parameter
// formal-ai:blockers arrow callback of .map() | assignment of a field or element | async function | call of a sibling function | call of an imported function | field access | global value Boolean | imported value | method call .census() | method call .filter() | method call .get() | method call .has() | method call .join() | method call .keys() | method call .map() | method call .push() | method call .read() | method call .set() | method call .slice() | method call .sourceFiles() | method call .test() | method call .values() | method call .write() | new Error | new Map | null | nullish coalescing | object without a $ tag | regular expression | try statement
