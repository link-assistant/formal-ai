# Architect's notes

The architect's (konard's) own statements about the system, in chronological
order, one file per note. Quoted, never paraphrased.

> we should start keep track of architect's (me) notes on the system progress.
> You can restore them from previous issues and GitHub comments, that were
> clearly written in the same style. We should not invent the language or
> terminology I'm not using.

Two rules for this folder:

- **Do not invent terminology the architect does not use.** A note records his
  words with a reference to where he wrote them, and nothing else.
- **The vision itself lives in [`VISION.md`](../../VISION.md).** These notes are
  the record of how it was stated over time; `VISION.md` is the standing
  statement of it and is updated from the latest note. Where a document, gate,
  requirement or plan contradicts the latest note, the document is wrong and
  must be fixed. The mechanically checkable part of that rule is enforced by
  the `docs_issue_citations` gate (`scripts/check-issue-citations.rs`) and the
  benchmark ledger parity test (`rust/tests/unit/docs_benchmarks.rs`); the rest is
  this folder's standing instruction to every contributor.
- **Note bodies are frozen at their date.** Numbers and states inside a note
  describe the moment the note was written and are deliberately not updated;
  a note that records current state does so under a "State when this was
  written" heading, and the standing statement stays in `VISION.md`.

## Notes, oldest first

| Date | Note | Subject |
| --- | --- | --- |
| 2026-02-?? | [Deduplication is how invariants are found](2026-02-00-deduplication-finds-invariants.md) | Repetition, compression and intelligence (issue #531) |
| 2026-??-?? | [Translate the whole source to meta language and back](2026-00-00-translate-source-to-meta-language.md) | Auto-learning; source in the seed data (issue #558) |
| 2026-09-11 | [The goal is the meta algorithm, not a kernel](2026-09-11-the-goal-is-the-meta-algorithm.md) | Meta algorithm; meta-language representation; approval of self-modification; deduplication |
| 2026-09-11 | [.lino and /src must be 1 to 1 on every merge](2026-09-11-lino-and-src-one-to-one.md) | Full representation of Rust code in `.lino` |
| 2026-09-11 | [Do not obstruct progression to the vision](2026-09-11-do-not-obstruct-the-vision.md) | Relax requirements after a day; releases for Hive Mind |
| 2026-09-11 | [Start from a working Hello World](2026-09-11-start-from-hello-world.md) | Top 10-20 languages; branches, not repositories |
| 2026-09-12 | [Nothing is a hard task](2026-09-12-nothing-is-a-hard-task.md) | No rating, judgement or assessment of a given task |
| 2026-09-12 | [Notes are notes; the vision is VISION.md](2026-09-12-notes-are-notes-vision-is-vision.md) | Naming; this folder's existence |
| 2026-09-14 | [Know how to get to know anything when it is needed](2026-09-14-know-how-to-get-to-know-anything.md) | Dynamic discovery, trusted sources, rediscoverable knowledge, and retained experience |
| 2026-09-24 | [Three roots, full parity, via the meta language](2026-09-24-three-roots-full-parity-via-the-meta-language.md) | Full js/ts/rust parity for client and backend; translation in any direction (PR #1139) |
| 2026-09-25 | [Deliver the requirements in code, not memories](2026-09-25-deliver-the-requirements-in-code.md) | Compile every directive into one file; deliver them as code (PR #1139) |
