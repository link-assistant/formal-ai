// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=6c53c98c7ad63c0510aee65feedaab7eb38f6fcf5ebbe036a7318d8c7d5650cb bytes=24940
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

// meta-language:carried JavaScript import_statement (unsupported)
// formal-ai:refusal import { … }

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal object without a $ tag

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .lastIndexOf()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal Array.from

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .map()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .filter()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal arrow function

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .split()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal method call .some()

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal optional chaining

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal optional chaining

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal new expression

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript export_statement (unsupported)
// formal-ai:refusal function value …

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal nullish coalescing

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal regular expression

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal method call .some()

// meta-language:translated JavaScript function_declaration items=1 sha256=69b23bc55472518e655383b36f26bdf81635fec121124e72b504fc7a46a4079a
// | /**
// |  * Whether `normalized` holds `marker`; a marker padded with spaces matches whole words only.
// |  * @param {string} normalized
// |  * @param {string} marker
// |  * @returns {boolean}
// |  */
// | function containsSearchMarker(normalized, marker) {
// |   if (marker.startsWith(' ') || marker.endsWith(' ')) return ` ${normalized} `.includes(marker);
// |   return normalized.includes(marker);
// | }
pub fn contains_search_marker(normalized: String, marker: String) -> bool {
    if (marker.starts_with(" ") || marker.ends_with(" ")) {
        (format!("{}{}", (format!("{}{}", (String::from(" ")), normalized)), (String::from(" ")))).contains(marker.as_str())
    } else {
        normalized.contains(marker.as_str())
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal undefined

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript lexical_declaration (type)
// formal-ai:refusal type: unknown name (a sibling item or an import)

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .some()

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript lexical_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .indexOf()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal method call .slice()

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null

// meta-language:carried JavaScript function_declaration (unsupported)
// formal-ai:refusal null
