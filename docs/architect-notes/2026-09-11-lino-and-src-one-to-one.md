# `.lino` and `/src` must be 1 to 1 on every merge

**Date:** 2026-09-11 · **Source:** direct statement to the assistant

> We must have .lino files containing full representation of rust code, and we
> must have /src folder with Rust code all 1 to 1 on each pull request merged.

## State when this was written

1-to-1 file correspondence holds (516 of 516). **Full representation does not**:
the committed census is a signature, not the source. A lossless round-trip
(`source → links → source`, byte for byte, 100% of the repository) exists and
passes in `self_source_links.rs`, but the test is `#[ignore]`d, so CI never runs
it and the representation it proves is never committed.
