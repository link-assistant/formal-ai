# Links notation style

This is the style of the links notation we own: the `.lino` files under
`data/seed/` and `data/meta/` that Formal AI reads as its seed memory and its
ledgers. It follows the owner's vision of 2026-10-08 (R1188-U6 and R1188-U7 in
`docs/requirements/issue-1188-user-requirements.md`):

> Double check our links notation fully human readable, yet concise and
> deduplicated as possible. [...] In links notation we should prefer `-` over
> `_`.

The rules below are measured and applied by scripts, not kept by hand. The
rule data lives in `data/meta/notation-rules.lino`.

## What a name is

A name is an unquoted lowercase token of a line that reads as an identifier:
a link name, a key, a role, an intent id or a slug. In

```lino
file_read_action
  defined-by action
  role file_read_action_cue
  lexeme en
    surface
      text "read the file"
```

the names are `file_read_action`, `defined-by`, `action`, `role`,
`file_read_action_cue`, `lexeme`, `en`, `surface` and `text`. These are not
names, and no rule touches them:

- quoted text (`"read the file"`), which is human text or an external spelling;
- a token with `/` or `.`, which is a path or a file name;
- a capitalized token (`WebSearch`, `FORMAL_AI_DATA_DIR`, `Q42`), which is an
  external spelling: a tool name, an environment variable, a Wikidata id;
- code identifiers in JavaScript and Rust, which follow their language's own
  convention (`camelCase`, `snake_case`) even when they read a notation name.

`data/meta/self-ast/` is excluded: it is a generated census of the Rust syntax
trees, so its names are code identifiers.

## Names prefer `-` over `_`

A new name is written with `-`: `defined-by`, `sentence-coverage`,
`specialization-ceiling`. Existing `_` names are renamed by rule, one family of
files at a time (below), never by hand.

## Names are full English words

A name says what it holds in full words: `position`, not `pos`; `arguments`,
not `args`; `identifier`, not `id`. The abbreviations the measure knows, each
with its full word, are listed under `abbreviations` in
`data/meta/notation-rules.lino`. Established acronyms and external terms
(`json`, `url`, `qid`, `triz`) are listed under `proper-terms` with what they
stand for; they stay as they are.

## Shared structure is stated once

Two measures find structure that is repeated instead of stated once:

- a **duplicated block**: the same child block of three or more lines under
  more than one link. Name it once and reference it by name.
- a **repeated field**: a leaf line that more than half of the sibling records
  of one kind repeat verbatim, such as `manual "not yet confirmed"` on most
  records of the requirement-status ledger. State it once on the parent as a
  default, and write it on a record only where the record differs.

Deduplication must not cost readability: a reader of one record should still
find what applies to it by looking at the record and its parent.

## How it is measured

`node scripts/measure-notation.mjs` reports, per directory, the files, the
characters, and the distinct names spelled with `_`, the distinct names spelled
with `-`, the distinct names holding an abbreviation, the distinct duplicated
blocks and the distinct repeated fields. The counts per scope (`data/seed`,
`data/meta`) are held to the ceilings of `data/meta/notation-ratchet.lino` by
the CI gate `check-notation`; they only fall. A count is of distinct items, so
a new record that reuses existing keys moves nothing, while a new `_` name
fails the gate. `--list underscore|abbreviated|duplicated|repeated` names
them, and `--write` lowers the ceilings after a rule pass.

## How it is applied

`node experiments/formal_ai_subagent/apply-notation-rules.mjs --family <name>`
applies the rename rule to one `family` of `data/meta/notation-rules.lino`:

1. The family's names are the `_` names of its files that occur in no other
   `.lino` file. A name shared with other files waits for the family that owns
   all of them; a name a shell, Python or workflow file spells waits for the
   family that moves those files too; a name the family keeps is listed under
   `keep` with its reason.
2. Each name is rewritten in the family's files and their byte mirrors
   (`rust/embedded/`), and in the code that reads them: as a token inside a
   string literal in JavaScript and Rust, inside a JavaScript regular
   expression as a pattern that accepts both spellings during the transition
   (`specialization[-_]ceiling`), and in comments and backticked documentation
   mentions. Code identifiers and Rust format arguments (`{handler_files}`) are
   never touched.
3. Every rename is recorded in `data/meta/notation-renames.lino`. The tool's
   `--check` (run by the gate `check-notation`) fails when a notation file or a
   reader literal holds an old spelling again.

`--family <name> --files --lines` shows every line a pass would change before
it is applied. After a pass, `node scripts/translate-es.mjs --write`
regenerates `ts/`, and `node scripts/measure-notation.mjs --write` lowers the
ceilings.
