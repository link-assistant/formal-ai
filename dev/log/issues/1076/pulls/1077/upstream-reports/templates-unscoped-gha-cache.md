# Upstream report 2 - `cache-to: type=gha,mode=max` without `scope=` shares one key across every build

**Targets:** `link-foundation/rust-ai-driven-development-pipeline-template`,
`link-foundation/js-ai-driven-development-pipeline-template`,
`link-foundation/python-ai-driven-development-pipeline-template`
(`php-...` has no container build and is unaffected)

**Severity:** cache thrash between concurrent builds, plus unbounded growth
against a repository-wide 10 GB quota that is shared with every other cache
consumer in the pipeline.

---

## Title

`docker/build-push-action` writes to the default GitHub Actions cache scope, so
separate builds overwrite each other and crowd out `actions/cache` entries

## Affected sites

```
rust-ai-driven-development-pipeline-template   .github/workflows/release.yml:275   cache-to: type=gha,mode=max
rust-ai-driven-development-pipeline-template   .github/workflows/release.yml:1005  cache-to: type=gha,mode=max
js-ai-driven-development-pipeline-template     .github/workflows/release.yml:396   cache-to: type=gha,mode=max
python-ai-driven-development-pipeline-template .github/workflows/release.yml:490   cache-to: type=gha,mode=max
```

The python template already demonstrates the correct spelling one job later, so
this is an inconsistency within a single file rather than an unknown technique:

```
python-ai-driven-development-pipeline-template .github/workflows/release.yml:833
  cache-to: type=gha,mode=max,scope=${{ matrix.platform }}
```

## Why this matters

Two distinct effects, both documented by GitHub and by buildx:

1. **Scope collisions.** When `scope=` is omitted, buildx's GHA backend uses the
   default scope `buildkit` -- the parameter table for the `gha` backend reads
   `| scope | cache-to,cache-from | String | buildkit | Which scope cache object
   belongs to |`. Every unscoped `cache-to` in the repository therefore writes
   to that one key, and the same page states the consequence outright:

   > If you're using multiple builds in the same workflow, you should use
   > different scopes for each build. Otherwise, each build will overwrite the
   > cache of the previous, leaving only the final cache.

   So a repository with more than one image, more than one stage set, or a build
   that runs on both `push` and `pull_request` has those builds overwrite each
   other in turn, and each then restores a cache written for a different build
   and re-executes layers it should have hit.

2. **Quota crowding.** GitHub Actions cache is a *repository-wide* pool with a
   10 GB cap ("By default, the limit is 10 GB per repository") and
   least-recently-used eviction once it is exceeded ("the cache eviction policy
   will create space by deleting the caches in order of last access date, from
   oldest to most recent"). It is
   the same pool `actions/cache` uses for the cargo registry, `target/`,
   sccache, and node modules. `mode=max` exports every intermediate layer, which
   for a multi-stage compiled-language build is the largest single writer in the
   pipeline. Unscoped and unbounded, it evicts the compile caches that the rest
   of the workflow depends on.

Effect (2) is not hypothetical downstream: in `link-assistant/formal-ai` the
pool was measured at 11.44 GB against the 10 GB cap, with buildkit holding
5.26 GB against sccache's 2.44 GB, and the macOS lane's sccache hit rate fell
from 48% to 27% while its job died at its budget with no tests run
(link-assistant/formal-ai#1057). That repository derives from this template and
carries the same unscoped `cache-to`.

## Reproduction

Two builds in one repository, no `scope=`, and the second overwrites the first:

```yaml
# .github/workflows/repro.yml
name: repro
on: workflow_dispatch
jobs:
  a:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: docker/setup-buildx-action@v3
      - uses: docker/build-push-action@v6
        with: { context: ., file: Dockerfile.a, push: false, cache-from: type=gha, cache-to: type=gha,mode=max }
  b:
    needs: a
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: docker/setup-buildx-action@v3
      - uses: docker/build-push-action@v6
        with: { context: ., file: Dockerfile.b, push: false, cache-from: type=gha, cache-to: type=gha,mode=max }
  a_again:
    needs: b
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: docker/setup-buildx-action@v3
      - uses: docker/build-push-action@v6
        with: { context: ., file: Dockerfile.a, push: false, cache-from: type=gha, cache-to: type=gha,mode=max }
```

with two Dockerfiles that differ in an expensive layer:

```dockerfile
# Dockerfile.a
FROM alpine
RUN echo A && sleep 30 && touch /a
# Dockerfile.b
FROM alpine
RUN echo B && sleep 30 && touch /b
```

Job `a_again` builds the identical `Dockerfile.a` that job `a` just cached, yet
its log shows the `RUN` layer executing rather than `CACHED`, because job `b`
overwrote the shared scope in between. Adding `scope=a` / `scope=b` to both
`cache-from` and `cache-to` makes `a_again` report `CACHED`.

Cache occupancy is observable directly:

```bash
gh cache list --repo <owner>/<repo> --limit 100 --json key,sizeInBytes \
  --jq 'map(.sizeInBytes) | add / 1024 / 1024 / 1024'
```

## Workaround

Set an explicit scope at each call site without changing anything else:

```yaml
cache-from: type=gha,scope=<name>
cache-to: type=gha,mode=max,scope=<name>
```

## Suggested fix

Give each build a stable, distinct scope on both the read and the write side.
The name should identify the *image and stage set*, not the job, so that two
jobs building the same image still share:

```diff
-          cache-from: type=gha
-          cache-to: type=gha,mode=max
+          cache-from: type=gha,scope=docker-image
+          cache-to: type=gha,mode=max,scope=docker-image
```

Where a matrix builds per-architecture images, key the scope on the matrix leg,
which is what `python-.../release.yml:833` already does:

```yaml
cache-from: type=gha,scope=${{ matrix.platform }}
cache-to: type=gha,mode=max,scope=${{ matrix.platform }}
```

Where two steps in the *same* job publish the same layers to different
registries, only the first needs to export; the second should read and use
`cache-to: type=inline`, so the pool holds one copy rather than two.

Keep `mode=max`. `mode=min` exports only the final stage's layers, and for a
multi-stage compiled-language build the final stage is a thin runtime image, so
the expensive compile layers would stop being cached at all - which is the
regression the `cache-to` was added to prevent in the first place.

## Regression test

A workflow-linting assertion is enough, and does not need a CI run to evaluate:

```bash
# Every `cache-to: type=gha` must carry a scope.
if grep -rn 'cache-to: type=gha' .github/workflows/ | grep -v 'scope='; then
  echo "unscoped gha cache export: builds will overwrite each other's cache" >&2
  exit 1
fi
```

## References

- buildx `gha` cache backend, `scope` parameter: https://docs.docker.com/build/cache/backends/gha/
- GitHub Actions cache limits and LRU eviction: https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/caching-dependencies-to-speed-up-workflows#usage-limits-and-eviction-policy
- Downstream evidence: link-assistant/formal-ai#1057, link-assistant/formal-ai#1076
