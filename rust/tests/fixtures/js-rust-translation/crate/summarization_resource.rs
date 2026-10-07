// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=4064a671905cdf30d7b5c08fb7d0e00f33ded8ea3a9d058bb154c7857084f084 bytes=9064
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal field access

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:translated JavaScript function_declaration items=1 sha256=d19ce43bd6f79ec4c3c7ad7b57c3c6a81cfd0d10fc79bdb1f3a46318fcd92027
// | /**
// |  * Mirrors `const fn pluralize` in rust/src/summarization/resource.rs.
// |  * @param {number} count
// |  * @param {string} singular
// |  * @param {string} plural
// |  * @returns {string}
// |  */
// | function pluralize(count, singular, plural) {
// |   return count === 1 ? singular : plural;
// | }
pub fn pluralize(count: f64, singular: String, plural: String) -> String {
    if (count == (1f64)) {
        singular.clone()
    } else {
        plural.clone()
    }
}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal case test

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal JSDoc type {…}
