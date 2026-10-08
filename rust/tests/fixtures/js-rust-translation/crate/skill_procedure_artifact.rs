// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=1d929512c58691ed9b4acb2116460e5d2c98c19724f05ee046d5ecab60b2fb7a bytes=10247
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1 carried=5

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]

/// Reads of JavaScript arrays, which the portable core never mutates.
pub mod ml_array {
    /// The element at an index; a read outside the array, undefined in
    /// JavaScript, aborts.
    pub fn at<T: Clone>(values: &[T], index: Option<usize>) -> T {
        match index.and_then(|index| values.get(index)) {
            Some(value) => value.clone(),
            None => panic!("array index out of range"),
        }
    }

    /// The index a Number names: a non-negative integer, -0 included.
    pub fn number_index(index: f64) -> Option<usize> {
        if index >= 0.0 && index.fract() == 0.0 && index < 9007199254740992.0 {
            Some(index as usize)
        } else {
            None
        }
    }

    pub fn append<T>(mut left: Vec<T>, right: Vec<T>) -> Vec<T> {
        left.extend(right);
        left
    }
}
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

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning carried JavaScript import_statement
// formal-ai:refusal import of names its module does not translate
// formal-ai:blockers import of names its module carries

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=07b960894096992bed2c87e4cb0bf65152b802e97957073735855b39a82add61
// | import {
// |   KNOWLEDGE_SCHEMA_VERSION, MINIMUM_STEPS, ROLE_SKILL_PROCEDURE_STEP_OBJECT, ROLE_SKILL_PROCEDURE_STEP_VERB,
// |   ROLE_TRANSLATION_LANGUAGE, canonicalProgram, meaningHasRole, requirementId, stepId,
// | } from './skill_procedure.mjs';
use crate::skill_procedure::{KNOWLEDGE_SCHEMA_VERSION, MINIMUM_STEPS, ROLE_SKILL_PROCEDURE_STEP_OBJECT, ROLE_SKILL_PROCEDURE_STEP_VERB, ROLE_TRANSLATION_LANGUAGE};

// meta-language:carried JavaScript class_declaration (unsupported)
// formal-ai:refusal top-level class statement
// formal-ai:blockers class

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow function | call of an imported function | field access | global call String() | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal throw new …
// formal-ai:blockers call of an imported function | field access | new ProcedureArtifactError

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers call of a sibling function | field access | method call .replace() | method call .test() | new ProcedureArtifactError | regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow function | field access | method call .filter() | method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
// formal-ai:blockers call of an imported function | method call .slice() | null

// meta-language:translated JavaScript lexical_declaration items=1 sha256=79ab840611c941c9a315836b741b7bfe6a11746e95cd14797a500aec77a139d0
// | /**
// |  * Whether two `[start, end]` source spans are the same span.
// |  * @param {number[]} left
// |  * @param {number[]} right
// |  * @returns {boolean}
// |  */
// | const sameSpan = (left, right) => left[0] === right[0] && left[1] === right[1];
pub fn same_span(left: Vec<f64>, right: Vec<f64>) -> bool {
    (((crate::ml_array::at(&left, crate::ml_array::number_index(0f64))) == (crate::ml_array::at(&right, crate::ml_array::number_index(0f64)))) && ((crate::ml_array::at(&left, crate::ml_array::number_index(1f64))) == (crate::ml_array::at(&right, crate::ml_array::number_index(1f64)))))
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .findIndex() | arrow callback of .flatMap() | arrow callback of .forEach() | arrow callback of .some() | call of a sibling function | call of an imported function | field access | method call .findIndex() | method call .flatMap() | method call .forEach() | method call .some() | new ProcedureArtifactError | null | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .find() | method call .map() | new ProcedureArtifactError | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function | instanceof operator | method call .indexOf() | method call .slice() | null | sibling value | throw of a non-error value | try statement

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
// formal-ai:blockers call of an imported function
