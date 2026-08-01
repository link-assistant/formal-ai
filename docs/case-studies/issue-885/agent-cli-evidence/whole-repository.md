# Whole-repository Agent CLI evidence

- Date: 2026-08-01
- Audited commit: `901d6e7688e2d202234a83a8dab8139b5b36f59b`
- Agent CLI version: 0.25.4
- Session: `ses_042daf081ffecMWSd71J5l7B6E`
- Formal AI route: local `formalai/formal-ai`
- Working tree after the run: clean

The prompt was:

> Fact-check every statement in each Markdown document and the whole workspace,
> including relative references and dependent probabilities; preserve the
> result in statement-audit.lino.

The Agent CLI selected this tool call without a manually supplied command:

```text
formal-ai statement-audit --root . --output statement-audit.lino
```

Its final response preserved the exact command summary:

```json
{
  "statement_audit": {
    "contradictions": 4660,
    "evidence_captures": 0,
    "findings": 20885,
    "output": "statement-audit.lino",
    "root": ".",
    "skipped_paths": 243,
    "statements": 215975,
    "temperature": 0.699999988079071
  }
}
```

The generated 4,085,456-line graph was moved to `/tmp` rather than committed.
Its SHA-256 was
`7bd45aa123575af2d7a9f548e79f68c57d96018ba3c6e834114b3fbd54adf18a`.
The retained digest and immutable commit identify the result while avoiding a
roughly 152 MiB generated repository artifact.
