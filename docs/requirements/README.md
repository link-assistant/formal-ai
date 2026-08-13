# Requirement shards

`REQUIREMENTS.md` in the repository root is **generated** from this directory.
Edit the shard for your issue; never edit the root document.

```bash
rust-script scripts/assemble-requirements.rs           # check the root document is current
rust-script scripts/assemble-requirements.rs --write   # rebuild it from these shards
```

## Why this directory exists

`REQUIREMENTS.md` used to be one file that every issue appended a section to, so
every branch edited the same end-of-file region.
`scripts/analyze-merge-conflicts.py` counted 64 hand-resolved conflicts in it —
the worst append-only document in the repository, and every one of those
conflicts was between two issues that had nothing to do with each other.

One file per issue removes the collision entirely: two branches writing
requirements for two different issues create two different files.
`data/meta/merge-conflict-policy.lino` records this as the `append_only_document`
cause and the `shard_per_issue` mechanism.

## Adding requirements for a new issue

Create `issue-NNNN-<subject>.md`, where `NNNN` is the zero-padded issue number:

```markdown
## Issue #1234 What This Issue Requires

| ID | Requirement | implementation status |
| --- | --- | --- |
| R900 | ... | ... |
```

Then run `rust-script scripts/assemble-requirements.rs --write` and commit both
the shard and the regenerated `REQUIREMENTS.md`.

An issue may have several shards — issue #398, for example, has one for its
original requirements and one per review comment. Give each a distinct subject
slug; they assemble in file-name order.

## Assembly order

The order is read entirely from the file name, so there is no shared index file
to conflict on either:

| prefix      | position in the document | sorted by                           |
| ----------- | ------------------------ | ----------------------------------- |
| `preamble-` | first                    | file name                           |
| `issue-`    | middle                   | issue number, then the subject slug |
| `doctrine-` | last                     | file name                           |

`rust-script scripts/assemble-requirements.rs` fails if a shard whose first line
is `## Issue #N ...` is not named `issue-NNNN-<subject>.md`, so a shard copied
from another issue cannot silently keep the wrong number.
