// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=add5df7f1a6dff5f0b8816095d737d02ec9b4e56819593b1199338ea5209da18 bytes=31584
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=5

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

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=2b009bdbd8e6faca6793df38c651d90c599842ae1019c96cd2bc85e1edfff84c
// | import { NULL_LINK, SequenceStore, SymbolTable, balancedConvert, compress } from './sequences.mjs';
use crate::sequences::NULL_LINK;

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import { … }

// meta-language:translated JavaScript lexical_declaration items=1 sha256=ea2291a713bdc97af4de7987115368aaf67d29e42a2c819b48ed5e898f01700a
// | const DEFAULT_MIN_STEPS = 2;
pub const DEFAULT_MIN_STEPS: f64 = 2f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=82aec5708d7e17cbdf48272c8c1d9dd8d3095214c883183a48490b235d03c330
// | const DEFAULT_SUPPORT_OCCURRENCES = 2;
pub const DEFAULT_SUPPORT_OCCURRENCES: f64 = 2f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=06d81550f679cb4e998e0a84f98b9d53b88ba30361a84a023f02c350529a5655
// | const DEFAULT_HELD_OUT_OCCURRENCES = 1;
pub const DEFAULT_HELD_OUT_OCCURRENCES: f64 = 1f64;

// meta-language:translated JavaScript export_statement items=1 sha256=b975878b60096ed668d0a902c836c3b4722f8423d62c9b57b1dc022095499ae3
// | /** Mirrors `MAX_DISCOVERY_INPUT_STEPS` in rust/src/algorithm_discovery.rs. */
// | export const MAX_DISCOVERY_INPUT_STEPS = 4096;
pub const MAX_DISCOVERY_INPUT_STEPS: f64 = 4096f64;

// meta-language:translated JavaScript export_statement items=1 sha256=480f3d3ac027acde48cedc366faad56c6ed830001ba3ca22ed9b97e336cca4de
// | /** Mirrors `MAX_DISCOVERED_ALGORITHM_STEPS` in rust/src/algorithm_discovery.rs. */
// | export const MAX_DISCOVERED_ALGORITHM_STEPS = 32;
pub const MAX_DISCOVERED_ALGORITHM_STEPS: f64 = 32f64;

// meta-language:translated JavaScript lexical_declaration items=1 sha256=cafa27b6be30decb0a662c2a5df11f2a87a1fbb696ca77f45a0f3fe6a30f6a8b
// | /**
// |  * Rust `Ord::cmp` for integers.
// |  * @param {number} left
// |  * @param {number} right
// |  * @returns {number}
// |  */
// | const cmpNum = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
pub fn cmp_num(left: f64, right: f64) -> f64 {
    if (left < right) {
        (-1f64)
    } else {
        if (left > right) {
            1f64
        } else {
            0f64
        }
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .sort() | call of an imported function | destructuring | method call .entries() | method call .set() | method call .sort() | new Map

// meta-language:carried JavaScript function_declaration (syntax)
// formal-ai:refusal syntax: unsupported string escape \b
// formal-ai:blockers String.fromCharCode | arrow function | destructuring | global call parseInt() | in operator | method call .charCodeAt() | method call .exec() | method call .push() | method call .set() | method call .slice() | method call .test() | new Map | new SyntaxError | null | object without a $ tag | regular expression | try statement | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal Object.is
// formal-ai:blockers Object.is | assignment of a field or element | destructuring | let without a value | method call .repeat() | method call .replace() | method call .slice() | method call .split() | method call .toExponential()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal BigInt
// formal-ai:blockers call of a sibling function | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSON.stringify
// formal-ai:blockers JSON.stringify | arrow callback of .map() | call of a sibling function | field access | global call String() | imported value | method call .get() | method call .join() | method call .keys() | method call .map() | method call .sort()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of a sibling function | field access | method call .some() | method call .values() | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .entries()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .entries() | method call .indexOf() | method call .map() | method call .push() | method call .slice() | method call .split() | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .map() | arrow callback of .sort() | call of a sibling function | call of an imported function | field access | method call .entries() | method call .get() | method call .has() | method call .map() | method call .push() | method call .set() | method call .sort() | new Map | null | nullish coalescing | object without a $ tag | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | field access | method call .every()

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal for with const
// formal-ai:blockers call of a sibling function | destructuring | field access | undefined

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function | field access | global call String()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | method call .join() | method call .map()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow function | field access | method call .find() | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers arrow callback of .forEach() | arrow callback of .some() | call of a sibling function | destructuring | field access | global call String() | method call .forEach() | method call .get() | method call .has() | method call .push() | method call .set() | method call .some() | new Map | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers arrow callback of .sort() | field access | method call .get() | method call .push() | method call .set() | method call .slice() | method call .sort() | new Map | nullish coalescing

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | arrow function | method call .every()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers JSON.stringify | arrow callback of .every() | arrow callback of .findIndex() | arrow callback of .forEach() | arrow callback of .map() | arrow callback of .some() | arrow callback of .sort() | arrow function | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | let without a value | method call .every() | method call .findIndex() | method call .forEach() | method call .get() | method call .has() | method call .map() | method call .push() | method call .set() | method call .slice() | method call .some() | method call .sort() | new Map | new Set | null | object without a $ tag | undefined

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .every() | arrow callback of .map() | arrow function | field access | method call .every() | method call .has() | method call .map() | new Set

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow callback of .reduce() | arrow callback of .sort() | call of an imported function | field access | method call .filter() | method call .has() | method call .map() | method call .reduce() | method call .sort() | new Set | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .flatMap() | arrow callback of .forEach() | arrow callback of .map() | arrow callback of .reduce() | arrow callback of .some() | arrow callback of .sort() | call of a sibling function | call of an imported function | destructuring | field access | method call .flat() | method call .flatMap() | method call .forEach() | method call .get() | method call .has() | method call .isLossless() | method call .join() | method call .map() | method call .marker() | method call .push() | method call .reduce() | method call .set() | method call .slice() | method call .some() | method call .sort() | method call .values() | new Map | new SequenceStore | new Set | new SymbolTable | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access | method call .filter() | sibling value

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal null
// formal-ai:blockers arrow callback of .forEach() | call of a sibling function | call of an imported function | destructuring | field access | global call String() | method call .forEach() | null | object without a $ tag | undefined

// meta-language:carried JavaScript class_declaration (unsupported)
// formal-ai:refusal top-level class statement
// formal-ai:blockers class

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers field access | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | new InvalidArtifact

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal throw new …
// formal-ai:blockers call of a sibling function | call of an imported function | new InvalidArtifact

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .replace() | method call .test() | new InvalidArtifact | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | instanceof operator | null | sibling value | throw of a non-error value | try statement

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal throw new …
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | call of an imported function | field access | let without a value | method call .entries() | method call .filter() | method call .find() | method call .map() | method call .push() | method call .set() | method call .sort() | new InvalidArtifact | new Map | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | call of an imported function | destructuring | field access | global call String() | let without a value | method call .entries() | method call .get() | method call .has() | null | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .every()
// formal-ai:blockers arrow callback of .every() | arrow function | assignment of a field or element | destructuring | field access | method call .every() | object without a $ tag
