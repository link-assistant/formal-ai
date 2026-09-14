# Upstream report — `meta-language`: `LinkNetwork::parse` is quadratic in input size

Filed against <https://github.com/link-foundation/meta-language>.
Reason it belongs upstream: the defect is entirely inside
`src/tree_sitter_adapter.rs` of the published crate; nothing in this repository
can change the algorithm, only avoid calling it.

Status: filed as
<https://github.com/link-foundation/meta-language/issues/193> on 2026-08-16.

Verified present in **0.54.0** (the version this repository pins) and in
**0.58.1** (the latest published version at the time of writing) — the function
is byte-for-byte identical in both.

---

## Body as filed

### `LinkNetwork::parse` is O(nodes × bytes): `point_at_byte` rescans the whole source for every span

**Version:** reproduced on `meta-language` 0.54.0 and 0.58.1 (latest).

## What happens

Every node conversion resolves its byte offsets to a `(row, column)` `Point`, and
that resolution walks the source from byte 0 each time. `src/tree_sitter_adapter.rs`
(0.58.1, line 374):

```rust
fn point_at_byte(text: &str, byte: usize) -> Point {
    let mut row = 0;
    let mut line_start = 0;
    for (index, value) in text.bytes().enumerate().take(byte) {
        if value == b'\n' {
            row += 1;
            line_start = index + 1;
        }
    }
    Point::new(row, byte - line_start)
}
```

It is called twice per span (line 366):

```rust
fn span_for_range(text: &str, start: usize, end: usize, offset: SpanOffset) -> SourceSpan {
    SourceSpan::new(
        ByteRange::new(offset.byte + start, offset.byte + end),
        offset.point(point_at_byte(text, start)),
        offset.point(point_at_byte(text, end)),
    )
}
```

and `span_for_range` is reached from `span_for_node`, `insert_leaf_token` and
`insert_gap_token` — i.e. once per node and once per token, for every node in the
tree. Each call is `O(byte)`, so a full parse is `O(nodes × bytes)`: quadratic in
file size, since node count grows linearly with bytes.

## Reproduction

A standalone crate with `meta-language` as its only dependency. It parses the
same synthetic Rust module at growing sizes and prints nanoseconds per byte — a
linear parser holds that column flat; a quadratic one doubles it every time the
input doubles.

```toml
[package]
name = "meta-language-quadratic-parse"
version = "0.1.0"
edition = "2021"

[dependencies]
meta-language = "0.58.1"
```

```rust
use std::time::Instant;

use meta_language::{LinkNetwork, ParseConfiguration};

fn unit(index: usize) -> String {
    format!(
        "/// Doc comment for item {index}.\n\
         pub fn item_{index}(input: &str) -> usize {{\n\
         \x20   let trimmed = input.trim();\n\
         \x20   if trimmed.is_empty() {{\n\
         \x20       return {index};\n\
         \x20   }}\n\
         \x20   trimmed.len() + {index}\n\
         }}\n\n"
    )
}

fn main() {
    println!("{:>6}  {:>9}  {:>10}  {:>14}", "units", "bytes", "parse ms", "ns per byte");
    for units in [64_usize, 128, 256, 512, 1024] {
        let source: String = (0..units).map(unit).collect();
        let started = Instant::now();
        let network = LinkNetwork::parse(&source, "rust", ParseConfiguration::default());
        let elapsed = started.elapsed();
        assert_eq!(network.reconstruct_text(), source);
        let bytes = source.len();
        let per_byte = elapsed.as_nanos() as f64 / bytes as f64;
        println!("{units:>6}  {bytes:>9}  {:>10}  {per_byte:>14.0}", elapsed.as_millis());
    }
}
```

Measured with `cargo run --release` (the *favourable* case — optimisations on):

| units | bytes | parse ms | ns per byte |
| ---: | ---: | ---: | ---: |
| 64 | 11 416 | 86 | 7 615 |
| 128 | 22 984 | 312 | 13 593 |
| 256 | 46 408 | 1 115 | 24 032 |
| 512 | 93 256 | 4 262 | 45 710 |
| 1 024 | 187 048 | 18 168 | 97 134 |

Doubling the input multiplies the total time by ~3.6–3.8 and the per-byte cost
by ~1.8–1.9 at every step — the signature of an `O(n²)` term dominating. A
linear parser would hold the last column flat.

The same measurement on the `dev` profile — the profile a test suite runs
under, and the one that produced the CI failure below — scales identically and
is roughly 25× slower in absolute terms:

| units | bytes | parse ms | ns per byte |
| ---: | ---: | ---: | ---: |
| 64 | 11 416 | 2 167 | 189 902 |
| 128 | 22 984 | 8 263 | 359 539 |
| 256 | 46 408 | 33 223 | 715 893 |
| 512 | 93 256 | 127 817 | 1 370 607 |
| 1 024 | 187 048 | 503 302 | 2 690 767 |

## Attribution

Sampled with `gdb -p <pid> -batch -ex "bt 80"` against a process parsing a
39 KB Rust module. **12 of 12 samples** were inside `point_at_byte`, called from
`span_for_range` → `convert_node` → `network_from_tree` → `LinkNetwork::parse`.

## Impact

A 39 KB Rust module takes **over ten seconds** to parse on the `dev` profile.
In our case (`link-assistant/formal-ai`, issue
[#1017](https://github.com/link-assistant/formal-ai/issues/1017)) that parse sat
behind a lookup on the request path, so a server's *first* HTTP response took
~13 s and two macOS CI slices hit the integration harness's 30-second timeout.
Any consumer that parses a real source file inside a request, a language server,
or a watch loop will meet the same wall.

## Suggested fix

Compute the line-start table **once per parse** and binary-search it, which turns
each lookup into `O(log lines)` and the parse into `O(nodes × log lines)`:

```rust
/// Byte offset of the first character of each line, ascending. `line_starts[0]`
/// is always 0, so the table is never empty and the search below always hits.
fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    starts.extend(
        text.bytes()
            .enumerate()
            .filter(|(_, value)| *value == b'\n')
            .map(|(index, _)| index + 1),
    );
    starts
}

fn point_at_byte(line_starts: &[usize], byte: usize) -> Point {
    // The index of the last line start at or before `byte`.
    let row = line_starts.partition_point(|start| *start <= byte) - 1;
    Point::new(row, byte - line_starts[row])
}
```

The table belongs next to `text` in `ConvertContext`, built once in
`network_from_tree` (and once per embedded/injected region, alongside the
existing `SpanOffset`), so no call site needs to own it. `line_starts` is
`O(bytes)` once, versus `O(bytes)` per node today.

Two smaller wins are available on top:

* `span_for_node` can take `node.start_position()` / `node.end_position()`
  straight from tree-sitter, which already tracks points during parsing — no
  lookup at all for the node case.
* `span_for_range(text, start, end, …)` resolves `end` from scratch after
  resolving `start`; with the table that no longer matters, but scanning
  forward from `start` would also have removed the worst of it.

## Workaround for consumers

Until this lands, avoid parsing on any latency-sensitive path, and memoise
parses of build-time-constant sources. That is what we did:
[`link-assistant/formal-ai#1018`](https://github.com/link-assistant/formal-ai/pull/1018)
proves a cache lookup will miss *before* building the artifact that parses, and
memoises the one round-trip whose input is a compile-time constant. It removed
the timeout (first response ~13 s → ~0.3 s) without touching the algorithm —
which is why the algorithmic fix still belongs here.

---

## Local artifacts

| Path | Contents |
| --- | --- |
| `experiments/issue-1017-meta-language-quadratic/` | The standalone reproducer above, dependency-free apart from `meta-language`. Produced the `--release` table. |
| `examples/issue_1017_parse_scaling.rs` | The same measurement through this repository's `ast_census`, so the numbers can be re-derived without a second checkout. Produced the `dev` table. |
| `examples/issue_1017_cold_request_profile.rs` | The before/after request-latency measurement that attributes the CI timeout to this parse. |
