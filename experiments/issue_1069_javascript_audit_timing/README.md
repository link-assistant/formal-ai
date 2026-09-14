# How long an advisory audit takes, and what it says when it fails

`scripts/check-javascript-dependencies.sh` retries an unanswered advisory
registry rather than reporting the outage as a dependency finding. That fix
needs three facts about the real world, and the first draft guessed all three.
This directory is where they were measured instead.

## What the guesses got wrong

**The deadline.** The first draft gave each attempt 120 seconds. A healthy
`npm audit --package-lock-only` over `desktop/package-lock.json` -- the largest
lockfile in the repository -- was then timed at **2m01s** while reporting
`found 0 vulnerabilities`. The gate killed that audit twice and reported the
outage it had itself caused:

```text
Auditing desktop/package-lock.json
::warning ...::Attempt 1/3 exited 124 within its 120s deadline ...
::warning ...::Attempt 2/3 exited 124 within its 120s deadline ...
::error   ...::The gate's 240s budget ran out with this lockfile still unaudited.
```

**The wording.** The list of strings that mean "the registry never answered"
was written from `bun audit`, which ends its message with `- 503`. `npm` says
something else entirely. Three consecutive samples on 2026-09-04, against a
registry that was genuinely degraded (`measured-2026-09-04.log`), all took
about 300 seconds -- npm's own internal give-up point -- and all exited 1 with:

```text
npm warn audit 503 Service Unavailable - POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - Service Unavailable
npm warn audit network timeout at: https://registry.npmjs.org/-/npm/v1/security/advisories/bulk
npm error audit endpoint returned an error
```

None of those matched the pattern, so a real npm outage fell through to the
failing branch -- the exact failure the gate had just been fixed for, still
present for the other half of the toolchain. `npm error audit endpoint returned
an error` is the reliable anchor: npm prints it when the endpoint failed and
prints nothing of the sort when it has advisories to report.

**The ceiling.** `Lint and Format Check` allows itself fifteen minutes and
spends about nine on a healthy run, so the gate has to fail well inside what is
left or `timeout-minutes` cancels the job before anyone can read why -- which is
what happened in run 100948708530. 180s per attempt with a 300s shared outage
budget puts the worst case near six minutes.

## Running it

```sh
./experiments/issue_1069_javascript_audit_timing/measure-npm-audit.sh 3 desktop 400
```

Samples are network-dependent by construction: the numbers above describe a
degraded registry, and a healthy one answers in seconds (run 100948708530
reached a clean answer in 4.35s). That spread is the point. The deadline has to
outlast a slow answer and still cut off a request that is never coming back.

`tests/unit/ci-cd/javascript_dependency_audit.rs` holds each of these
conclusions as a test, including one that reads the shipped defaults back out
of the script so a later edit cannot quietly return them to a guess.
