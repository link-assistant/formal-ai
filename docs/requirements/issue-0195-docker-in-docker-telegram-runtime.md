## Issue #195 Docker-in-Docker Telegram Runtime

Issue [#195](https://github.com/link-assistant/formal-ai/issues/195)
narrows the supported container story: the Telegram bot Docker image must be
based on Link Foundation's Box Docker-in-Docker image, coding-task commands
must go through `start-command` with Docker isolation, and the root README must
show the real container start path.

| ID | Requirement | Status |
| --- | --- | --- |
| R220 | The root Docker image must use Link Foundation Box Docker-in-Docker as its only supported runtime image. | Implemented by the final stage of `Dockerfile` using `konard/box-dind:2.1.1`, preserving `/usr/local/bin/dind-entrypoint.sh`, and rejecting the previous Debian HTTP-server runtime in `tests/unit/docker_runtime.rs`. |
| R221 | The Docker image must start the Telegram bot by default, with HTTP webhook mode only as an explicit override. | Implemented by `CMD ["formal-ai", "telegram", "--mode", "polling"]`; README documents the webhook override command. |
| R222 | Coding-task commands in the container must be launched through `start-command` with Docker isolation so command output and metadata are tracked. | Implemented by installing the `start-command` package, exposing the `$` CLI, setting `FORMAL_AI_START_RUNNER` to `$ --isolated docker --auto-remove-docker-container --`, and verifying it in `scripts/verify-docker-runtime.sh`. The issue's `--isolation docker` wording maps to the current Start CLI flag `--isolated docker`. |
| R223 | The seed environment directory must describe the Docker-in-Docker Telegram runtime rather than the stale HTTP-server container. | Implemented in `data/seed/environments.lino` and pinned by `docker_microservice_seed_declares_dind_start_command_contract`. |
| R224 | Docker operation instructions must live in the root README and include the privilege/runtime, Telegram token, storage volume, verification command, and socket-safety warning. | Implemented in `README.md` under "Docker-in-Docker Telegram bot image" and pinned by `tests/unit/docs_requirements.rs`. |
| R225 | Issue research, upstream facts, and repro logs must be preserved under `docs/case-studies/issue-195`. | Implemented with issue/PR snapshots, Box/Start/Hive Mind source captures, local tool availability, and the failing pre-fix Docker runtime test log. |
