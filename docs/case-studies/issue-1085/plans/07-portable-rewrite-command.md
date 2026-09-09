# Plan 07 -- the rename recipe must work on macOS (#1110)

`repeated_identifier_rewrite_command` emits GNU sed:

```
sed -i 's/\bUNKNOWN_INTENT\b/UNKNOWN_INTENT_NAME/g' -- src/x.rs
```

On BSD sed (macOS) `-i` consumes the script as its suffix argument, and `\b` is
not a word boundary at all. Measured locally on ladder leaf 2.2.2.2.1: the
command failed, the file was unchanged, and Formal AI reported
`Verification failed ... the observed bytes differ from the planned workspace
effect`. A Mac user asking for a rename gets a confident failure.

## Steps

- [x] 1. `repeated_identifier_rewrite_command` emits `perl -pi -e` instead:
      `perl -pi -e 's/\bX\b/Y/g' -- FILE`. `\b` and in-place editing behave
      identically on GNU and BSD systems, and perl is present on both the
      Ubuntu runners and macOS. Escaping: the identifiers are already
      constrained to `[A-Za-z0-9_]` by `shell_safe_identifier`, so neither
      side of the substitution can carry a metacharacter.
- [x] 2. Move the two pinned strings with it:
      `tests/unit/issue_848_coding_ladder.rs` and
      `tests/unit/issue_1069_ladder_change_tasks.rs`.
- [x] 3. A test that the emitted command is portable: no `sed -i` without a
      suffix, no `\b` inside a `sed` script.
- [x] 4. Verify by running the ladder leaf locally on macOS and watching it
      pass where it failed.
