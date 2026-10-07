// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=515a10ebf9b4933b4e8447fcfa67cc676adab1a5b3d85d89e7288124b3d0d88d bytes=487
// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:translated JavaScript export_statement items=1 sha256=90e5d4da9e45d229533c8e26e1e3012d9f26c7abb492910f0f33a695ebd5528e
// | /** Mirrors `const LINO_FENCE_LANGUAGE`. */
// | export const LINO_FENCE_LANGUAGE = 'lino';
pub const LINO_FENCE_LANGUAGE: &str = "lino";

// meta-language:translated JavaScript export_statement items=2 sha256=ea42b264877b3ad19b35c3d9ab444a6c6c2e27ba3248dc0ef0ad3ba484c7a5c3
// | /**
// |  * Mirrors `fn fenced_block` in rust/src/issue_report.rs.
// |  * @param {string} language
// |  * @param {string} content
// |  */
// | export function fencedBlock(language, content) {
// |   const body = content.trimEnd();
// |   let fence = '```';
// |   while (body.includes(fence)) fence += '`';
// |   return `${fence}${language}\n${body}\n${fence}`;
// | }
pub fn fenced_block(language: String, content: String) -> String {
    {
        let body = content.trim_end_matches(|c: char| (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}').to_string();
        {
            let fence = String::from("```");
            {
                let fence_2 = crate::ml_fenced_block_loop1(body.clone(), fence.clone());
                format!("{}{}", (format!("{}{}", (format!("{}{}", (format!("{}{}", (format!("{}{}", fence_2, language)), (String::from("\n")))), body)), (String::from("\n")))), fence_2)
            }
        }
    }
}

pub fn ml_fenced_block_loop1(mut body: String, mut fence: String) -> String {
    loop {
        return if body.contains(fence.as_str()) {
            {
                let fence_2 = format!("{}{}", fence, (String::from("`")));
                {
                    fence = fence_2.clone();
                    continue;
                }
            }
        } else {
            fence.clone()
        };
    }
}
