# Workspace member read failures

Release package discovery previously treated every member manifest read error
as an absent manifest. An invalid UTF-8 manifest or a directory named Cargo.toml
could silently redirect the release to a later publishable member.

Reproduce with a workspace containing members `broken` and `public`. Make
`broken/Cargo.toml` contain byte `ff`, or create a directory at that path, and
provide a valid package in `public/Cargo.toml`. The original helper selects
`public`. The repair returns the read error with the failing member path.
A genuinely absent member retains the existing fallback to the next member.

The physical native regression is
`cargo_package_identity::unreadable_workspace_members_do_not_redirect_release_identity`.
It exercises both malformed filesystem states and checks that neither the
invalid input nor the later package changes. A separate missing-member control
preserves the previous supported behavior. Both are part of the existing
`workflow_contracts` native owner and the ordinary native CI admission closure.

Local validation passed the native-admission JavaScript suite, all affected
available JavaScript gates, rustfmt and whitespace checks. No local Rust build
ran; actual execution of the new native regressions remains pending hosted CI.
The retained original source demonstrates the discarded error, but an executed
before-fix native failure has not yet been observed.

The pinned Rust template has the same discarded read in
`scripts/rust-paths.rs:254`. The fresh upstream source is bound to commit
`260ae6f826d6bff0cbfe2280098c287a4c09b3f4`;
the matching defect is reported in
[261](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/261).
Reporting it does not establish an upstream repair. No independent authorship or complete-issue acceptance is claimed.
