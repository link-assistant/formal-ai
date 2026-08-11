# Issue #989 online research

Research was performed on 2026-08-11 using primary project documentation.

## GitHub CLI gist behavior

Source: [GitHub CLI `gh gist create` manual](https://cli.github.com/manual/gh_gist_create)

The command accepts one or multiple filenames, but all supplied files belong to
the newly created gist. Gists default to secret/unlisted and `--public` makes
them publicly listed.

Design consequence: one invocation with harness, server, and merged files would
produce one URL, not the three separate file links required by the issue.
`--separate-context-links` therefore invokes the existing upload boundary once
per file. Visibility remains explicit and secret by default because the files
may contain a private conversation.

## OpenCode session export

Source: [OpenCode CLI documentation](https://opencode.ai/docs/cli/#export)

OpenCode documents `opencode export [sessionID]` as a JSON session export, with
an optional sanitization flag. That matches the repository's existing harness
context extractor and confirms there is no need for a second session-storage
reader.

Design consequence: the report implementation reuses `exported_context` for
the harness capture and the server log reader for the server capture, then
reuses `conversation_context_to_lino` for each. The merged export stays the
existing `ContextSource::Both` result.

## Code and licensing

The sources informed command semantics only. No third-party source code,
documentation text, fixtures, or datasets were copied, and no dependency was
added.
