# Upload your full memory to a bug report

When you report a bug from the [formal-ai demo](https://link-assistant.github.io/formal-ai),
the maintainers can almost always reproduce the issue **only** when they have the
full memory of the agent at the moment it went wrong. This page is the single
link the prefilled "Report issue" body points to, so the issue body itself can
stay short while still giving you everything you need to attach the export.

## What "full memory" means

Clicking **Export memory** in the topbar writes `formal-ai-memory.lino`. The
file is a complete [`formal_ai_bundle`](case-studies/issue-18/README.md): the
entire seed (rules, concepts, tools, multilingual responses), your UI
preferences (demo mode, diagnostics mode), environment metadata (version, URL,
user agent), and the append-only event log of every turn, reasoning step, and
tool invocation in the session. With that file a maintainer can reconstitute
the exact session that triggered the bug — no extra steps required.

## Step 1 — Export it

1. Open [https://link-assistant.github.io/formal-ai](https://link-assistant.github.io/formal-ai).
2. Reproduce the bug, or stop right after you saw it.
3. Click **Export memory** in the topbar. The browser saves
   `formal-ai-memory.lino` to your downloads folder.

The same export is available in the CLI:

```bash
cargo run -- memory export --from memory.lino --path formal-ai-memory.lino
```

## Step 2 — Redact sensitive content

The export contains everything you typed into the chat, plus the assistant's
replies and tool outputs. Before you share it, open `formal-ai-memory.lino` in
any text editor (it is plain UTF-8) and remove:

- personal names, emails, phone numbers, addresses
- API keys, tokens, passwords, internal URLs
- any pasted source code or document you are not comfortable publishing

If you are unsure whether something is safe to share, redact it. The bundle
remains useful even with redacted strings replaced by `[REDACTED]`.

## Step 3 — Attach the file (choose one)

GitHub's issue uploader does not currently include `.lino` in its
[supported attachment types](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/attaching-files),
so you cannot drag `formal-ai-memory.lino` directly into the issue body. Two
workarounds both work; pick whichever is easier for you.

### Option A — Upload as a GitHub Gist (no zipping needed)

GitHub Gists accept arbitrary file extensions, so this is the lightest path.

1. Open [https://gist.github.com](https://gist.github.com).
2. In the filename field type `formal-ai-memory.lino`.
3. Paste the file contents (or drag the file in).
4. Choose **Create secret gist** if you want a private link, or **Create
   public gist** if the content is safe to publish.
5. Copy the gist URL and paste it into the issue body.

Secret gists are not indexed and only people with the link can view them, but
they are still readable to anyone with the URL — redact first.

### Option B — Wrap the file in a `.zip` and attach it

GitHub does accept `.zip` attachments on issues. Wrap the file locally:

- **macOS**: right-click `formal-ai-memory.lino` → *Compress* → drag the
  resulting `formal-ai-memory.zip` into the issue body.
- **Windows**: right-click `formal-ai-memory.lino` → *Send to* → *Compressed
  (zipped) folder* → drag the resulting `formal-ai-memory.zip` into the issue
  body.
- **Linux / WSL / macOS terminal**:
  ```bash
  zip formal-ai-memory.zip formal-ai-memory.lino
  ```
  Then drag the resulting `.zip` into the issue body.

## Re-importing a bundle

A maintainer (or you, on a different machine) can reconstruct the session with:

1. Click **Import memory** in the topbar, then pick the `.lino` file.
2. Or from the CLI:
   ```bash
   cargo run -- memory import --path formal-ai-memory.lino --into memory.lino
   ```

Both surfaces auto-detect the bundle vs. the legacy `demo_memory` format and
report any seed-version migrations they suggest.

## Why a `.lino` upload is not yet native

GitHub maintains a server-side allow-list of file extensions for issue and PR
attachments. `.lino` is not on that list at the time of writing, so the
uploader silently rejects it. We file this as an upstream limitation rather
than a formal-ai bug; the `.zip` and Gist workarounds above are the
documented escape hatch until GitHub broadens the allow-list.
