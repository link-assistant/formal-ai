# Online research: changelog fragment lifecycle

Established fragment-based release tools agree on one invariant: once a
fragment is incorporated into a release changelog, it is consumed rather than
left pending.

## Findings

- [Towncrier documentation](https://towncrier.readthedocs.io/) describes news
  fragments as inputs to a release build. Its
  [release tutorial](https://towncrier.readthedocs.io/en/23.10.0/tutorial.html)
  shows the production build removing news files with `git rm` and staging the
  generated news file. This closely matches the required fix.
- [Scriv's collect command](https://scriv.readthedocs.io/en/latest/commands.html)
  deletes collected fragments by default. Its `--add` option stages both the
  changelog and deleted fragment files.
- [Changesets' official workflow](https://github.com/changesets/changesets/blob/main/docs/intro-to-using-changesets.md)
  says `changeset version` consumes all changesets while updating versions and
  changelogs. This is the JavaScript template's lifecycle model.
- [release-plz](https://release-plz.dev/docs/usage) derives release changes from
  Git history rather than persistent fragment files. That avoids stale-fragment
  replay by design, but adopting it would be a larger pipeline migration and is
  unnecessary for this repair.

## Decision

Retain the repository's existing fragment architecture and enforce the common
consume-after-success invariant. Deletion occurs after the changelog write, so
a failed write leaves inputs available for retry. Staging uses `git add -A`, so
the release commit atomically records the changelog, package version, lock file,
and fragment deletions.

For the one-time cleanup, reconstruct from immutable release trees rather than
attempting to de-duplicate the polluted changelog text. This preserves the
first released form of each fragment, recovers later-deleted released files,
and gives every assignment a commit-level provenance record.
