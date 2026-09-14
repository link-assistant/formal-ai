# Issue 1014 nextest archive relocation experiment

`run.sh` builds a tiny integration test that launches a binary through Cargo's
compile-time `CARGO_BIN_EXE_*` path. It then removes the original target tree
and compares nextest's default temporary extraction with extraction into the
original workspace. The experiment verifies whether `--extract-to` alone can
make legacy compile-time paths portable without changing every test caller.

Pass the path to a `cargo-nextest` executable as the first argument.
