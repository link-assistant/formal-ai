# cargo-audit 0.22.2 exit-status evidence (issue #1079, defect D2)

Lockfile as committed at main f971b8205 (chacha20 0.10.1):
```
      Loaded 1239 security advisories (from /home/box/.cargo/advisory-db)
    Scanning /tmp/Cargo.lock.bak for vulnerabilities (598 crate dependencies)
Crate:     fxhash
Version:   0.2.1
Warning:   unmaintained
Title:     fxhash - no longer maintained
Date:      2025-09-05
ID:        RUSTSEC-2025-0057
URL:       https://rustsec.org/advisories/RUSTSEC-2025-0057

Crate:     chacha20
Version:   0.10.1
Warning:   yanked

warning: 2 allowed warnings found
```

Exit status, lockfile at f971b8205:
  plain            -> exit 0
  --deny warnings  -> exit 1

Exit status, lockfile after `cargo update -p chacha20@0.10.1 --precise 0.10.2`:
  plain            -> exit 0
  --deny warnings  -> exit 1
