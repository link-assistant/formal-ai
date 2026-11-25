#!/usr/bin/env python3
"""Mark intentional repeated `#[path]` module inclusions with
`#[allow(clippy::duplicate_mod)]` (repo convention, see rust/tests/unit/mod.rs).

Usage: python3 experiments/issue-1637/add-duplicate-mod-allow.py <basename>... -- <file>...
Rust 1.100's clippy reports each shared fixture/script helper included by more
than one module in the same test crate; the lint is silenced only when all but
one inclusion carry the allow, so every inclusion is marked.
"""
import re, sys
sep = sys.argv.index('--')
names, files = set(sys.argv[1:sep]), sys.argv[sep+1:]
for f in files:
    lines = open(f).read().split('\n')
    out, changed = [], False
    for line in lines:
        m = re.match(r'\s*#\[path\s*=\s*"([^"]+)"\]', line)
        if m and m.group(1).split('/')[-1] in names or (m and m.group(1) in names):
            prev = out[-1] if out else ''
            if 'duplicate_mod' not in prev:
                indent = line[:len(line)-len(line.lstrip())]
                pm = re.match(r'(\s*)#\[allow\((.*)\)\]\s*$', prev)
                if pm:
                    out[-1] = f'{pm.group(1)}#[allow({pm.group(2)}, clippy::duplicate_mod)]'
                else:
                    out.append(f'{indent}#[allow(clippy::duplicate_mod)]')
                changed = True
        out.append(line)
    if changed:
        open(f, 'w').write('\n'.join(out))
        print('updated', f)
