# links-notation: no comment syntax, so `#` prose lines parse by accident

Filed against `link-foundation/links-notation`.

## What happens

Links Notation has no comment form. A line starting with `#` is parsed as an
ordinary link whose first reference is `#`, so it survives only as long as the
prose inside it happens to avoid the notation's own delimiters. `:` is one of
them, and it is the most common character in English technical prose after the
comma.

```rust
use links_notation::parse_lino; // 0.16.1

parse_lino("# a b").unwrap();   // OK: a link (# a b)
parse_lino("# a: b").unwrap();  // panics: Syntax error ... code: Verify
```

Same grammar, same result in the JavaScript implementation (`links-notation`
0.16.1 on npm), so this is the notation rather than one port:

```js
const { Parser } = require('links-notation')
new Parser().parse('# a b')    // OK
new Parser().parse('# a: b')   // Parse error: Expected "(", [ \t], [\r\n], or [^ \t\n\r(:)] but ":" found.
```

A colon inside a backtick span is fine, because the span is one reference:

```
# Issue #1047 measured `cargo nextest --partition slice:N/D`.   <- parses
# What a commit can break: two of the tests parse the tree.     <- does not
```

## Why it matters

Consumers write prose in `.lino` files today. In
[link-assistant/formal-ai](https://github.com/link-assistant/formal-ai) 52
checked-in `.lino` files carry `#` paragraphs explaining what the data is for,
and a CI gate parses every one of them with `parse_lino`. A pull request added
a gate file whose comment read

```
# holds is the part a commit *can* break: two of the tests parse the
```

and the full test suite went red on an otherwise correct data file. The prose
is not part of the data, so the failure has no relationship to what the file
means — the author has to learn that `:` is structural inside what looks like
a comment.

## Workaround

Rewrite the prose: replace the colon with ` -- `, or wrap the colon-bearing
token in backticks so the span parses as a single reference. Both keep the
line readable, and neither is discoverable from the error.

## Suggested fix

1. Add a line-comment production to the grammar — a `#` (or `//`) at the start
   of a line, and after horizontal whitespace, consumes to the end of line and
   produces nothing. This matches how consumers already use `#` and costs one
   alternative in the whitespace/line parser. In the Rust parser that is a
   branch alongside the existing line handling in `src/parser.rs`; the JS
   grammar needs the same rule so the ports stay in step (issue #138 asks that
   every implementation share a test list — a comment rule belongs in it).
2. If comments are deliberately out of scope, say so in `README.md` and in the
   EBNF from issue #144, and consider rejecting a leading `#` outright rather
   than accepting it as a reference. Silent acceptance is what makes the
   failure arrive later, in a different file, as a parse error the writer
   cannot connect to their edit.

Either resolution is fine for the consumer; the current state — accepted
sometimes, rejected otherwise, documented nowhere — is the problem.
