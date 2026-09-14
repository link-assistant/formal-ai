# Upstream report 7 - `--compaction-model same` is silently ignored whenever the `--compaction-models` default is present

**Target:** `link-assistant/agent` (`@link-assistant/agent`, reproduced on
0.26.0 and read in the published 0.26.1 source)

**Severity:** a documented flag accepts its value, logs nothing, and has no
effect. Sessions that asked to be compacted with their own model are compacted
with a hosted third-party model instead -- which for a self-hosted or offline
provider means an outbound request the operator never configured, and, when
that host answers with an error, a failed run (report 8).

---

## Title

`--compaction-model` never takes effect: the default value of
`--compaction-models` shadows it, so the single-model branch is unreachable

## Affected site

`src/cli/model-config.js:383-455` (0.26.1, `npm pack @link-assistant/agent`):

```js
  // Check for --compaction-models (cascade) first — it overrides --compaction-model
  const cliCompactionModelsArg = getCompactionModelsFromProcessArgv();
  const defaultCompactionModels = getDefaultCompactionModels(defaultOptions);
  const compactionModelsSource =
    cliCompactionModelsArg ||
    (argv['compaction-models'] &&
      argv['compaction-models'] !== defaultCompactionModels)
      ? 'cli'
      : 'default';
  const compactionModelsArg =
    cliCompactionModelsArg ??
    argv['compaction-models'] ??
    defaultCompactionModels;

  const modelNames = parseLinksNotationSequence(compactionModelsArg);

  if (modelNames.length > 0) {
    ...
    return { ... };
  }

  // Fallback to single --compaction-model
  const cliCompactionModelArg = getCompactionModelFromProcessArgv();
```

## Root cause

The comment above the block states the intended precedence -- the cascade
overrides the single model -- and that is reasonable when the user actually
passed a cascade. But `compactionModelsArg` falls back to
`defaultCompactionModels`, which is never empty, so `modelNames.length > 0` is
always true and the function always returns from the cascade branch. The
`// Fallback to single --compaction-model` code below it is dead in every
invocation that does not somehow produce an empty default.

The precedence that is implemented is therefore not "an explicit cascade beats
an explicit single model", it is "the *default* cascade beats an explicit
single model" -- the one ordering nobody would write down.

The fix is already half-present: three lines above, `compactionModelsSource`
distinguishes exactly the case that matters (`'cli'` vs `'default'`). It is
computed, logged, used to pick a log level for skipped entries -- and not used
to decide which branch runs.

## Reproducible example

Self-contained, no hosted account, no network egress beyond loopback: a
stdlib-only OpenAI-compatible server on `127.0.0.1` is the *only* provider the
CLI is configured with, so any other model name in the compaction cascade is
unambiguously not something the operator asked for.

- driver: <https://github.com/link-assistant/formal-ai/blob/main/experiments/issue_1079_agent_compaction_flag/run.sh>
- mock provider: <https://github.com/link-assistant/formal-ai/blob/main/experiments/issue_1079_agent_compaction_flag/mock_openai_server.py>

```bash
bun add -g @link-assistant/agent          # 0.26.0 / 0.26.1
git clone https://github.com/link-assistant/formal-ai && cd formal-ai
./experiments/issue_1079_agent_compaction_flag/run.sh
```

Its first two cases run the same one-turn prompt against the same local
provider, once per spelling, and grep the CLI's own `compaction models cascade
configured` line plus the provider the summarizer actually reached (the third
case belongs to the companion report and can be ignored here):

```
== singular: agent --compaction-model same
"message":"compaction models cascade configured"
"models":["opencode/big-pickle","kilo/minimax-m2.5-free","same"],"source":"default"
"service":"session.summary","providerID":"opencode","modelID":"big-pickle"

== plural: agent --compaction-models (same)
"message":"compaction models cascade configured"
"models":["same"],"source":"cli"
"service":"session.summary","providerID":"mock","modelID":"mock-model"
```

Read the first block twice. `same` *is* in the resolved list -- the flag was
parsed, and it was appended to the default cascade rather than replacing it --
but it is third, `source` is `default`, and the summarizer went to
`opencode/big-pickle`. The only provider in the config file is `mock`.

The script exits non-zero if the first block stops reproducing (upstream fixed
it) or if the second stops holding (the workaround broke), so it doubles as a
canary against the fix -- the downstream repository runs it that way, and will
notice the day this is closed.

## Workaround

Use the plural flag with a one-element links-notation sequence:

```
--compaction-models "(same)"
```

`source` becomes `cli`, the cascade is exactly `["same"]`, and summarization
resolves to the session's own model. This is what the downstream repository now
passes at all 27 call sites, pinned by a test so it cannot be reverted to the
singular spelling by anyone reading the flag list:
<https://github.com/link-assistant/formal-ai/pull/1080>.

Note the quoting: `(same)` is links notation, and an unquoted `(` is a shell
syntax error in bash, so the parentheses must be inside the quotes.

## Suggested fix

One condition, using the value the function already computes:

```diff
-  if (modelNames.length > 0) {
+  // Only let the cascade win when the user actually asked for one. Taking this
+  // branch on the *default* cascade makes the single-model branch below
+  // unreachable, which silently discards an explicit --compaction-model.
+  const singularRequested =
+    getCompactionModelFromProcessArgv() != null ||
+    (argv['compaction-model'] != null &&
+      argv['compaction-model'] !== getDefaultCompactionModel(defaultOptions));
+
+  if (modelNames.length > 0 && !(compactionModelsSource === 'default' && singularRequested)) {
```

Two smaller changes are worth landing with it, and either one alone would have
turned this from a silent no-op into a five-second diagnosis:

1. **Warn on the conflict rather than resolving it silently.** If both spellings
   are given explicitly, one of them is being discarded; say which.
2. **Log the singular argument too.** `compaction models cascade configured`
   reports `models` and `source` but never mentions `--compaction-model`, so the
   log is consistent with the flag having been honoured *and* with it having
   been dropped. Adding `compactionModel: compactionModelArg ?? null` makes the
   two cases distinguishable in an artifact after the fact -- which is how this
   was eventually found, and it took a mock provider to do it.

An alternative fix, if the precedence is meant to stay as documented: make
`--compaction-models` have no yargs default, and apply
`getDefaultCompactionModels()` only after both flags have been read. That
removes the class of bug rather than this instance, since any future flag pair
with a non-empty default on the higher-precedence side has the same shape.

## Why this is not cosmetic

`--compaction-model same` reads as an explicit instruction to keep the session
on one model, and it is the natural thing to pass when the configured provider
is local, air-gapped, metered, or under test. What actually happens is a
request to a hosted gateway the operator did not configure. In this
repository's CI that request then failed and took the whole run's exit status
with it -- see report 8, filed separately because the two defects are
independent: fixing either one alone stops the failures, and both are worth
fixing.

## Related

- <https://github.com/link-assistant/agent/issues/219> -- "Add
  --compaction-model and set it to free `gpt-nano` by default", the flag this
  report is about.
- <https://github.com/link-assistant/agent/issues/217> -- "Turn on
  `--summarize-session` by default with the same model". The behaviour above
  contradicts the stated intent of that issue's title.
- <https://github.com/link-assistant/agent/issues/275> -- single-provider
  cascade noise, filed earlier from the same downstream repository.
- <https://github.com/link-assistant/formal-ai/pull/1080> -- the downstream fix
  and the reproduction.
