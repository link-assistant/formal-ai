# Picomatch repeated-wildcard backport

The dependency gate found GHSA-fc5q-39xc-vjpf after its 17 November review.
The registry and GitHub release metadata both still identify 4.0.7 as latest;
the advisory has no patched release. The published tarball is integrity checked,
with its MIT license and eight source hashes retained in the local package.
No advisory exception, omitted dependency class or higher severity threshold is used.

The existing guard handles dangerous multi-branch repeated extglobs but misses a
single wildcard branch. The original-source run has eight failures and three
passes, including two hostile matches killed after 1.25 seconds. Each child has
a 64 MiB heap, 256 KiB stack and finite 42-byte subjects.

The first candidate passed eleven cases. Further parser review found two
bracket bypasses: +([[]*a) and +([*a). The retained edge-before run reproduces
both failures. The guard now follows non-nesting character classes, recognized
POSIX classes, escaped/quoted stars and disabled bracket parsing; dangerous
patterns use the existing literal fallback. The expanded focused file also
checks actual locked resolution and per-file provenance.

The same installed-consumer probe passes on published 4.0.7 and the backport:
micromatch 4.0.8, tinyglobby 0.2.17 and fdir 6.5.0. Its initial failure was an
incorrect expectation that fdir omits dotfiles; the upstream control confirms
fdir includes them. Both full dependency-class npm audits report zero
vulnerabilities, and npm pack includes all eleven intended files.

This is a local compatibility backport, not an upstream release or proof that
arbitrary regular expressions are safe. The existing explicit
maxExtglobRecursion:false opt-out remains. Supplied code and commands earn
zero autonomous Formal AI authorship credit. Final gate results are recorded
separately; native execution is not claimed.
