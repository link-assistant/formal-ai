# Browser execution experiment (#670)

The application already contains an opt-in Pyodide adapter and an honest
`browser_runtime:pyodide` execution note. This experiment adds an isolated
measurement page using that exact adapter, with startup/execution timing,
assertion outcome, explicit cancellation and a downloadable measurement record.
Serve the repository root over HTTP and open `experiments/webvm/index.html`.
No runtime is fetched before pressing Download Python. Every run has a 30 s
worker deadline; loading has 120 s. Termination discards its Python memory.

| Candidate | Primary reference | Assessment |
| --- | --- | --- |
| Pyodide | [Official deployment guide](https://pyodide.org/en/stable/usage/downloading-and-deploying.html) | Existing app adapter; Python assertions directly runnable; proposed first target. |
| CheerpX/WebVM | [WebVM](https://webvm.io/) | Full x86 VM adds a guest image and networking surface; defer until a measured workload requires them. |
| ruby.wasm | [CRuby WASM ports](https://github.com/ruby/ruby.wasm) | Language-specific alternative; does not execute the Python benchmark candidates. |

Recommendation: **no further promotion pending measurement**. Pyodide 314.0.7
was confirmed in the official deployment guide on 2026-09-30. The existing
20 MiB transfer estimate is not a measurement; the full published distribution
is 200+ MB and differs from the core runtime assets loaded by this page.

Measurements: not run under the session's no-build/no-test instruction. Startup,
actual transferred bytes, memory and repeated-run determinism remain unknown.
Save records from cold and warm loads on a mid-range laptop, repeat candidates
five times, and compare stdout/assertion results before recording a go decision.
The existing app runtime adapter has no enforced execution deadline; the bounded
experiment should replace that path only after integration review.

Without WebAssembly, this experiment reports failure and does not claim code
execution. The main app retains its existing diagnostic fallback policy.
Browser execution grants no host-level process or filesystem access. HumanEval
and MBPP generated-candidate integration and browser e2e proof remain outstanding.
