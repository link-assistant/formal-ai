<!-- repo: link-foundation/meta-language -->
<!-- title: Portable-core lexer: regular-expression bodies, valid escapes and optional JSDoc parameters give syntax errors -->

### Problem

On main at `679a3b3c`, the portable-core JavaScript lexer (`js/src/translation/lexer.js`, used by `parseJavaScript`) rejects valid JavaScript with a syntax error. The right result is a precise refusal, or a translation. Each case below goes through `translateProgram(source, 'JavaScript', 'Rust').diagnostic.message`:

| Source (inside a JSDoc-typed function) | Diagnostic | Expected |
| --- | --- | --- |
| `return /x/u.test(v);` | `regular expression: outside the portable core` | (correct) |
| `return /[0-9a-f]/u.test(v);` | `malformed number 9a` | the regular-expression refusal |
| `const n = /[\s'"]/u.test(v);` | `unterminated string literal` | the regular-expression refusal |
| `` return `\u{1F}${a}`; `` | `unsupported template escape \u` | translated (`\u{…}` is valid in templates) |
| `` return `\"${a}`; `` | `unsupported template escape \"` | translated (identity escape) |
| `return '\/';` | `unsupported string escape \/` | translated (identity escape) |
| `@param {number} [b]` or `@param {number} [b=1]` | `@param needs a type and a name` | the optional parameter, which the core already supports as a default parameter |

A regular-expression literal is refused correctly only when its body happens to lex as code. When the body holds a digit followed by letters, or a quote, the lexer reads it as code and reports an error about something that is not in the source.

### Consumer need

link-assistant/formal-ai self-translates 123 JavaScript modules ([link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188)). 39 of its carried items are refused with syntax errors (census in `data/meta/js-rust-translation.lino`): 18 `malformed number`, 3 `unterminated string literal` and 3 `unsupported string escape` (`\]`, `\b`, `\p`), which are the shapes a regular-expression body takes when it is lexed as code (each of the four sampled was a regex body); 10 `@param needs a type and a name`, for optional `[name]` tags; and 5 template-literal errors (4 escapes, 1 unterminated). A syntax error says the source is malformed, which it is not. So the refusal census (`data/meta/js-rust-translation.lino`) cannot tell these items apart from real gaps, and the optional-parameter and escape items could translate today.

### Request

1. Lex a regular-expression literal as one token wherever an expression can start (after `(`, `,`, `=`, `:`, `[`, `!`, `&`, `|`, `?`, `{`, `}`, `;`, `return`, `typeof` and the other operators), as the ECMAScript goal-symbol rule says. Then refuse it with the existing `regular expression` obligation.
2. Accept every escape ECMAScript allows in string and template literals: identity escapes such as `\"`, `\'` and `\/`, and `\u{…}` in templates.
3. Read `@param {T} [name]` and `@param {T} [name=default]` as the type of a parameter that has a default.

Acceptance: the six rows above give the expected result, and a corpus case pins each one.

---
Found while wiring the js -> rust leg in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188) (`docs/case-studies/pull-request-1188/js-rust-translation.md`).
