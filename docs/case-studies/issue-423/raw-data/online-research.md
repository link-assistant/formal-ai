# Issue #423 online research

Captured on 2026-06-12 for
<https://github.com/link-assistant/formal-ai/issues/423>.

## GitHub issue and PR inputs

Commands used:

```sh
gh issue view https://github.com/link-assistant/formal-ai/issues/423 --json number,title,state,body,comments
gh api repos/link-assistant/formal-ai/issues/423/comments --paginate
gh pr view 424 --repo link-assistant/formal-ai --json number,title,state,isDraft,headRefName,baseRefName,url
gh api repos/link-assistant/formal-ai/pulls/424/comments --paginate
gh api repos/link-assistant/formal-ai/issues/424/comments --paginate
gh api repos/link-assistant/formal-ai/pulls/424/reviews --paginate
```

Findings:

- Issue #423 was open and requested README.md installation/deployment guide
  conversion to `sh`/PowerShell scripts and back.
- No issue comments were present.
- PR #424 already existed on branch `issue-423-85026fdc955a`.
- A follow-up PR comment on 2026-06-12 asked to continue in the widest possible
  sense, focus on a meta algorithm that produces algorithms, and double the
  test cases.
- No PR inline review comments or reviews were present.

The raw JSON outputs are committed beside this file.

## Popular GitHub project corpus

The original issue asked for at least 50 tests covering popular GitHub projects.
The 2026-06-12 follow-up asked for twice as many test cases, so the repository
snapshot was refreshed to 100 repositories with:

```sh
gh api 'search/repositories?q=stars:%3E1&sort=stars&order=desc&per_page=100'
```

The raw result is stored in `github-top-100-repositories.json`. The snapshot is
sorted by GitHub stars descending and stores each repository's full name, URL,
star count, primary language, default branch, and description.

Top 10 at capture time:

| # | Repository | Stars |
|---|------------|-------|
| 1 | `codecrafters-io/build-your-own-x` | 514595 |
| 2 | `sindresorhus/awesome` | 475035 |
| 3 | `freeCodeCamp/freeCodeCamp` | 446658 |
| 4 | `public-apis/public-apis` | 440960 |
| 5 | `EbookFoundation/free-programming-books` | 390118 |
| 6 | `openclaw/openclaw` | 378318 |
| 7 | `nilbuild/developer-roadmap` | 356826 |
| 8 | `donnemartin/system-design-primer` | 352721 |
| 9 | `jwasham/coding-interview-university` | 351193 |
| 10 | `vinta/awesome-python` | 302494 |

## Test corpus derived from the snapshot

Each repository feeds one README-to-`sh` conversion prompt in
`tests/unit/installation_conversion.rs`. The commands are intentionally simple
installation or verification commands so the test checks conversion routing and
command preservation rather than live project installation.

The 100 prompts span these installation command families:

- clone-and-enter flows;
- Node package managers (`npm`, `pnpm`, `yarn`);
- Python package/test commands;
- CMake, Make, Go, Maven, Cargo, Flutter, Docker, curl pipes, PowerShell
  one-liners, and project-provided shell scripts.

The full command-pair matrix lives in the unit test so the executable regression
source is the single source of truth.

## Meta-algorithm conclusion

The issue is not only "render this one README as a script." The reusable
construction pattern is:

1. collect representative examples for a problem class;
2. derive source and target surfaces;
3. extract a shared intermediate representation;
4. synthesize recognizers, extractors, renderers, and validators;
5. verify round-trip and command-preservation invariants with fixtures;
6. mirror the same algorithm across Rust and browser-worker runtimes;
7. promote the pattern so existing coding surfaces can be produced from the
   same class of algorithm construction.

For issue #423, the shared IR is the ordered install-step list. The same
problem-class-to-IR-to-renderer-to-verification pattern also covers the existing
coding catalog, program synthesis, program blueprints, numeric-list codegen,
and rule-synthesis paths.
