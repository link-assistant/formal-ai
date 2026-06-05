# Online Research Notes

Date: 2026-06-04

## Sources

- Unicode Standard Annex #24, "Unicode Script Property": https://www.unicode.org/reports/tr24/
- MDN, "Unicode character class escape: \\p{...}, \\P{...}": https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Regular_expressions/Unicode_character_class_escape

## Findings Applied

- Unicode UAX #24 treats script as a core text-processing classification. It explicitly notes that Russian is written with a subset of the Cyrillic script and that script properties are useful for text processing tasks such as regular expressions.
- MDN documents JavaScript Unicode property escapes and specifically shows `Script` / `Script_Extensions` matching, including matching a Cyrillic character with `\\p{sc=Cyrillic}`.
- The reported prompt is Cyrillic/Russian, so the correct product behavior is to keep user-facing recovery guidance in Russian when the unknown route cannot answer. English-only rule-management examples in that route are a localization bug, not a parser limitation.
