// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=8c8379b0f5a9f742e01a91151dfed20e147104a80eebcbc5964bd49949ca88de bytes=4623
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

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

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access

// meta-language:translated JavaScript function_declaration items=3 sha256=cabf218699fd54a2915da1e94b5775d840ab442b92bc42b43536852abd1027f1
// | /**
// |  * Lexicographic comparison of two number tuples of equal length: negative,
// |  * zero or positive.
// |  * @param {Array<number>} left
// |  * @param {Array<number>} right
// |  * @returns {number}
// |  */
// | function compareKeys(left, right) {
// |   for (let index = 0; index < left.length; index += 1) {
// |     if (left[index] !== right[index]) {
// |       return left[index] < right[index] ? -1 : 1;
// |     }
// |   }
// |   return 0;
// | }
pub fn compare_keys(left: Vec<f64>, right: Vec<f64>) -> f64 {
    {
        let index = 0f64;
        {
            let ml_s1 = crate::ml_compare_keys_loop1(left.clone(), right.clone(), index);
            match ml_s1.clone() {
                crate::MlCompareKeysLoop1Result::MlCompareKeysLoop1Done => {
                    0f64
                }
                crate::MlCompareKeysLoop1Result::MlCompareKeysLoop1Return(ml_result1) => {
                    ml_result1
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MlCompareKeysLoop1Result {
    MlCompareKeysLoop1Done,
    MlCompareKeysLoop1Return(f64),
}

pub fn ml_compare_keys_loop1(mut left: Vec<f64>, mut right: Vec<f64>, mut index: f64) -> crate::MlCompareKeysLoop1Result {
    loop {
        return if (index < ((left.len() as f64))) {
            if ((crate::ml_array::at(&left, crate::ml_array::number_index(index))) != (crate::ml_array::at(&right, crate::ml_array::number_index(index)))) {
                crate::MlCompareKeysLoop1Result::MlCompareKeysLoop1Return(if ((crate::ml_array::at(&left, crate::ml_array::number_index(index))) < (crate::ml_array::at(&right, crate::ml_array::number_index(index)))) {
                    (-1f64)
                } else {
                    1f64
                })
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
            crate::MlCompareKeysLoop1Result::MlCompareKeysLoop1Done
        };
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | call of a sibling function

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | arrow callback of .filter() | arrow callback of .map() | arrow callback of .reduce() | field access | method call .filter() | method call .map() | method call .push() | method call .reduce() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers JSDoc type {…} | field access | null

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers arrow callback of .filter() | arrow callback of .sort() | call of a sibling function | field access | method call .filter() | method call .sort() | object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
// formal-ai:blockers call of a sibling function | null | nullish coalescing
