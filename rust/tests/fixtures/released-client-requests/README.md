These first requests come from the public, sanitized text evidence for
[`40222223625`](https://github.com/link-assistant/formal-ai/actions/runs/40222223625),
the first actual Agent, Codex and Claude delivery attempt for v0.355.10.
No private session export or product binary was read.

The producer's sanitizer replaced the public model name and occurrences of
the public repository name because its test API key was the same string.
Only the model field and the exact public repository URL prefix are restored.
Other sanitizer placeholders are preserved. These fixtures exercise request
shape, advertised tools and initial issue-read planning; they do not claim
lossless recovery of redacted transcripts or real GitHub delivery.

Codex's export also removed its shell tool's `properties` object as account
metadata (it contains a shell `login` option). That loss is retained explicitly:
the protocol must refuse its incomplete schema. Its timing still covers the
planner before argument binding; no executable Codex replay equivalence is
claimed. A future exporter should distinguish schema declarations from actual
account metadata without weakening personal-data redaction.

`hive-bootstrap-placeholder.txt` is the complete public `.gitkeep` left in
Claude's actual PR 1416 at head `ef7e1ba7f4305558b97a2fe1a7849e36292c0fc6`.
Its Git blob is `7a9c542bcb875ce677c30723bf1bb50601f727e2` (174 bytes).
It is a timestamped Hive bootstrap instruction, rather than an empty file.
The regression binds its full content to the actual issue and branch and
preserves foreign, modified or appended content.

Agent HTTP responses took 48–51 seconds, Codex responses about 29 seconds,
and Claude responses about 8 seconds. Agent and Codex reached their finite
delivery deadlines while still progressing. The native regression times pure
planning without running any tools; JavaScript exercises the same requests.
