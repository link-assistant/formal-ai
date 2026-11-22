// R1188-U29: all assertions of the four native project-summary cases run
// against the actual production worker.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { REPO_ROOT, WorkerHost } from '../../../js/server/worker-host.mjs';
import { checkFailure, specificationCases } from '../../../scripts/lib/rust-specification-cases.mjs';

const NATIVE_CASE_NAMES = [
  'russian_hive_mind_prompt_prefers_link_assistant_project',
  'curated_project_concept_prompt_routes_to_project_lookup',
  'linksplatform_repository_is_promoted_by_default',
  'explicit_formal_ai_repository_url_still_routes_to_project_lookup',
];
const carried = specificationCases(REPO_ROOT).cases;
let host;
before(async () => { host = new WorkerHost(); await host.boot(); });

for (const [index, name] of NATIVE_CASE_NAMES.entries()) {
  test(`all native assertions: ${name}`, async () => {
    const selected = carried.filter(item => item.id.endsWith(`::${name}`));
    assert.equal(selected.length, 1, 'the original native test is carried exactly once');
    const item = selected[0];
    const responses = new Map();
    for (const ask of item.asks) responses.set(ask.binding, await host.solve(ask.prompt));
    assert.equal(item.checks.length, [4, 4, 4, 4][index], 'all 16 project-summary assertions are carried');
    for (const check of item.checks) assert.equal(checkFailure(check, responses), null, `${item.id}: ${JSON.stringify(check)}`);
  });
}

test('arbitrary records use the shared modes, stable weights and empty-summary behavior', async () => {
  const project = {
    displayName: 'Arbitrary project', description: 'Fallback description.', localized: [],
    statements: [
      { text: 'Install instructions.', kind: 'install', weight: 255 },
      { text: 'Highest statement.', kind: 'purpose', weight: 90 },
      { text: 'First tied statement.', kind: 'feature', weight: 70 },
      { text: 'Second tied statement.', kind: 'feature', weight: 70 },
      { text: 'Middle statement.', kind: 'feature', weight: 50 },
      { text: 'Low statement.', kind: 'misc', weight: 20 },
      { text: 'Zero statement.', kind: 'misc', weight: 0 },
    ],
  };
  for (const [mode, expected] of [
    ['short', 'Highest statement.'],
    ['standard', 'Highest statement. First tied statement. Second tied statement.'],
    ['full', 'Highest statement. First tied statement. Second tied statement. Middle statement. Low statement. Zero statement.'],
  ]) {
    assert.equal(await host.run('describeProjectRecord(__fixture, "en", __mode)', { __fixture: project, __mode: mode }), expected);
  }
  assert.equal(await host.run('describeProjectRecord(__fixture, "en", "short")', { __fixture: { ...project, statements: [] } }), '');
});

test('project statement weights follow the native complete unsigned-byte parse', async () => {
  const cases = [
    ['zero', '0', 0], ['upper', '255', 255], ['plus', ' +7 ', 7],
    ['partial', '7x', 50], ['negative', '-1', 50], ['overflow', '256', 50],
    ['missing', null, 50], ['empty', '', 50], ['leading-zero', '007', 7],
    ['scientific', '1e2', 50], ['rust-whitespace', '\u00857\u0085', 7],
    ['javascript-only-bom', '\uFEFF7\uFEFF', 50], ['negative-zero', '-0', 50],
  ];
  const fixture = 'project_fixture\n  org example-org\n  name arbitrary\n' + cases.map(([name, weight]) =>
    `  statement "${name}"\n    kind feature\n${weight === null ? '' : `    weight "${weight}"\n`}`).join('');
  const records = await host.run('self.FormalAiSeed.extractProjects(self.FormalAiSeed.parse(__fixture))', { __fixture: fixture });
  assert.equal(records.length, 1);
  assert.deepEqual(records[0].statements.map(item => [item.text, item.weight]), cases.map(([name, , expected]) => [name, expected]));
});


test('localized summaries retain native explicit-English and outer-statement fallback', async () => {
  const project = {
    statements: [{ text: 'Outer statement.', kind: 'purpose', weight: 10 }],
    localized: [
      { language: 'en', statements: [{ text: 'English statement.', kind: 'purpose', weight: 20 }] },
      { language: 'ru', statements: [{ text: 'Русское утверждение.', kind: 'purpose', weight: 30 }] },
    ],
  };
  assert.equal(await host.run('describeProjectRecord(__fixture, "ru", "short")', { __fixture: project }), 'Русское утверждение.');
  assert.equal(await host.run('describeProjectRecord(__fixture, "es", "short")', { __fixture: project }), 'English statement.');
  assert.equal(await host.run('describeProjectRecord(__fixture, "en", "short")', { __fixture: { ...project, localized: [{ language: 'en', statements: [] }] } }), 'Outer statement.');
});

test('an unavailable HTTP transport cannot substitute a registered project description', async () => {
  const isolated = new WorkerHost();
  await isolated.boot();
  const answer = await isolated.run(`(() => {
    PROJECTS.push({ org: "arbitrary-org", name: "arbitrary-project", displayName: "Arbitrary project", localized: [],
      statements: [{ text: "A registry description.", kind: "purpose", weight: 80 }] });
    fetch = undefined;
    return tryFetch("fetch https://github.com/arbitrary-org/arbitrary-project");
  })()`);
  assert.equal(answer.intent, 'http_fetch');
  assert.ok(answer.content.includes('HTTP fetch is not available'));
  assert.ok(!answer.content.includes('A registry description.'));
  assert.deepEqual(Array.from(answer.evidence), ['http_fetch:request:https://github.com/arbitrary-org/arbitrary-project']);
});

test('explicit fetch observes the requested repository paths regardless of promotion', async () => {
  const isolated = new WorkerHost();
  await isolated.boot();
  const targets = [
    ['https://github.com/link-assistant/hive-mind/issues/7', 'Issue data at the requested path.'],
    ['https://github.com/link-assistant/formal-ai/blob/main/README.md', 'Current README at the requested path.'],
    ['https://example.test/unregistered', 'Unregistered publisher response.'],
  ];
  const observation = await isolated.run(`(async () => {
    const requested = [];
    const replies = new Map(__targets);
    fetch = async (url) => {
      const href = new URL(String(url)).searchParams.get("url") || String(url);
      requested.push(href);
      return { ok: true, status: 200, headers: { get: () => "text/plain" }, text: async () => replies.get(href) };
    };
    const answers = [];
    for (const [url] of __targets) answers.push(await tryFetch("fetch " + url));
    return { requested, answers };
  })()`, { __targets: targets });
  assert.deepEqual(Array.from(observation.requested), targets.map(([url]) => url));
  for (const [index, [url, body]] of targets.entries()) {
    const answer = observation.answers[index];
    assert.equal(answer.intent, 'http_fetch');
    assert.ok(answer.content.includes(body));
    assert.ok(answer.evidence.includes(`http_fetch:request:${url}`));
    assert.ok(answer.evidence.includes('http_fetch:status:200'));
  }
});

test('promoted lookup retains actually fetched and fused browser search results and diagnostics', async () => {
  const requested = [];
  const providerUrl = 'https://example.test/independent-hive-mind';
  const isolated = new WorkerHost({ fetch: async (url) => {
    const href = String(url);
    if (href.startsWith('http://localhost')) {
      const relative = new URL(href).pathname.replace(/^\//, '');
      const disk = relative.startsWith('seed/') ? join(REPO_ROOT, 'data', relative) : join(REPO_ROOT, 'js', relative);
      return { ok: existsSync(disk), status: existsSync(disk) ? 200 : 404, text: async () => existsSync(disk) ? readFileSync(disk, 'utf8') : '' };
    }
    requested.push(href);
    if (new URL(href).hostname !== 'api.duckduckgo.com') throw new TypeError('Failed to fetch');
    const body = { Heading: 'Independent Hive Mind', AbstractURL: providerUrl, AbstractText: 'A separately fetched Hive Mind repository.', RelatedTopics: [] };
    return { ok: true, status: 200, headers: { get: () => 'application/json' }, text: async () => JSON.stringify(body), json: async () => body };
  } });
  const answer = await isolated.solve('What is Hive Mind?');
  assert.equal(answer.intent, 'project_lookup');
  assert.ok(requested.some(url => url.startsWith('https://api.duckduckgo.com/')));
  assert.ok(answer.content.includes(providerUrl), answer.content);
  assert.ok(answer.content.includes('A separately fetched Hive Mind repository.'), answer.content);
  assert.ok(answer.evidence.includes('web_search:provider_planned:duckduckgo'));
  assert.ok(answer.evidence.includes('web_search:fusion_planned:rrf:k=60'));
  assert.ok(answer.evidence.some(link => link.startsWith('web_search:fused:') && link.endsWith(providerUrl)));
  assert.ok(answer.diagnostics.fused.some(result => result.url === providerUrl));
});

test('project lookup and named summaries read repository metadata under the capture policy', async () => {
  const requested = [];
  const repository = 'https://github.com/link-assistant/hive-mind';
  const endpoint = 'https://api.github.com/repos/link-assistant/hive-mind';
  let description = 'Current description from the repository.';
  let identity = 'link-assistant/hive-mind';
  const isolated = new WorkerHost({ fetch: async (url) => {
    const href = String(url);
    if (href.startsWith('http://localhost')) {
      const relative = new URL(href).pathname.replace(/^\//, '');
      const disk = relative.startsWith('seed/') ? join(REPO_ROOT, 'data', relative) : join(REPO_ROOT, 'js', relative);
      return { ok: existsSync(disk), status: existsSync(disk) ? 200 : 404, text: async () => existsSync(disk) ? readFileSync(disk, 'utf8') : '' };
    }
    if (href !== endpoint) throw new TypeError('Unexpected external URL');
    requested.push(href);
    return { ok: true, status: 200, text: async () => JSON.stringify({ full_name: identity, html_url: repository, description }) };
  } });
  const lookup = () => isolated.solve('What is Hive Mind?');
  const summarize = (offline = false) => isolated.run('solve("Please summarize Hive Mind in one paragraph.", [], { offline: __offline })', { __offline: offline });
  const cold = await summarize(true);
  assert.equal(cold.intent, 'summarize_topic');
  assert.equal(cold.content, 'No verified repository description is available.');
  assert.equal(requested.length, 0);
  const live = await lookup();
  assert.equal(live.intent, 'project_lookup');
  assert.ok(live.content.includes(description), live.content);
  assert.ok(live.evidence.some(link => link.startsWith(`source:http:${endpoint} `)));
  assert.ok(live.evidence.includes('summarization:mode:short'));
  assert.deepEqual(requested, [endpoint]);
  const cached = await summarize(true);
  assert.equal(cached.content, description);
  assert.ok(cached.evidence.some(link => link.startsWith(`source:http:${endpoint} `) && link.endsWith('cached=true')));
  assert.deepEqual(requested, [endpoint]);

  await isolated.run('sourceWalkCaptureCache.clear()');
  description = 'Updated description from the same repository.';
  assert.equal((await summarize()).content, description);
  await isolated.run('sourceWalkCaptureCache.clear()');
  identity = 'another-org/another-repository';
  const rejected = await lookup();
  assert.ok(rejected.content.includes('No verified repository description is available.'));
  assert.ok(!rejected.content.includes(description));
  assert.ok(!rejected.evidence.some(link => link.startsWith(`source:http:${endpoint} `)));
  assert.deepEqual(requested, [endpoint, endpoint, endpoint]);
});

test('the production project registry carries identities without answer statements', async () => {
  const records = await host.run('PROJECTS.map(project => [project.url, project.statements.length, project.localized.map(entry => entry.statements.length)])');
  assert.ok(records.length > 1);
  for (const [url, statements, localized] of records) {
    assert.match(url, /^https:\/\/github\.com\//u);
    assert.equal(statements, 0, url);
    assert.ok(localized.every(count => count === 0), url);
  }
});


test('seeded project surfaces retain placeholder-looking metadata as literal bytes', async () => {
  const description = 'Source description contains {provider-summary} and {display-name}.';
  const body = await host.run('projectSummaryResponse("project-lookup-promoted", "en", __values)', {
    __values: { 'promoted-orgs': 'example-org', 'display-name': 'Literal {description}',
      'repo-slug': 'example-org/example', 'project-url': 'https://example.test/project',
      description, 'provider-summary': 'actual-provider', 'reciprocal-rank-constant': 60 },
  });
  assert.ok(body.includes('`Literal {description}`'));
  assert.ok(body.includes(description));
  assert.ok(body.includes('Providers: actual-provider.'));
});
