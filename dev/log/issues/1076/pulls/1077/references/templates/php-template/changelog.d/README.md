# Changelog fragments

Every code pull request adds **one** changelog fragment to this directory. At
release time the pipeline merges all fragments into `CHANGELOG.md`, derives the
next [Semantic Version](https://semver.org/) from the highest `bump:` it finds,
and then deletes the fragments.

This "changeset" pattern (borrowed from the JavaScript template's use of
[Changesets](https://github.com/changesets/changesets)) keeps the changelog
**conflict-free**: two PRs touch two different files instead of fighting over the
same line at the top of `CHANGELOG.md`.

## Creating a fragment

```bash
composer changeset
# or non-interactively:
php scripts/create-changeset.php --bump=minor --message="Add foo support"
```

## Format

A fragment is a Markdown file with optional frontmatter declaring the bump, then
[Keep a Changelog](https://keepachangelog.com/) category headings:

```markdown
---
bump: minor
---
### Added
- Describe the user-facing change here.
```

- `bump:` is one of `major`, `minor`, `patch`. The highest bump across all
  fragments wins for the release.
- Categories are `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`,
  `Security`. Anything without a heading is grouped under `Changed`.
- The filename does not matter; `create-changeset.php` uses a
  timestamp + branch + random suffix to avoid collisions.

`README.md`, `.gitkeep` and `fragment_template.md` are ignored — they are never
treated as fragments.
