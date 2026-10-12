<!-- repo: link-foundation/meta-language -->
<!-- title: Self-translation items should keep the portable-core diagnostic, not only its kind -->

### Problem

When the portable core refuses an item, `translateGroup` (`js/src/self-translation.js`, main at `679a3b3c`) records only the error's kind. The item list says `carried` with reason `unsupported`, `type` or `syntax`, and the output marker says `// meta-language:carried JavaScript export_statement (unsupported)`. The `TranslationError` it caught already names the construct, the reason and the span (`arrow function: functions as values are outside the portable core at 82..95`), and that is dropped.

### Consumer need

link-assistant/formal-ai ratchets its js -> rust translation and keeps a census of what keeps items out, so it can widen its JavaScript toward the portable core where that pays most ([link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188), `data/meta/js-rust-translation.lino`). To get the construct, it runs `translateProgram` a second time on every carried item's source lines and reads `diagnostic.message`: 1,926 extra translations per run. Two runs of the same item can only be assumed, not shown, to fail for the same reason. The upstream per-module report (`generate-self-translation-report.mjs`) has the same limit: it counts items by status, not by construct.

### Request

1. Keep the diagnostic in the item record: `{ status: 'carried', reason: 'unsupported', construct: 'arrow function', message: '…', span: { start, end } }`, with the span relative to the source.
2. Put the construct in the carried marker, e.g. `// meta-language:carried JavaScript export_statement (unsupported: arrow function)`. A round trip still restores the item, because the marker is not part of the hashed code.
3. Count carried items by construct in the self-translation report, so the most frequent missing constructs come out of the report, not out of each consumer's reconstruction.

---
Found while wiring the js -> rust leg in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188) (`docs/case-studies/pull-request-1188/js-rust-translation.md`).
