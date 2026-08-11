# Issue 990: production command-stream adoption

Issue [#990](https://github.com/link-assistant/formal-ai/issues/990) completes the
`command-stream` half of the command-execution requirement left open by issue
[#546](https://github.com/link-assistant/formal-ai/issues/546) and row 30 of the
[#710](https://github.com/link-assistant/formal-ai/issues/710) audit.

## Boundary inventory

| Production boundary | Result |
| --- | --- |
| Electron host shell and Docker-selected tools | Shared `desktop/lib/command-runner.cjs`, backed by `command-stream@0.18.0`, retains host/Docker selection |
| Electron local API server lifecycle | The shared adapter supplies streaming output and cancellation handles |
| Desktop Docker services, dreaming commands, in-process agent commands, and POSIX VS Code CLI installation | Routed through the shared adapter |
| Cross-platform VS Code host | Temporary existing `child_process` boundary because the only public entry point eagerly imports native PTY/rendering dependencies that cannot be bundled into one portable VSIX; upstream [command-stream#192](https://github.com/link-foundation/command-stream/issues/192) |
| Rust orchestration on POSIX | `command-stream@0.15.0` `StreamingRunner`, with the crate's own quoting helper, streamed stdout/stderr, and process-group timeout cancellation |
| Synchronous CommonJS availability/version probes | Temporary `spawnSync` workaround; upstream [command-stream#189](https://github.com/link-foundation/command-stream/issues/189) |
| Rust orchestration on Windows | Temporary exact-argv `std::process::Command` workaround; upstream [command-stream#190](https://github.com/link-foundation/command-stream/issues/190) |
| Windows-only `code.cmd` installation | Temporary Node `spawn(..., { shell: true })` workaround; upstream [command-stream#191](https://github.com/link-foundation/command-stream/issues/191) |

The Docker-in-Docker service lifecycle remains owned by the already adopted
`start-command` component; this change does not replace that boundary.

## Reproduction and verification

Before the fix, neither Node host manifest nor `Cargo.toml` directly depended
on the published component, and production hosts implemented independent
`child_process.spawn` capture loops. The regression test was first run against
that state and failed because the production adapter did not exist.

`desktop/scripts/command-runner.test.mjs` now executes the real Electron adapter and
pins streamed stdout and stderr, a nonzero exit status, `AbortSignal`
cancellation, and both host and Docker routing. Existing orchestration tests
exercise the Rust production boundary's output, exact argument, nonzero, and
timeout behavior. The test also pins the component's eager native-terminal
import and the linked VS Code exclusion, while the unchanged VS Code packaging
check proves that the portable extension still builds.

This is a process-boundary change with no visual output, so screenshots are not
applicable.
