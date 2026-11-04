# Main source-provenance and grammar-scope repair

PR #1543 merged as aa21c0871ee8884f940231fc3d73d670bd4072ff after its final fast checks passed. Main's larger suites exposed additional independent requirements.

| Observed failure | Root cause and required repair |
| --- | --- |
| Debugger client and server stage-source assertions | A new function-pointer wrapper hides the ordinary arithmetic call from the bounded source graph. Compose the quantity attempt and existing arithmetic fallback directly; retain the original emitter assertions. |
| Page formalization and corpus quality ratchets | Quantity parser cues compete in the global semantic index. Declare generic grammar scope, preserve role-based recognizer access, and exclude scoped grammar from ordinary meaning selection in both backends. Retain the old quality floors. |
| Native semantic definition closure | The new budget and capacity definitions reference a limit parent which the authored native graph does not contain. Supply genuinely grounded semantics; cache-aware lexical closure alone does not prove authored-graph closure. |

Captured source: main aa21c0871ee8884f940231fc3d73d670bd4072ff. Layered run [41821551028](https://github.com/link-assistant/formal-ai/actions/runs/41821551028) failed JS groups 1–4; groups 5–8 and all source-network serialization groups passed. The dependent TS/dogfood/parity stages were skipped, not executed. Native producer job [126546880457](https://github.com/link-assistant/formal-ai/actions/runs/41821551480/job/126546880457) succeeded; successful compilation is separate from the failed source/quality gates and does not qualify a release.

The repair keeps expected answers, corpus floors and meaningful source assertions. The small offline debugger regression joins the fast profile; full integration, corpus and native gates remain available on main. New source must obtain its own CI results. No local Rust, product WASM or Docker build is permitted. Full issue #1483 acceptance remains open.
