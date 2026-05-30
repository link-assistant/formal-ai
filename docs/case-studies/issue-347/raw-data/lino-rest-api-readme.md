# lino-rest-api — capture note

> **Not a verbatim mirror.** Unlike the sibling `*-readme.md` captures, this one
> is paraphrased. The upstream README describes the project with a
> deferred-status label that this repository's text policy (issue #103) forbids
> anywhere in-tree — even inside quoted third-party material, because the
> `repository_text_avoids_deferred_labels_requested_by_issue_103` test scans the
> whole tree. So this file records the citable facts and links to the original
> rather than mirroring it byte-for-byte.
>
> Verbatim source: <https://github.com/link-foundation/lino-rest-api#readme>

**Repository:** [link-foundation/lino-rest-api](https://github.com/link-foundation/lino-rest-api)
**Self-description (repo metadata):** "Yet another REST API framework, using Links Notation instead of Json"
**Language:** JavaScript · **Stars at capture:** 0 · **License:** Unlicense

## What it is

REST API frameworks that serialize with
[Links Notation](https://github.com/link-foundation/links-notation) (LINO)
instead of JSON. The repo ships two early-stage reference implementations:

- **JavaScript/Bun** — an Express.js-based `createLinoApp()`.
- **Python** — a FastAPI-based `LinoAPI()`.

Both use `text/lino` as the request/response content type. Related upstream
projects it points at: `links-notation` (core library),
`link-notation-objects-codec` (object ↔ LINO encoding), and `test-anywhere`.

## Why it matters for issue #347 (R6)

`lino-rest-api` is the upstream prior art for the deferred R6 "Links-native
REST" interface. Its object-codec dependency,
[link-notation-objects-codec](https://github.com/link-foundation/link-notation-objects-codec),
is what [`../ROADMAP.md`](../ROADMAP.md) §D3 proposes reusing rather than
hand-rolling a Links Notation ↔ objects mapping. The project is an early-stage
upstream experiment, which is part of why R6 is sequenced as deferred work.
