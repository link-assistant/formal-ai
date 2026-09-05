# links-notation (Rust): parse errors carry no position, unlike the JS port

Filed against `link-foundation/links-notation`.

## What happens

The Rust parser reports a failure by `Debug`-printing the underlying `nom`
error. There is no line, no column, and no statement of what was expected; the
payload is the entire unconsumed remainder of the input.

```rust
use links_notation::parse_lino; // 0.16.1

let error = parse_lino("# ok line\n# break: two\nci_gate x\n  stage rust").unwrap_err();
println!("{error}");
```

```text
Syntax error: Error(Error { input: "# break: two\nci_gate x\n  stage rust", code: Eof })
```

The JavaScript implementation of the same version, on the same input, reports
both:

```js
const { Parser } = require('links-notation') // 0.16.1
try { new Parser().parse('# ok line\n# break: two\n') } catch (e) {
  console.log(e.message)          // Parse error: Expected "(", [ \t], [\r\n], or [^ \t\n\r(:)] but ":" found.
  console.log(e.location.start)   // { offset: 17, line: 2, column: 8 }
}
```

## Why it matters

On a real data file the Rust message is unusable. In
[link-assistant/formal-ai](https://github.com/link-assistant/formal-ai) the CI
gate that parses every checked-in `.lino` file failed with a single-line
message containing several hundred characters of quoted file content and no
indication of which line to look at; the files reach 1500 lines, so the tail
can be most of the file. The consumer had to bisect the file line by line to
find the offending character. `code: Verify` and `code: Eof` are `nom` internals
that mean nothing at the call site, and which of the two appears depends on
where in the file the defect sits rather than on what the defect is.

## Workaround

Re-parse each line on its own and report the first one that fails. It finds the
common cases and misses multi-line quoted strings:

```rust
fn first_unparseable_line(content: &str) -> Option<(usize, String)> {
    content.lines().enumerate().find(|(_, line)| {
        let trimmed = line.trim();
        !trimmed.is_empty() && parse_lino(trimmed).is_err()
    }).map(|(index, line)| (index + 1, line.trim().to_string()))
}
```

## Suggested fix

1. Track the offset of the remaining input against the original input — the
   difference between the two pointers, or `nom_locate::LocatedSpan` — and
   convert it to a 1-based line and column in the `Display` implementation of
   the error type. The information is already present at the point the error is
   built; only the arithmetic is missing.
2. Render `Display` as `line:column: expected <...>, found <...>` and keep the
   `nom` code behind `Debug`. Quote one line of context rather than the whole
   remainder — the current behaviour makes the message grow with the size of
   the file, which is exactly backwards.
3. Add a test asserting the reported line for a defect on a known line, so the
   two ports can be held to the same contract (issue #138 asks for a shared
   test list across implementations, and the JS behaviour above is the one to
   match).
