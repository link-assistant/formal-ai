# The Formal AI GitHub Action

`author-with-formal-ai` lets any repository hand a task to Formal AI and get
back a pull request Formal AI wrote itself: one commit, authored by
`github-actions[bot]`, carrying the session evidence and the four trailers the
self-hosting metric attributes, with no human commit on the branch.

It exists so that a wrong attempt is cheap. The draft opens on a branch nobody
depends on, its CI runs like any other pull request, and a poor result is
closed rather than hand-corrected — the defect goes to the meta algorithm
instead. Fail fast to learn fast.

## What it does

1. Resolves the task from an issue.
2. Opens a draft pull request under the GitHub Actions bot **first**, so the
   commit Formal AI writes can name it in its `Formal-AI-Pull-Request` trailer.
3. Starts `formal-ai serve` and drives the pinned
   [Agent CLI](https://github.com/link-assistant/agent) against it.
4. Pushes one commit with `Formal-AI-Session`, `Formal-AI-Model`,
   `Formal-AI-Evidence` and `Formal-AI-Pull-Request`, and the evidence bundle
   committed beside it.
5. Comments the commit on the pull request and the pull request on the issue.

A re-run does not author the task twice: the guard counts commits on the pull
request that carry its own trailer, and checks again immediately before pushing.

## Install

```yaml
name: Formal AI draft
on:
  issues:
    types: [opened]

permissions: {}

concurrency:
  group: formal-ai-draft-${{ github.event.issue.number }}
  cancel-in-progress: false

jobs:
  draft:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write
      issues: write
    env:
      GH_TOKEN: ${{ secrets.FORMAL_AI_BOT_TOKEN || github.token }}
    steps:
      # The checkout keeps its credential: the action pushes the bot branch
      # with it when no bot token is configured.
      - uses: actions/checkout@v7
        with:
          fetch-depth: 0
      - uses: link-assistant/formal-ai/.github/actions/author-with-formal-ai@main
        with:
          require-contract: 'false'
```

That configuration attempts **every new issue**. Formal AI reads the issue title
and body as the task, and the draft it opens is the experiment.

To attempt only issues you have written a contract for, drop
`require-contract` and label the issue `formal-ai-solve`:

```yaml
on:
  issues:
    types: [labeled]

jobs:
  draft:
    if: github.event.label.name == 'formal-ai-solve'
```

## Inputs

| Input | Default | What it is for |
| --- | --- | --- |
| `issue` | the event's issue | Issue number to author. Falls back to the oldest open issue carrying `label`. |
| `label` | `formal-ai-solve` | Label that marks an issue as a task when no number is given. |
| `require-contract` | `true` | `false` derives the task from the issue title and body instead of demanding the contract below. |
| `formal-ai-source` | `container` | Where the binary comes from. `source` builds it from the checkout, which only the repository owning the sources can do. |
| `formal-ai-image` | `ghcr.io/link-assistant/formal-ai:latest` | Image the binary is taken from. |
| `agent-version` | `0.26.0` | Pinned `@link-assistant/agent` version. |
| `base-branch` | the checked-out branch | Branch the draft targets. |
| `port` | `8931` | Port `formal-ai serve` listens on. |
| `evidence-root` | `dev/log/self-authored` | Where the evidence bundle is committed. |

Outputs: `pull-request`, `issue`, `authored`.

No compiler is installed unless you ask for `formal-ai-source: source`: the
binary is copied out of the published container, so a run costs a pull rather
than a build.

## The task contract

With `require-contract: true`, the issue body carries the task as plain lines:

```text
task: Edit the tracked file `attribution.rs`: add "Gemfile.lock" to the LOCKFILE_NAMES list. Change only that file and keep it valid Rust.
seed: scripts
produces: attribution.rs
into: scripts/attribution.rs
contains: "Gemfile.lock"
message: fix(metric): count Gemfile.lock as a lockfile
```

- `seed` is the directory copied into the scratch workspace the agent works in.
- `produces` and `into` are read in order and paired, so one run can write more
  than one artifact.
- `contains` is repeatable; each text must appear in one of the artifacts.
- `message` becomes the commit subject.

Write the task for one behaviour and one file where you can. A task naming two
things is answered after the first
([#1099](https://github.com/link-assistant/formal-ai/issues/1099)).

## Reading the result

The pull request is the experiment, so read it as one:

- **Did Formal AI make the change asked for**, or answer around it? Its session
  stream is committed under `evidence-root`.
- **Is CI green?** The draft runs your repository's checks like any branch.
- **Is there a human commit on the branch?** There must not be. A human commit
  is the one thing that invalidates the authorship claim; if the change needs
  correcting, close the branch and fix the meta algorithm.

Known behaviour worth recognising when a draft disappoints:
[#1095](https://github.com/link-assistant/formal-ai/issues/1095) a continuation
cue routed to web search,
[#1096](https://github.com/link-assistant/formal-ai/issues/1096) an edit
verified against a generated file,
[#1099](https://github.com/link-assistant/formal-ai/issues/1099) a two-part task
ending after the first part.

## Notes

- A push or pull request made with `GITHUB_TOKEN` starts no workflow run, so a
  draft opened with it has no checks until it is closed and reopened. Provide
  `FORMAL_AI_BOT_TOKEN`, a fine-grained token with contents and pull-requests
  write, to get checks without touching the branch.
- The action targets the branch the run checked out, not the repository's
  default branch, so a task belonging to a pull request lands in it.
- `changelog.d/` fragments, when the repository has that directory, are written
  by the bootstrap commit rather than by Formal AI: they are process record, and
  the metric excludes them from both sides of the share.
