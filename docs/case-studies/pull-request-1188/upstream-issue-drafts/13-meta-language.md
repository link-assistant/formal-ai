<!-- repo: link-foundation/meta-language -->
<!-- title: selfTranslate parses each module twice at about 5 KB/s; parse once and speed up the lossless JavaScript parse -->

### Problem

`selfTranslate` (main at `679a3b3c`, `js/src/self-translation.js`) parses the whole source twice with the lossless parser. `LinkNetwork.parse(text, from).reconstructText()` only checks that the links reproduce the source. Then `topLevelItems` calls `parseProgrammingLanguage(text, language)` again to cut the items. Each parse runs at about 5 KB/s, linear in the size of the source. The portable-core work per item (`parseJavaScript`, `checkProgram`, the emitters) takes milliseconds.

Measured on a module of N identical JSDoc-typed functions (node 20, one thread, on a busy 12-core laptop):

| items | bytes | `LinkNetwork.parse` | `parseProgrammingLanguage` | `selfTranslate` total |
| ---: | ---: | ---: | ---: | ---: |
| 25 | 3,405 | 660 ms | 566 ms | 1,202 ms |
| 50 | 6,830 | 1,255 ms | 1,213 ms | 2,618 ms |
| 100 | 13,680 | 2,606 ms | 2,606 ms | 5,160 ms |
| 200 | 27,580 | 5,037 ms | 5,398 ms | 10,432 ms |

A real 33 KB module (`algorithm_discovery.mjs` in link-assistant/formal-ai) takes 11.5 s in `LinkNetwork.parse` and 9.8 s in `parseProgrammingLanguage`. `docs/self-translation.md` says the report over meta-language's own `js/src` "takes over an hour on one runner" and needs eight CI shards. Almost all of that time is these two parses.

### Consumer need

link-assistant/formal-ai re-translates its 123 JavaScript modules (814 KB) on every change, to hold the committed Rust to the translator ([link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188), the `js-rust` job of `.github/workflows/layered-ci.yml`). That takes about 7.5 CPU-minutes, so the job needs four worker threads to stay within its budget. A developer re-translating one edited module waits 10-35 s for it. At tokenizer speed the job would take seconds and could run inside the ordinary JavaScript test tier.

### Request

1. Parse once: build the link network from the same tree `topLevelItems` uses (or cut the items from the network), so the reproduction check costs no second parse.
2. Profile the lossless JavaScript parse. 5 KB/s is about three orders of magnitude slower than a JavaScript tokenizer. Issue #193 was the same kind of hot spot on the Rust side (`point_at_byte` rescanning the source for every span).
3. Optionally, let `selfTranslate` take a pre-parsed network, so a consumer that already parsed the module can reuse it.

Acceptance: `selfTranslate` on the 200-item module above is at least ten times faster, and the output is byte-identical.

---
Found while wiring the js -> rust leg in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188) (`docs/case-studies/pull-request-1188/js-rust-translation.md`).
