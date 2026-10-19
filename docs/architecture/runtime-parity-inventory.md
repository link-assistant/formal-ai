# Runtime parity: maintained capabilities and evidence limits

The standing September 24 and October 6–8 doctrines require full JavaScript,
TypeScript and Rust client/backend capabilities, with JavaScript first and
translation through the meta language. Historical Rust-only wording in open
issues #934, #951 and #953 does not establish the current three-root contract.
This inventory covers the capability families, not just matching symbol names.
It does not claim full semantic parity or close those issues.

| Capability family | Maintained surfaces | Actual evidence and remaining gap |
| --- | --- | --- |
| HTTP, authentication, OpenAI/Anthropic/Gemini protocols | `js/server/`, `ts/server/`, `rust/src/server*`, protocol modules | `scripts/check-server-parity.mjs` compares the finite `server-parity/requests.lino` corpus and rejects shape differences. A zero content-debt ceiling covers that corpus only, not every route, session lifecycle, concurrent request or host failure. |
| Deterministic agent planning, observation and file/tool effects | `js/agentic/`, `ts/agentic/`, `rust/src/agentic_coding/` | Real shared/read/write receipt tests and hosted client runners exist. Dynamic tools, external clients and unobserved native effects are not certified by function-name correspondence. Existing unsupported issue tasks remain named Gaps. |
| Solver, knowledge, natural-language and catalog behavior | `js/worker/`, `js/agentic/crate/`, `ts/`, Rust solver/catalog modules | Handler-registry and language-debt gates are structural ratchets. They do not prove every input or language has equivalent answers. Committed documentation captures and explicit execution receipts must retain their distinct authority. |
| Meta language, program CST and rendering | `scripts/self-translation/`, `rust/src/meta_translate.rs`, `rust/src/coding/cst.rs` | #1167's current source already exposes `try_render_cst_source` and `try_compose_and_validate`; its issue body's original missing-function observations are historical. Rust CST/grammar behavior still needs its actual hosted tests. The portable-pure fragment is narrower than the whole backend. |
| Permission decisions and desktop/native tools | `desktop/lib/`, agentic capability routers, Rust computer-use and host modules | #953 remains relevant: identical quoting does not prove path confinement, grants, symlinks, Windows/UNC handling or IPC authorization. Generated string contracts grant no execution permission. |
| UI and browser worker integration | `js/app/`, `js/worker/`, `ts/`, WASM worker | #951's function/line counts describe an older layout; moving files alone is not a semantic migration. #934 hard-fail/provenance tests must remain intact. Browser-only DOM and host adapters need real runtime observations. |
| CLI, process, filesystem, caches, resources and platform APIs | Node host, Rust CLI/transports, desktop supervisors | JS APIs and Rust OS implementations have different host contracts. Hosted native/platform checks remain necessary; pure translation cannot replace OS evidence. Cold/warm cache or download metadata must not be relabeled execution or publication proof. |
| Build, packaging, release and publication consumers | `scripts/`, workflow manifests, native release helpers | JS twins and authenticated source/download receipts cover specific operations. A generated TypeScript file is not a second independently observed publication or runtime. |

## Translation inventory and this slice

The pre-change baseline `data/meta/js-rust-translation.lino` recorded 146 modules,
293 upstream-translated items, 33 workaround items, and 2320 carried items.
These are source census values, not capability coverage or live native delivery.
Only one recorded module is fully translated by that projection mechanism.
The projections mostly live under test fixtures; counting them as replacements
for every production Rust implementation would be wrong. Imports, host APIs,
objects, async/effects and unsupported constructs remain material gaps.

JavaScript-to-TypeScript currently uses `scripts/translate-es.mjs` and the ES meta
token/layout representation. The existing `portable-pure-v1` JS/Rust profile
explicitly refuses string concatenation. The new sibling `portable-string-v1`
uses that same Links IR serialization with a checked string-only subset:
parameters, literals, concatenation, strict equality conditionals and literal
nonempty replacement. Replacement expansion tokens, dynamic replacements,
unknown calls, side effects, coercion, unsupported IR and non-scalar literal
text are refused by name. This does not broaden or weaken the older profile.

`js/agentic/portable/string_contracts.mjs` is the authoritative executable
source for POSIX single-word quoting and already-trimmed failure-invitation
joining. `scripts/generate-portable-contracts.mjs` formalizes it to
`data/meta/portable-string-contracts.lino`, emits the actual production Rust
module, and uses the existing ES translator for TypeScript. The old public
adapters delegate, preserving caller APIs. Trimming, seed lookup, permission
checks and tool execution retain their existing owners.

The shared `rust/tests/fixtures/portable-string-contracts.lino` corpus executes
in JavaScript, the TypeScript projection and the registered hosted Rust suite.
It includes empty text, apostrophes, shell metacharacters, multiline text,
Unicode, backslashes, NUL and invitation branches. NUL preservation is a string
contract; it is not a claim that a shell accepts a NUL-bearing operand.
`check-portable-string-contracts` rejects generated drift. Unsupported syntax
and malformed IR have negative controls. Rust compilation and native consumer
acceptance are hosted-only and remain pending until those jobs actually pass.
