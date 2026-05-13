# Non-Goals

These boundaries keep the project aligned with the symbolic, link-native direction.

## Runtime Non-Goals

- GPU-required neural inference is not a project target.
- A memoized answer cache is not a substitute for reasoning from source data and traceable steps.
- Hidden autonomous actions are not acceptable in chat mode.
- Unbounded reasoning loops are not acceptable; long-running agent work must expose actions and logs.
- Unverified generated code should not be presented as tested.
- Silent execution failures should not be hidden from users.
- Browser-only mode should not claim host-level execution capabilities.

## Data Non-Goals

- A large preloaded database is not the first objective.
- Vendoring massive public datasets into the repository is not a goal.
- Opaque binary knowledge stores are not enough unless paired with reviewable Links Notation exports.
- Destructive memory updates should not erase history by default.
- External web/API access should not become untracked context.
- Duplicate names should not be forced into one meaning when evidence shows different concepts.

## Product Non-Goals

- The visual graph is not meant to replace chat as the primary interface.
- The GitHub Pages demo is not expected to become a full production backend.
- Telegram support is not meant to hide environment limits or require unsupported execution features.
- The desktop app path is not a separate product until the library, API, and local data boundaries are stable.
- Agent mode is not intended for unsafe use on personal or production systems without isolation.

## Documentation Non-Goals

- Case studies should not become marketing pages.
- Vision documents should not imply that all long-term architecture exists today.
- Requirements should not be marked complete until there is implementation evidence or an explicit scope boundary.
- Research notes should not copy large external texts; they should summarize and cite sources.
