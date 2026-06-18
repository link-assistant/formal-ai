# Issue 438 Case Study

## Collected Data

Raw issue and PR evidence is preserved under `raw-data/`:

- `issue-438.json` and `issue-438-comments.json`
- `pr-470.json`, `pr-470-conversation-comments.json`,
  `pr-470-review-comments.json`, and `pr-470-reviews.json`
- `related-prs-telegram-docker.json` and `related-prs-docker-release.json`
- `code-search-formal-ai-start-isolation.txt`,
  `code-search-docker-hub.txt`, and `code-search-telegram-polling.txt`
- `ci-runs-before.json`
- `online-research.md`
- `repro-before-docker-runtime.txt`,
  `repro-before-release-workflow.txt`, and
  `repro-before-docs-traceability.txt`

The issue has no discussion comments. The existing PR started as a draft WIP.
The latest baseline CI run captured before this work was green for the prepared
branch, so the change should be evaluated as a new feature rather than a repair
of an already failing branch.

## Online Facts

The online research in `raw-data/online-research.md` points to the upstream
facts that shape the implementation:

- GitHub Container Registry supports publishing container images from GitHub
  Actions with `GITHUB_TOKEN` and package write permission.
- GitHub's package guidance recommends an `org.opencontainers.image.source`
  label so a container image can be linked back to the source repository.
- Docker Compose supports required-variable and default-value interpolation,
  which fits a one-line `TELEGRAM_BOT_TOKEN=123:abc docker compose up` start.
- Telegram polling only needs the bot token, matching the existing
  `formal-ai telegram --mode polling` command.
- The repository already uses Docker's official login, metadata, and
  build-push actions for optional Docker Hub publication.

## Requirements

- R306: Preserve issue, PR, CI, related-work, online-research, and
  reproduce-before-fix artifacts under this case-study directory.
- R307: Publish a prepared Telegram Docker image to
  `ghcr.io/link-assistant/formal-ai:latest` and the release version tag without
  requiring Docker Hub credentials, while attaching repository source metadata.
- R308: Support a one-line Docker run with only `TELEGRAM_BOT_TOKEN` required.
- R309: Provide a root compose file with the same minimum token-only
  configuration and a persistent inner Docker volume.
- R310: Keep local build, Sysbox, and optional mirror paths available.
- R311: Include the prepared container image in release notes.
- R312: Add tests that keep compose, release workflow, docs, and requirements
  synchronized.

## Solution Options

Option A: Document a local `docker build` and `docker run` sequence only.

- Reuses the existing Dockerfile.
- Does not satisfy the issue's "fully prepared" image request because every
  operator must build locally before starting the bot.

Option B: Publish only to Docker Hub.

- Reuses the existing optional workflow branch.
- Adds operational friction because Docker Hub publication depends on
  repository variables and secrets.

Option C: Publish to GHCR by default and keep Docker Hub as a mirror.

- Uses the repository's GitHub Actions token and package permission.
- Gives the project a stable default image reference:
  `ghcr.io/link-assistant/formal-ai:latest`.
- Links the container package back to this repository with
  `org.opencontainers.image.source`.
- Preserves the existing Docker Hub mirror path for operators who need it.
- Pairs naturally with a checked-in `compose.yaml` for
  `TELEGRAM_BOT_TOKEN=123:abc docker compose up`.

Option C is the implemented path.

## Existing Components Checked

- `Dockerfile`: already builds the Rust binary into `konard/box-dind:2.1.1`
  and defaults to `formal-ai telegram --mode polling`; this change adds the
  repository source label used by GHCR package metadata.
- `scripts/verify-docker-runtime.sh`: already verifies the DinD runtime and
  `start-command` wrapper.
- `.github/workflows/release.yml`: already publishes crates.io releases and
  has optional Docker Hub image publication, making it the right place for
  GHCR publication.
- `scripts/create-github-release.rs`: already adds release badges, so it is
  extended with a GHCR badge instead of adding a second release-note tool.
- `README.md` and `ARCHITECTURE.md`: already describe the Docker-in-Docker
  Telegram image; this case extends that surface with prepared-image startup.

## Solution Plan

1. Add failing tests for missing compose startup, missing GHCR publication, and
   missing issue-438 traceability.
2. Add root `compose.yaml` defaulting to
   `ghcr.io/link-assistant/formal-ai:latest` and requiring
   `TELEGRAM_BOT_TOKEN`.
3. Extend release workflow auto and manual release jobs to publish GHCR after
   crates.io visibility and before optional Docker Hub and GitHub release
   publication.
4. Add repository source metadata to the runtime image.
5. Extend release-note generation with a GHCR badge.
6. Update README, architecture, requirements, and changelog docs.
7. Run targeted tests, formatting, and broader checks before pushing PR #470.

## Verification Plan

- `cargo test --test unit docker_runtime -- --nocapture`
- `cargo test --test unit release_workflow -- --nocapture`
- `cargo test --test unit issue_438_prebuilt_telegram_image_documents_are_present_and_traceable -- --nocapture`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `cargo test --all-features --verbose`
- `cargo test --doc --verbose`
- `rust-script scripts/check-file-size.rs`
- Fresh CI check on PR #470 after push.
