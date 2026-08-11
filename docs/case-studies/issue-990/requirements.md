# Requirements trace

| ID | Requirement | Evidence |
| --- | --- | --- |
| R1 | Depend directly on the published component | `Cargo.toml` and `desktop/package.json` pin published releases |
| R2 | Share Desktop and VS Code Node-host execution or map exclusions | Electron uses `desktop/lib/command-runner.cjs`; command-stream#192 maps the tested native-dependency packaging limitation for VS Code |
| R3 | Integrate supported Rust surfaces | POSIX orchestration uses `StreamingRunner`; the unsafe Windows string-only surface is mapped to command-stream#190 |
| R4 | Preserve stdout/stderr streaming and exit status | `production command adapter streams stdout/stderr and preserves a non-zero exit` |
| R5 | Preserve cancellation | `production command adapter cancels the real child process` plus existing orchestration timeout tests |
| R6 | Preserve host/Docker selection | `the same production adapter executes host and Docker-selected commands` |
| R7 | File confirmed limitations upstream | command-stream#189, command-stream#190, command-stream#191, and command-stream#192 |
| R8 | Keep `start-command` Docker lifecycle | The service-control boundary is unchanged |
