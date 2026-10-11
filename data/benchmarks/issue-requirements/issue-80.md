## Environment

- **Version**: 0.16.0
- **URL**: https://link-assistant.github.io/formal-ai/
- **User Agent**: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36
- **Worker**: wasm worker
- **Mode**: manual
- **Status**: Manual mode
- **Diagnostics**: off
- **Timestamp**: 2026-05-16T11:02:11.239Z

## Dialog

### 1. You

- **Role**: user
- **Time**: 13:02

```
Hi, can you write for me extension for owlbear? I am currently leading some dnd games and i want to try wargame. So, i need extensions that can track hp for different units, that can track Protection and Resistance stacks on unit an will reduce damage count on those stats. Also this extension should track cooldown of some abilities
```

### 2. formal-ai (reported message)

- **Role**: assistant
- **Time**: 13:02
- **Intent**: intent: unknown

```
I do not have a learned symbolic rule for that prompt yet. Add a Links Notation fact or rule, then run the request again.
```


## Reproduction Steps

1. Open https://link-assistant.github.io/formal-ai/
2. Send the prompt "Hi, can you write for me extension for owlbear? I am currently leading some dnd games and i want to try wargame. So, ..."
3. Click the report link on the dialog message

## Description

<!-- Please describe what looked wrong or incomplete. -->

## Attach full memory (recommended)

Click **Export memory** in the top bar to save `formal-ai-memory.lino`. The file is the **full memory** of the agent — the entire seed (rules, concepts, tools, multilingual responses), your UI preferences, environment metadata, and the complete append-only event log of this session (every user turn, assistant reply, reasoning step, tool invocation) — so the maintainer can reconstruct the exact session.

**Wrap the export in a `.zip` before attaching.** GitHub's issue uploader does not currently accept `.lino` files (see [supported file types](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/attaching-files)). On any OS:

- macOS: right-click → *Compress*.
- Windows: right-click → *Send to* → *Compressed (zipped) folder*.
- Linux: `zip formal-ai-memory.zip formal-ai-memory.lino`.

**Redact sensitive content first.** The export contains everything you typed into the chat. Open `formal-ai-memory.lino` in any text editor and remove personal names, secrets, API keys, internal URLs, or any pasted code you are not comfortable publishing before zipping and attaching.

