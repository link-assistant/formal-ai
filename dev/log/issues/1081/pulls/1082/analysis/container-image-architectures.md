# Finding: the published container images are `linux/amd64` only

Found while working through requirement R4 (`CI-CD-BEST-PRACTICES.md`)
principle by principle. Principle 13, "Container Images: Native Runners per
Architecture", says:

> **Publish images for every architecture your users run.** A single-
> architecture image silently excludes Apple Silicon, Graviton, and arm CI
> runners.

## Evidence

### What is published, read from the registry

Anonymously, no credentials, at the time of writing:

```console
$ TOKEN=$(curl -sS "https://ghcr.io/token?scope=repository:link-assistant/formal-ai:pull&service=ghcr.io" \
    | python3 -c 'import sys,json;print(json.load(sys.stdin)["token"])')
$ curl -sS -H "Authorization: Bearer $TOKEN" \
    -H 'Accept: application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json' \
    https://ghcr.io/v2/link-assistant/formal-ai/manifests/latest
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.oci.image.index.v1+json",
  "manifests": [
    { "digest": "sha256:11fc56ce…", "platform": { "architecture": "amd64", "os": "linux" } },
    { "digest": "sha256:93cb55d1…", "platform": { "architecture": "unknown", "os": "unknown" },
      "annotations": { "vnd.docker.reference.type": "attestation-manifest" } }
  ]
}
```

One real platform. The second entry is the build attestation, not an image.

### What the pipeline asks for

Five `docker/build-push-action@v7` steps in `.github/workflows/release.yml`
(lines 151, 715, 760, 920, 963). **None of them passes `platforms:`**, so each
builds for the runner's architecture, and every runner in those jobs is
`ubuntu-latest`.

```console
$ grep -c 'platforms:' .github/workflows/release.yml
0
$ grep -rn 'setup-qemu' .github/workflows/
(nothing)
```

### What the README promises

`README.md:23` announces the image as
`ghcr.io/link-assistant/formal-ai:latest`, and `README.md:715-737` gives three
`docker run` invocations against it with no architecture caveat. An Apple
Silicon or Graviton reader following those lines gets an emulated run at best.

### The base image is not the blocker

`Dockerfile:59` is `FROM konard/box-dind:2.1.1`, which **already publishes both
architectures**:

```console
$ TOKEN=$(curl -sS "https://auth.docker.io/token?service=registry.docker.io&scope=repository:konard/box-dind:pull" \
    | python3 -c 'import sys,json;print(json.load(sys.stdin)["token"])')
$ curl -sS -H "Authorization: Bearer $TOKEN" \
    -H 'Accept: application/vnd.docker.distribution.manifest.list.v2+json' \
    https://registry-1.docker.io/v2/konard/box-dind/manifests/2.1.1
… "platform": { "architecture": "amd64", "os": "linux" } …
… "platform": { "architecture": "arm64", "os": "linux" } …
```

`ubuntu-24.04-arm` is already in use by this repository, in
`.github/workflows/desktop-release.yml:195`, so the runner is available too.

## What is already right

Three of principle 13's five bullets hold today, which is why this is a
one-line gap and not a rewrite:

- **No QEMU emulation.** `setup-qemu-action` appears nowhere.
- **Caching on every publishing build.** `cache-from: type=gha` on all five
  steps; `cache-to: type=gha,mode=max` on the two GHCR legs, deliberately
  `inline` on the Docker Hub legs that re-push the layers the GHCR leg just
  exported (issue #1057).
- **The release is not gated on the image push.** The crate publish and the
  GitHub Release come first.

The two that do not: publish every architecture, and assert what you shipped.

## Why it is not fixed in pull request #1082

Two reasons, both about evidence rather than effort:

1. It changes **what is published**, not what is reported. Issue #1081 is about
   false positives, false negatives, warnings and errors -- signals. A
   single-architecture image is a correct signal about a narrow artifact.
2. The only real verification is a release run against the live registries.
   This pull request can verify its own changes locally and through CI; it
   cannot verify a manifest merge that only happens on a push to `main`, and
   the job it would change is the one that publishes the crate.

## Suggested implementation

```yaml
build-image:
  strategy:
    fail-fast: false
    matrix:
      include:
        - platform: linux/amd64
          runner: ubuntu-latest
        - platform: linux/arm64
          runner: ubuntu-24.04-arm
  runs-on: ${{ matrix.runner }}
  steps:
    - uses: docker/build-push-action@v7
      with:
        context: .
        platforms: ${{ matrix.platform }}
        cache-from: type=gha,scope=docker-image-${{ matrix.platform }}
        cache-to: type=gha,mode=max,scope=docker-image-${{ matrix.platform }}
        outputs: type=image,push-by-digest=true,name-canonical=true,push=true

merge-manifest:
  needs: [build-image]
  steps:
    - run: docker buildx imagetools create -t "$IMAGE:$VERSION" -t "$IMAGE:latest" $DIGESTS
```

Then assert it, so the gap cannot reopen silently: extend
`scripts/verify-ghcr-visibility.sh` -- which already pulls the published image
anonymously -- to read the manifest index and fail if it does not list every
intended platform. That check is the difference between this finding being
fixed and being fixed *until someone drops the matrix*.

Note that the publishing legs build with `BINARY_SOURCE=compile`
(`Dockerfile:13`, `:49`), so the arm64 leg compiles the crate natively on the
arm runner rather than copying an amd64 artifact -- correct by construction,
and the reason the matrix runner must be native rather than emulated.

## Filed as

<https://github.com/link-assistant/formal-ai/issues/1084>
