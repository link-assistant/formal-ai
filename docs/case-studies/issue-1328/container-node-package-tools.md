# Missing npm in the published full runtime image

The post-release Hive workflow for v0.355.3 failed before executing the original coding tasks. [Image preparation job 120226539250](https://github.com/link-assistant/formal-ai/actions/runs/39895420428/job/120226539250) ran the pinned Hive Dockerfile against the published Formal AI runtime and exited 127 at:

```sh
npm --prefix /opt/hive-e2e ci --omit=dev --ignore-scripts
```

The full log has SHA-256 `782caeedd47d954e53e50ad8d5918e7d9efa760a87d8b64ce8d552b29b9972b5`. The failure was `npm: command not found`. The subsequent missing lifecycle artifact and final missing-qualified-client assertion were consequences of image preparation stopping before lifecycle measurement and client execution. Their fail-closed acceptance checks remain in place.

The full Dockerfile copied the Node executable from its pinned official Node stage, while omitting Node's bundled npm directory and CLI launchers. Existing Node and agent version probes passed without exercising npm. The derived Hive Dockerfile needed npm to install its locked production dependencies as `box`.

The fix copies npm from the same official Node stage and installs its npm/npx launchers. The [official Node image](https://github.com/nodejs/docker-node/blob/main/26/bookworm-slim/Dockerfile) also smoke-tests npm alongside Node. Both commands must pass version probes as `box` during the image build and remain required by the full runtime smoke check. Missing package-manager files now fail before publication.

The regression retains the exact failing command and diagnostic from the CI log. A temporary filesystem first reproduces exit 127 with Node available and npm absent, then applies the maintained donor-copy/launcher instructions and runs the captured command with a real bundled npm. The project has no dependencies, execution is offline, and an install script that would fail proves `--ignore-scripts` remains effective. The fixture removes its temporary project, package payload, and cache afterward. Rust and Docker run only in CI.

These changes are manually authored under the release-blocker priority; Formal AI receives no authorship credit. Delivery acceptance for the original three tasks and issue #1364 remains pending the actual corrected-image workflow and released-runtime evidence.
