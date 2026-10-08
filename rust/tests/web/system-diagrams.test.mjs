// The generated system diagrams (issue #538, R382): js/agentic/system_diagram.mjs
// renders docs/diagrams/{system-overview,solver-handlers,cli-subcommands,
// http-routes}.md from live data, and scripts/generate-system-diagrams.mjs
// --check is the CI drift gate. Twin of rust/tests/unit/agentic-coding/system_diagrams.rs.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { describe, it } from 'node:test';

import { hasHost, installHost } from '../../../js/agentic/host.mjs';
import {
  CONFIG_PATH, DIAGRAM_DIR, cliDefinition, precedenceRows, promotionRows, renderSystemDiagrams, routeRows,
} from '../../../js/agentic/system_diagram.mjs';
import { REPO_ROOT, parseLino, readRepoFile } from '../../../js/server/lino.mjs';
import { serverRoutes } from '../../../js/server/routes.mjs';

if (!hasHost()) installHost({ readText: readRepoFile, parseLino });

const part = (file) => renderSystemDiagrams(readRepoFile).find(([name]) => name === file)[1];

describe('system diagrams (rust/tests/unit/agentic-coding/system_diagrams.rs)', () => {
  it('committed_parts_are_the_generated_parts', () => {
    const parts = renderSystemDiagrams(readRepoFile);
    assert.deepEqual(parts.map(([file]) => file),
      ['system-overview.md', 'solver-handlers.md', 'cli-subcommands.md', 'http-routes.md']);
    for (const [file, text] of parts) assert.equal(readRepoFile(`${DIAGRAM_DIR}/${file}`), text, file);
  });

  it('every_handler_appears_in_precedence_order', () => {
    const text = part('solver-handlers.md');
    const seed = readRepoFile('data/seed/handler-precedence.lino');
    const names = [...seed.matchAll(/^ {2}handler (\S+)$/gm)].map((match) => match[1]);
    assert.ok(names.length > 50, String(names.length));
    assert.deepEqual(new Set(precedenceRows(seed).map((row) => row.name)), new Set(names));
    let last = -1;
    for (const row of precedenceRows(seed)) {
      const at = text.indexOf(`h_${row.name}["${row.rank}: ${row.name}"]`);
      assert.ok(at > last, `${row.name} is drawn in precedence order`);
      last = at;
      const surface = row.browserOnly ? 'browser worker only' : 'native and browser';
      assert.ok(text.includes(`| ${row.rank} | \`${row.name}\` | ${surface} |`), row.name);
    }
  });

  it('every_promotion_appears_in_rank_order', () => {
    const text = part('solver-handlers.md');
    const seed = readRepoFile('data/seed/handler-promotions.lino');
    const names = [...seed.matchAll(/^ {2}promotion (\S+)$/gm)].map((match) => match[1]);
    const rows = promotionRows(seed);
    assert.equal(rows.length, names.length);
    let last = -1;
    for (const row of rows) {
      const at = text.indexOf(`["${row.rank}: ${row.handler}"]`);
      assert.ok(at > last, `${row.handler} keeps rank order`);
      last = at;
    }
  });

  it('every_route_and_path_appears', () => {
    const text = part('http-routes.md');
    const routes = serverRoutes();
    assert.deepEqual(routeRows(readRepoFile('data/meta/server-routes.lino')).map((row) => row.id),
      routes.map((route) => route.id));
    for (const route of routes) {
      assert.ok(text.includes(`r_${route.id}["`), route.id);
      for (const path of route.paths) assert.ok(text.includes(`\`${path}\``), path);
    }
  });

  it('every_subcommand_appears', () => {
    const text = part('cli-subcommands.md');
    const source = readRepoFile('rust/src/main.rs');
    const cli = cliDefinition(source);
    assert.equal(cli.binary, 'formal-ai');
    const body = source.slice(source.indexOf('enum Command {'));
    const variants = [...body.slice(0, body.indexOf('\n}\n')).matchAll(/^ {4}([A-Z]\w*)/gm)].map((match) => match[1]);
    assert.equal(cli.subcommands.length, variants.length);
    for (const sub of cli.subcommands) {
      assert.ok(text.includes(`| \`formal-ai ${sub.name}\` |`), sub.name);
      assert.ok(text.includes(`cli --> c_${sub.name.replace(/-/g, '_')}["${sub.name}"]`), sub.name);
    }
    assert.ok(cli.subcommands.some((sub) => sub.name === 'shared-dialog'));
  });

  it('an_edge_naming_a_vanished_subcommand_fails_the_renderer', () => {
    const broken = (relative) => {
      const text = readRepoFile(relative);
      return relative === CONFIG_PATH ? text.replace('subcommand chat', 'subcommand vanished') : text;
    };
    assert.throws(() => renderSystemDiagrams(broken),
      { message: 'system_diagrams:edge:cli_chat:unknown_subcommand:vanished' });
  });

  it('a_new_seed_handler_reaches_the_diagram', () => {
    const extended = (relative) => {
      const text = readRepoFile(relative);
      return relative === 'data/seed/handler-precedence.lino'
        ? `${text}  handler diagram_probe_handler\n    rank 999999\n`
        : text;
    };
    const handlers = renderSystemDiagrams(extended).find(([file]) => file === 'solver-handlers.md')[1];
    assert.ok(handlers.includes('h_diagram_probe_handler["999999: diagram_probe_handler"]'));
  });

  it('the --check gate passes on the committed parts', () => {
    const out = execFileSync(process.execPath, ['scripts/generate-system-diagrams.mjs', '--check'],
      { cwd: REPO_ROOT, encoding: 'utf8' });
    assert.match(out, /current/);
    assert.equal(readFileSync(`${REPO_ROOT}/${DIAGRAM_DIR}/system-overview.md`, 'utf8'), part('system-overview.md'));
  });
});
