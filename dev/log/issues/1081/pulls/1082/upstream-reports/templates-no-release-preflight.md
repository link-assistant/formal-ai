# Upstream report 5 - no template proves it can publish before it spends the build

**Targets:** all five templates —
`link-foundation/rust-ai-driven-development-pipeline-template`,
`link-foundation/js-ai-driven-development-pipeline-template`,
`link-foundation/python-ai-driven-development-pipeline-template`,
`link-foundation/php-ai-driven-development-pipeline-template`,
`link-foundation/csharp-ai-driven-development-pipeline-template`

**Severity:** every publishing credential is exercised for the first time by
the step that uses it, after 30-55 capped minutes of building. A revoked
registry token therefore costs a full pipeline before anyone learns the
release cannot happen, and the answer arrives as a registry error in the
middle of a long log rather than as a verdict.

---

## Title

`release.yml` touches its publishing credentials only at publish time, so a
revoked token is discovered after the whole build (principle 16 of
`docs/CI-CD-BEST-PRACTICES.md` is unimplemented in all five templates)

## Evidence

Principle 16, "Prove You Can Publish Before You Build", is in the shared
best-practices document the five templates are written against
(`link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md`). It asks
for a `release-preflight` job that every publishing job `needs:`. No template
has one:

```console
$ for t in rust js python php csharp; do
    printf '%-7s %s\n' "$t" "$(grep -rli preflight "$t-template" | wc -l)"
  done
rust    0
js      0
python  0
php     0
csharp  0
$ grep -rl 'blobs/uploads' */ | wc -l    # the write-probe the principle asks for
0
```

In every template the release job `needs: [lint, test, build]` (js: `[lint,
test]`), so the whole matrix runs before any credential is touched. Measured
over each template's `release.yml` at the commits below, as the longest capped
path from the first job to the job that first uses each credential:

| template | job | publishing credential | capped minutes of build before its first use |
|---|---|---|---|
| rust | `auto-release` | `CARGO_REGISTRY_TOKEN`/`CARGO_TOKEN`, `DOCKERHUB_TOKEN` | **50** (`DOCKERHUB_TOKEN` again in `docker-publish`, at 110) |
| js | `release`, `instant-release` | `NPM_TOKEN`, `DOCKERHUB_TOKEN` | **30** (`docker-publish-build`, at 70) |
| python | `docker-publish-config` | `DOCKERHUB_TOKEN` | **85** |
| php | `auto-release` | none beyond `GITHUB_TOKEN`; the package reaches Packagist through the webhook, and `wait-for-packagist.php` is the first step that ever asks whether Packagist knows the package | **55** |
| csharp | `release`, `instant-release` | `NUGET_API_KEY` | **55** |

Reproduce the table:

```console
$ python3 - <<'EOF'
import re
from pathlib import Path
for t in ['rust','js','python','php','csharp']:
    lines = Path(f'{t}-template/.github/workflows/release.yml').read_text().split('\n')
    jobs = [(m.group(1), i) for i, l in enumerate(lines)
            if (m := re.match(r'^  ([A-Za-z0-9_-]+):\s*$', l))]
    info = {}
    for k, (name, st) in enumerate(jobs):
        body = '\n'.join(lines[st:jobs[k+1][1] if k+1 < len(jobs) else len(lines)])
        cap = re.search(r'timeout-minutes:\s*(\d+)', body)
        needs = re.search(r'needs:\s*\[([^\]]*)\]', body)
        info[name] = (int(cap.group(1)) if cap else 0,
                      [x.strip() for x in needs.group(1).split(',')] if needs else [],
                      sorted(set(re.findall(r'secrets\.([A-Z_]*(?:TOKEN|API_KEY)[A-Z_]*)', body))
                             - {'CODECOV_TOKEN', 'GITHUB_TOKEN'}))
    before = lambda j: max([info[d][0] + before(d) for d in info[j][1] if d in info] or [0])
    for name, (cap, _, creds) in info.items():
        if creds:
            print(f'{t:7} {name:22} {before(name):3} min of build first, then {",".join(creds)}')
EOF
rust    auto-release            50 min of build first, then CARGO_REGISTRY_TOKEN,CARGO_TOKEN,DOCKERHUB_TOKEN
rust    manual-release          50 min of build first, then CARGO_REGISTRY_TOKEN,CARGO_TOKEN,DOCKERHUB_TOKEN
rust    docker-publish         110 min of build first, then DOCKERHUB_TOKEN
rust    docker-merge-manifest  170 min of build first, then DOCKERHUB_TOKEN
js      release                 30 min of build first, then NPM_TOKEN
js      instant-release         30 min of build first, then DOCKERHUB_TOKEN,NPM_TOKEN
js      docker-publish-config   60 min of build first, then DOCKERHUB_TOKEN
js      docker-publish-build    70 min of build first, then DOCKERHUB_TOKEN
js      docker-publish         100 min of build first, then DOCKERHUB_TOKEN
python  docker-publish-config   85 min of build first, then DOCKERHUB_TOKEN
python  docker-publish-build    90 min of build first, then DOCKERHUB_TOKEN
python  docker-publish         150 min of build first, then DOCKERHUB_TOKEN
csharp  release                 55 min of build first, then NUGET_API_KEY
csharp  instant-release         55 min of build first, then NUGET_API_KEY
```

(php prints nothing because its only release credential is `GITHUB_TOKEN`; its
equivalent unchecked precondition is the Packagist webhook, and
`scripts/wait-for-packagist.php` polls for up to five minutes inside the same
30-minute job before reporting the package is not there.)

Two consequences, both observed in the repository this report is filed from
(`link-assistant/formal-ai`, which is built from the rust template):

1. **A revoked credential costs a full build.** `auto-release` there is a
   90-minute job whose GHCR push alone is budgeted 45 minutes, and the push is
   the first thing that ever asks whether the token can write.
2. **A login is not a probe.** `docker/login-action` succeeding means the
   credential is valid, not that it can write the package it is about to push.
   The templates use it as their only pre-push signal. Worse, on Docker Hub the
   token endpoint answers a `pull,push` scope request with **200** and a token
   whose `access` claim silently contains only `pull` (measured below), so even
   a check that asks for the push scope passes and the push then fails 403.

## Reproducible example

**A token request is not a write.** Ask Docker Hub for push rights on a
repository you have no credentials for at all:

```console
$ curl -s -o token.json -w '%{http_code}\n' \
    'https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/hello-world:pull,push'
200
$ python3 -c "import base64,json,sys; t=json.load(open('token.json'))['token'].split('.')[1]; \
    print(json.loads(base64.urlsafe_b64decode(t+'='*(-len(t)%4)))['access'])"
[{'type': 'repository', 'name': 'library/hello-world',
  'actions': ['pull'], ...}]
```

200, a usable token, and `push` quietly dropped from the grant. ghcr.io is
stricter about the scope itself -- it answers the same request `403 DENIED` --
but that only covers the scoped form; `docker/login-action` authenticates
against the registry, not against a package, so it passes for any valid PAT
regardless of whether the package can be written:

```console
$ curl -s -o /dev/null -w '%{http_code}\n' \
    'https://ghcr.io/token?service=ghcr.io&scope=repository:link-assistant/formal-ai:pull'
200
$ curl -s -o /dev/null -w '%{http_code}\n' \
    'https://ghcr.io/token?service=ghcr.io&scope=repository:link-assistant/formal-ai:pull,push'
403
```

**The probe the principle asks for**, against a repository the credential
really can write (`202` = the registry opened an upload session; the `DELETE`
cancels it, so nothing is stored and no tag moves):

```console
$ token=$(curl -s -u "$USER:$PAT" \
    'https://ghcr.io/token?service=ghcr.io&scope=repository:OWNER/NAME:pull,push' \
    | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
$ curl -s -D - -o /dev/null -X POST -H "Authorization: Bearer $token" \
    https://ghcr.io/v2/OWNER/NAME/blobs/uploads/ | head -2
HTTP/1.1 202 Accepted
Location: /v2/OWNER/NAME/blobs/uploads/1a2b3c...
$ curl -s -o /dev/null -w '%{http_code}\n' -X DELETE \
    -H "Authorization: Bearer $token" https://ghcr.io/v2/OWNER/NAME/blobs/uploads/1a2b3c...
204
```

Unauthenticated, the same `POST` is `401`, so the probe cannot pass by
accident:

```console
$ curl -s -o /dev/null -w '%{http_code}\n' -X POST \
    https://ghcr.io/v2/link-assistant/formal-ai/blobs/uploads/
401
```

One round trip per registry, nothing stored, and the only form of the check
that is not a guess. The crates.io equivalent is `GET /api/v1/me` (`403` with
no token, `200` with a valid one) plus a public
`GET /api/v1/crates/<name>/owners` to confirm the authenticated account is
actually an owner -- a valid token belonging to the wrong account passes the
first check and fails the publish.

## Workaround

Until a preflight job exists, a repository built from one of these templates
can add a first step to its release job that probes the credentials before the
job's expensive steps run. That still costs the upstream `lint`/`test`/`build`
matrix, but it stops the release job itself from failing an hour in:

```yaml
      - name: Probe the release credentials
        env:
          CARGO_TOKEN: ${{ secrets.CARGO_TOKEN }}
        run: |
          status=$(curl -s -o /dev/null -w '%{http_code}' \
            -A "release-preflight" -H "Authorization: $CARGO_TOKEN" \
            https://crates.io/api/v1/me)
          [ "$status" = 200 ] || { echo "::error::crates.io rejected the token ($status)"; exit 1; }
```

## Suggested fix in code

Add a `release-preflight` job before the matrix and make every publishing job
need it. The mode is what keeps it honest on a pull request: a fork has no
secrets, so there a missing credential is a report, not a failure.

```yaml
  release-preflight:
    name: Release Preflight
    runs-on: ubuntu-latest
    timeout-minutes: 5
    permissions:
      contents: read
      packages: write   # the GHCR probe opens a blob upload session
    steps:
      - uses: actions/checkout@v5
        with:
          persist-credentials: false
      - name: Probe every release credential
        env:
          PREFLIGHT_MODE: >-
            ${{ (github.event_name == 'push' && github.ref == 'refs/heads/main')
            && 'release' || 'report' }}
          # ... the registry credentials this template releases with ...
        run: bash scripts/preflight-credentials.sh

  auto-release:
    needs: [lint, test, build, release-preflight]
    if: ${{ !cancelled() && needs.release-preflight.result == 'success' }}
```

A working implementation of `scripts/preflight-credentials.sh` — 345 lines of
POSIX-ish bash with no dependency beyond `curl` and `base64`, covering
crates.io (token *and* crate ownership), ghcr.io and Docker Hub through the
blob-upload probe above — is in
[`link-assistant/formal-ai`](https://github.com/link-assistant/formal-ai)
at `scripts/preflight-credentials.sh`, with its tests in
`tests/unit/ci-cd/issue_1081/release_preflight.rs` and an offline `curl` stub
in `experiments/issue_1081_preflight/`. It is language-agnostic; a template
only has to swap the crates.io check for its own registry:

* **npm** (js) — `GET https://registry.npmjs.org/-/whoami` with the bearer
  token, then compare the login it returns against the package's
  `maintainers`. Measured: `401 {"error":"Unauthorized"}` with no token and
  `401 {}` with an invalid one, so the failure is unambiguous. The js template
  also publishes with provenance (`id-token: write`), whose own precondition is
  that `ACTIONS_ID_TOKEN_REQUEST_URL` is set in the job — check it, because it
  is absent exactly when the permission was not granted.
* **PyPI** (python) — the template uses trusted publishing, so there is no
  token to check; what breaks is the trusted publisher not being configured for
  the workflow. Request the OIDC token with audience `pypi` and exchange it at
  `POST https://pypi.org/_/oidc/mint-token`, which names the mismatch instead
  of hinting at it. Measured with a deliberately bad payload:
  `422 {"errors":[{"code":"invalid-payload","description":"malformed JWT"}]}`.
  The mint is what the publish step does anyway and the token it returns
  expires in 15 minutes.
* **NuGet** (csharp) — `PUT https://www.nuget.org/api/v2/package` with an empty
  body. Nothing is uploaded, and the answer is exact. Measured:

  ```console
  $ curl -s -o - -w ' <- %{http_code}\n' -X PUT --data-binary '' \
      https://www.nuget.org/api/v2/package
  An API key must be provided in the 'X-NuGet-ApiKey' header to use this service <- 401
  $ curl -s -o - -w ' <- %{http_code}\n' -X PUT --data-binary '' \
      -H 'X-NuGet-ApiKey: oy2bogus' https://www.nuget.org/api/v2/package
  The specified API key is invalid, has expired, or does not have permission to access the specified package. <- 403
  ```

  A key that is valid gets past authentication and fails on the missing
  package instead, which is the pass condition.
* **Packagist** (php) — the precondition is registration, not a secret:
  `GET https://repo.packagist.org/p2/<vendor>/<name>.json` is `200` for a
  package Packagist knows and `404` for one it does not. The template already
  ships the client (`scripts/src/Packagist.php`), so the preflight is a call to
  code that exists; today the first thing that asks is
  `scripts/wait-for-packagist.php`, 55 capped minutes in and after the version
  commit and tag have already been pushed.
* **Docker Hub / GHCR** (rust, js, python) — the blob-upload probe verbatim.

Three rules the implementation follows, each of which is a defect if dropped:

1. **Report every failure, not the first.** One report naming all three broken
   credentials beats three runs each naming one, so no probe aborts the script.
2. **Report `unknown`, never a guess.** A registry that times out or answers
   429 has not said the credential is broken. `0 verified, 3 unknown` is
   actionable; `no failures` is not — and a run that verified nothing is not a
   pass.
3. **Probe with a write, not a login**, for the reason measured above.

## Where this was verified

Template commits (unchanged between issue #1079 and this analysis):

| template | commit |
|---|---|
| rust | `4d444d976f6bb46168b9f4a7001c775bf7c0ef6b` |
| js | `338fafa7b428637a18fe06a114d446a5cfb23ae7` |
| python | `81c978684f04ce018f606e5fcd418202157e009d` |
| php | `5c3c906c1d8b30941f639159ebb9bf0657616fdb` |
| csharp | `83efb9e4482ffed9c9ef1c7b07ea250ac6d3b141` |
