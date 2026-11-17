// Issue #1159 (sub-issue of #1162), R1159-4: the delivered workflow sets up
// the toolchain the recovery verified with. JavaScript twin of
// `the_scala_workflow_installs_scalac_from_the_release_feed_of_the_recovery_publisher`
// in rust/tests/unit/issue_1159_toolchain_bootstrap.rs: the Scala workflow's
// compiler pin is resolved from the same release feed the pinned `scalac`
// publisher record names, and the workflow installs `scalac` at that version
// and runs the verified commands.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { readText } from '../../../js/agentic/host.mjs';
import { renderWith } from '../../../js/agentic/ci_workflow.mjs';
import { baselineVersionSet } from '../../../js/agentic/crate/version_resolution.mjs';
import { programContractAnswer } from '../../../js/agentic/crate/coding_program_contract.mjs';
import { findChildValue, parseRoot } from '../../../js/agentic/crate/seed_parser.mjs';

const PUBLISHER = 'https://api.github.com/repos/scala/scala3/releases/latest';

before(async () => {
  await installNodeHost(new WorkerHost());
});

test('the Scala workflow installs scalac from the release feed of the recovery publisher (R1159-4)', () => {
  const publisher = parseRoot(readText('data/seed/setup-publishers.lino')).children
    .find(node => findChildValue(node, 'program') === 'scalac');
  assert.ok(publisher, 'scalac has a pinned publisher');
  assert.equal(findChildValue(publisher, 'documentation'), PUBLISHER);
  const versions = baselineVersionSet();
  assert.ok(versions, 'the toolchains seed carries every baseline');
  assert.equal(versions.scala.source_url, PUBLISHER, 'the workflow\'s Scala version and the recovery come from one publisher record');
  const recipe = programContractAnswer('Write a Scala program that prints "Hello, World!" to stdout.').execution_recipe;
  assert.ok(recipe, 'a recipe');
  const workflow = renderWith(recipe, versions);
  const scala = versions.scala.tag;
  assert.ok(workflow.includes(`apps: scala:${scala} scalac:${scala}\n`), workflow);
  for (const command of recipe.commands) {
    assert.ok(workflow.includes(`      - run: |\n          ${command}\n`), `the workflow runs the verified command \`${command}\`: ${workflow}`);
  }
});
