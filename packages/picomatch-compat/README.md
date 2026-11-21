# Picomatch security backport

This private CommonJS package retains picomatch 4.0.7's published source and MIT
license, with one local parser change for
[GHSA-fc5q-39xc-vjpf](https://github.com/advisories/GHSA-fc5q-39xc-vjpf).
The upstream tarball identity and per-file hashes are in upstream.json.
All original source files except lib/parse.js are unchanged. Its repeated-extglob
helpers are extracted into lib/extglob-guard.js to keep both files below the
repository warning threshold. Added source identity is recorded separately.
This is a local backport,
not an upstream patched release.

The existing repeated-extglob guard skips single branches such as +(*) and
+(*a). A wildcard nested under unbounded repetition can consume exponential
time even for a 42-byte subject. The backport extends that guard to every branch
count, while retaining its safe character-class flattening. Escaped stars,
bracket-class stars (including POSIX negation), ordinary globs and literal
repetitions remain supported. With unescape:true, an escaped star that would
become a raw regex quantifier is treated as risky as well.

Risky repeated wildcards now match their literal spelling through the existing
fallback. The upstream maxExtglobRecursion: false option still disables the guard;
callers processing untrusted patterns must retain the default protection.
This does not make arbitrary raw regular-expression features safe.

The repository's `rust/tests/web/picomatch-compat.test.mjs` checks the pinned
sources, the lockfiles for both consumers and ordinary glob behavior. Remove
the local override when an upstream release fixes the advisory.
