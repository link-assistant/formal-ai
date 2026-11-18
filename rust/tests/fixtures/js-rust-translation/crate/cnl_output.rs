// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=effc21f83f3b0c9278e89893d4a4a5802ab8086789321776e8de9c60c7456d29 bytes=27453
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)
// formal-ai:workarounds import-pruning items=1

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

// formal-ai:workaround import-pruning JavaScript import_statement items=1 sha256=89fa19fd048f90612d93493d5441dc96df70addf3f73ed5105128847ea0eb7ee
// | import {
// |   CNL_LANGUAGE_PATH, allPairs, answerableQuestions, builtinCheck, canonicalPrompt, compileGrammar, graphHolds,
// |   graphKey, graphLogic, outputSentences, parseGrammarPolicy, parseLanguageGrammar, promptAnalysis, realizations,
// |   utteranceGraphs,
// | } from './cnl_grammar.mjs';
use crate::cnl_grammar::CNL_LANGUAGE_PATH;

// meta-language:translated JavaScript export_statement items=1 sha256=944558e206c5d5b92ddcf91572bdb24dd3f45d1a6813127db2592aba83d66d85
// | export const CNL_POLICY_PATH = 'data/seed/cnl-output.lino';
pub const CNL_POLICY_PATH: &str = "data/seed/cnl-output.lino";

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers CNL_LANGUAGE_PATH.replace | arrow function | call of a sibling function | call of an imported function | imported value | null | object without a $ tag | try statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …
// formal-ai:blockers arrow callback of .map() | arrow function | assignment of a field or element | call of an imported function | field access | global value Boolean | method call .filter() | method call .map() | method call .split() | null | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal regular expression
// formal-ai:blockers method call .slice() | method call .test() | null | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .find() | arrow callback of .some() | call of a sibling function | field access | method call .find() | method call .some() | null | nullish coalescing | object without a $ tag | typeof operator | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal typeof operator
// formal-ai:blockers arrow callback of .filter() | arrow callback of .some() | call of a sibling function | field access | method call .filter() | method call .indexOf() | method call .push() | method call .slice() | method call .some() | object without a $ tag | optional chaining | typeof operator

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .map() | arrow callback of .some() | call of an imported function | field access | method call .at() | method call .map() | method call .push() | method call .some() | object spread | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | call of an imported function | field access | method call .filter() | method call .map()

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unsupported string escape \p
// formal-ai:blockers method call .match() | nullish coalescing | regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()
// formal-ai:blockers method call .slice()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .filter() | arrow callback of .find() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | field access | method call .filter() | method call .find() | method call .join() | method call .map() | method call .push() | method call .sort() | method call .split() | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | arrow callback of .flatMap() | arrow callback of .map() | call of a sibling function | call of an imported function | field access | method call .find() | method call .flatMap() | method call .map() | new Set | null

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: one value is used as a boolean and as a number; declare the types of the function with JSDoc
// formal-ai:blockers field access

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal arrow function
// formal-ai:blockers arrow callback of .flatMap() | arrow callback of .map() | arrow callback of .sort() | arrow function | call of a sibling function | field access | method call .flatMap() | method call .indexOf() | method call .map() | method call .sort()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .filter() | arrow callback of .map() | arrow callback of .sort() | call of a sibling function | call of an imported function | field access | method call .filter() | method call .get() | method call .map() | method call .sort() | new Map | object without a $ tag | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .join()
// formal-ai:blockers arrow function | call of a sibling function | call of an imported function | field access | method call .get() | method call .has() | method call .join() | method call .set() | null | object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression
// formal-ai:blockers new Set

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()
// formal-ai:blockers arrow callback of .filter() | field access | method call .filter() | method call .has() | sibling value

// meta-language:translated JavaScript function_declaration items=3 sha256=1a5a4a6e0962aae8e89404e02d93139500f86d7c0a6a4edeeda09feed42e58d0
// | /**
// |  * Whether one output cost tuple is lexicographically below another of equal length.
// |  * @param {Array<number>} left
// |  * @param {Array<number>} right
// |  * @returns {boolean}
// |  */
// | function cheaper(left, right) {
// |   for (let index = 0; index < left.length; index += 1) {
// |     if (left[index] !== right[index]) return left[index] < right[index];
// |   }
// |   return false;
// | }
pub fn cheaper(left: Vec<f64>, right: Vec<f64>) -> bool {
    {
        let index = 0f64;
        {
            let ml_s1 = crate::ml_cheaper_loop1(left.clone(), right.clone(), index);
            match ml_s1.clone() {
                crate::MlCheaperLoop1Result::MlCheaperLoop1Done => {
                    false
                }
                crate::MlCheaperLoop1Result::MlCheaperLoop1Return(ml_result1) => {
                    ml_result1
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MlCheaperLoop1Result {
    MlCheaperLoop1Done,
    MlCheaperLoop1Return(bool),
}

pub fn ml_cheaper_loop1(mut left: Vec<f64>, mut right: Vec<f64>, mut index: f64) -> crate::MlCheaperLoop1Result {
    loop {
        return if (index < ((left.len() as f64))) {
            if ((crate::ml_array::at(&left, crate::ml_array::number_index(index))) != (crate::ml_array::at(&right, crate::ml_array::number_index(index)))) {
                crate::MlCheaperLoop1Result::MlCheaperLoop1Return(((crate::ml_array::at(&left, crate::ml_array::number_index(index))) < (crate::ml_array::at(&right, crate::ml_array::number_index(index)))))
            } else {
                {
                    let index_2 = (index + 1f64);
                    {
                        index = index_2;
                        continue;
                    }
                }
            }
        } else {
            crate::MlCheaperLoop1Result::MlCheaperLoop1Done
        };
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal optional chaining
// formal-ai:blockers assignment of a field or element | call of a sibling function | field access | method call .unshift() | new Set | null | nullish coalescing | optional chaining | undefined

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated string literal
// formal-ai:blockers arrow callback of .find() | arrow callback of .map() | arrow callback of .some() | arrow function | call of a sibling function | call of an imported function | field access | method call .find() | method call .has() | method call .map() | method call .push() | method call .replaceAll() | method call .some() | method call .split() | method call .test() | new Set | null | nullish coalescing | object spread | object without a $ tag | optional chaining | regular expression | undefined

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | nullish coalescing | object without a $ tag | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | field access | object without a $ tag

// meta-language:carried JavaScript export_statement (syntax)
// formal-ai:refusal syntax: unterminated template literal
// formal-ai:blockers assignment of a field or element | call of a sibling function | destructuring | field access | method call .replace() | method call .search() | method call .slice() | method call .split() | method call .test() | null | object without a $ tag | regular expression | try statement | typeof operator

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSON.stringify
// formal-ai:blockers JSON.stringify | Object.entries | assignment of a field or element | call of a sibling function | call of an imported function | destructuring | field access | method call .join() | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .find() | call of a sibling function | field access | method call .find() | method call .reverse() | null | nullish coalescing | object spread | object without a $ tag | optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .find()
// formal-ai:blockers arrow callback of .find() | field access | method call .find() | method call .slice() | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .map()
// formal-ai:blockers arrow callback of .map() | call of a sibling function | field access | method call .join() | method call .map() | nullish coalescing | object without a $ tag

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers call of a sibling function | call of an imported function | field access | nullish coalescing | object spread | object without a $ tag | optional chaining

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .forEach() | call of a sibling function | global call String() | method call .forEach() | method call .push() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag
// formal-ai:blockers arrow callback of .findLast() | arrow callback of .map() | arrow function | call of a sibling function | call of an imported function | destructuring | field access | global call String() | global value Boolean | method call .at() | method call .exec() | method call .filter() | method call .findLast() | method call .map() | null | nullish coalescing | object without a $ tag | optional chaining | regular expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | async function | call of a sibling function | field access | null | object spread | object without a $ tag
