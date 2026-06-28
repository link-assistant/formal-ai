# Issue 492 Online Research

Date: 2026-06-28

## Sources

- docs.rs badge documentation: <https://docs.rs/about/badges>
- Shields.io badge documentation: <https://shields.io/badges>
- GitHub Actions workflow status badge documentation:
  <https://docs.github.com/en/actions/how-tos/monitor-workflows/add-a-status-badge>

## Findings

docs.rs badge documentation points crate owners to Shields.io for badge
generation. That supports using a Shields docs.rs badge in README.md instead of
the legacy `https://docs.rs/<crate>/badge.svg` endpoint.

Shields.io documents static badge URLs of the form
`https://img.shields.io/badge/<label>-<message>-<color>`. Static badges are a
better fit for historical GitHub release notes because the release version can be
encoded directly into the badge text while the badge link points to the exact
artifact version.

GitHub Actions documents workflow status badges for README usage, including the
branch query (`?branch=main`) to scope the badge to the default branch. That
matches the restored README CI/CD and Desktop Release badges.

## Decision

Use dynamic status badges where current status is useful, such as README workflow
or current package-version badges. Use static version badges in immutable release
notes so a release page cannot later display `invalid` or `failing` because an
external status endpoint changed state.
