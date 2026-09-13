---
bump: patch
---

### Fixed

- Docker Hub publishing now runs. `DOCKERHUB_IMAGE` and `DOCKERHUB_USERNAME`
  were read from repository configuration that was never set, so every release
  since the feature was added disabled Docker Hub and still reported success --
  `hub.docker.com/r/linkassistant/formal-ai` had never received a push. Both
  now default in `release.yml` (`konard/formal-ai`, `konard`), the way hive-mind
  names its image, and `DOCKERHUB_TOKEN` becomes what opts in: a fork without
  the secret skips Docker Hub and still gets a green release (#1131).
- A whole-file rewrite is no longer mistaken for a member insertion. A request
  to rewrite a shell script was routed into the structural-edit path because its
  prose contained the bare word `set` -- from `set -euo pipefail` -- and quoted
  two values while explaining the change. Both were spliced into the script's
  nearest bracket, producing a shell condition that parses and is always true.
  Growing a member list now requires a verb that asks for it (#1131).
