// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=ac8d3a74202670111bc7b815699b2a1781b0e2cf3bced35af588556daf62ca0a bytes=10233
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

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript class_declaration (unsupported)
// formal-ai:refusal top-level class statement

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal throw new …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

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

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .find()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)
