# Issue 438 Online Research

Captured: 2026-06-14.

## Sources Checked

- GitHub Container Registry: https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry
- Docker Compose variable interpolation: https://docs.docker.com/compose/how-tos/environment-variables/variable-interpolation/
- Telegram Bot API: https://core.telegram.org/bots/api
- Docker Build Push Action: https://github.com/docker/build-push-action
- Docker Metadata Action: https://github.com/docker/metadata-action
- Docker Login Action: https://github.com/docker/login-action

## Facts Used

- GitHub Container Registry can host OCI container images under `ghcr.io` and
  GitHub Actions can authenticate with `GITHUB_TOKEN` when the workflow has
  package write permission. This makes GHCR a better default prepared-image
  target than Docker Hub because it does not require repository-specific
  Docker Hub credentials. GitHub also documents
  `org.opencontainers.image.source` as the container-image label for connecting
  a package back to a repository.
- Docker Compose interpolation supports default values and required-variable
  errors, including `${VAR:-default}` and `${VAR:?error}` forms. That lets the
  root `compose.yaml` default to `ghcr.io/link-assistant/formal-ai:latest` and
  fail early when `TELEGRAM_BOT_TOKEN` is missing.
- Telegram long polling uses `getUpdates`; the existing `formal-ai telegram`
  polling mode already maps to that API and only requires the bot token for a
  minimal startup.
- `docker/build-push-action`, `docker/metadata-action`, and
  `docker/login-action` are the existing ecosystem actions for registry login,
  tag/label generation, and image push. The repository already used those
  actions for optional Docker Hub publication, so extending the same release
  workflow to GHCR keeps the implementation close to the current pattern.

## Design Consequence

The prepared image should be published to GHCR on release with `latest` and
version tags, then documented as:

```bash
docker run --rm --privileged \
  -e TELEGRAM_BOT_TOKEN=123:abc \
  -v formal-ai-docker:/var/lib/docker \
  ghcr.io/link-assistant/formal-ai:latest
```

The compose wrapper should be:

```bash
TELEGRAM_BOT_TOKEN=123:abc docker compose up
```

Docker Hub remains useful as an optional mirror, but it should not be the only
prepared-image path because it requires additional credentials and repository
variables.
