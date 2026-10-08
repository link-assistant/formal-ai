#!/usr/bin/env node
// Generate the split system diagrams under docs/diagrams/ (issue #538, R382)
// from live data: the seed's handler precedence and promotions, the clap CLI
// definition and the server route manifest. The renderer is
// js/agentic/system_diagram.mjs (Rust twin rust/src/agentic_coding/
// system_diagram.rs); data/meta/system-diagrams.lino names the sources.
//
// Usage:
//   node scripts/generate-system-diagrams.mjs --write   regenerate the parts
//   node scripts/generate-system-diagrams.mjs --check   fail when a committed
//                                                       part drifts (CI gate)

import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import { hasHost, installHost } from '../js/agentic/host.mjs';
import { DIAGRAM_DIR, renderSystemDiagrams } from '../js/agentic/system_diagram.mjs';
import { REPO_ROOT, parseLino, readRepoFile } from '../js/server/lino.mjs';

const mode = process.argv[2];
if (mode !== '--write' && mode !== '--check') {
  console.error('usage: node scripts/generate-system-diagrams.mjs --write|--check');
  process.exit(2);
}
if (!hasHost()) installHost({ readText: readRepoFile, parseLino });

const read = (relative) => {
  try {
    return readFileSync(path.join(REPO_ROOT, relative), 'utf8');
  } catch {
    return '';
  }
};

let drift = 0;
for (const [file, text] of renderSystemDiagrams(readRepoFile)) {
  const relative = `${DIAGRAM_DIR}/${file}`;
  if (read(relative) === text) continue;
  if (mode === '--write') {
    writeFileSync(path.join(REPO_ROOT, relative), text);
    console.log(`wrote ${relative}`);
  } else {
    drift += 1;
    console.error(`::error file=${relative}::${relative} drifted from its generator; run node scripts/generate-system-diagrams.mjs --write`);
  }
}
if (drift) process.exit(1);
console.log(mode === '--check' ? 'system diagrams are current' : 'system diagrams written');
