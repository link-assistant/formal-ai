// @ts-check
const { existsSync, readFileSync } = require('node:fs');
const path = require('node:path');
const { defineConfig, devices } = require('@playwright/test');

const PORT = process.env.E2E_PORT || 3456;
const ORIGIN = `http://localhost:${PORT}`;
// The web app moved from / to /app/ (issue #479); the site root is now the
// landing page. Pointing baseURL at /app/ keeps every relative goto('./') in
// the app specs aimed at the app, while absolute paths like /download/ and
// relative ../tests/ continue to reach their siblings unchanged.
const BASE_URL = `${ORIGIN}/app/`;

// The spec files of the local suite.
const TEST_MATCH = [
  '**/demo.spec.js',
  '**/multilingual-*.spec.js',
  '**/connectivity.spec.js',
  '**/playwright-script-request.spec.js',
  '**/issue-667-debugger-view.spec.js',
  '**/creator-question.spec.js',
  '**/search-menu-and-deduplication.spec.js',
  '**/offline-bundled-runtime.spec.js',
  '**/ocr-image-attachments.spec.js',
  '**/prime-proof-prompts.spec.js',
  '**/quoted-russian-translation.spec.js',
  '**/deformalize-step.spec.js',
  '**/apple-translation.spec.js',
  '**/common-noun-translation.spec.js',
  '**/pandas-join-documentation.spec.js',
  '**/implicit-research-web-search.spec.js',
  '**/enumeration-research-web-search.spec.js',
  '**/search-phrase-translation.spec.js',
  '**/dictionary-lookup-recovery.spec.js',
  '**/desktop-shell-bridge.spec.js',
  '**/rust-wasm-worker-parity.spec.js',
  '**/cross-runtime-synthesis-parity.spec.js',
  '**/antiregime-concept-lookup.spec.js',
  '**/false-totality-concept-lookup.spec.js',
  '**/code-highlighting-and-copy.spec.js',
  '**/fibonacci-agent-plan.spec.js',
  '**/composite-wikipedia-research.spec.js',
  '**/compound-interest-conversion.spec.js',
  '**/github-repository-extraction.spec.js',
  '**/relational-box-arithmetic.spec.js',
  '**/research-table-follow-up.spec.js',
  '**/how-to-typo-correction.spec.js',
  '**/download-page.spec.js',
  '**/vscode-extension-bridge.spec.js',
  '**/write-program-diagnostics.spec.js',
  '**/reasoning-first-report.spec.js',
  '**/trimmed-issue-report.spec.js',
  '**/adaptive-header-and-dark-theme.spec.js',
  '**/activation-safe-copy.spec.js',
  '**/free-time-small-talk.spec.js',
  '**/calendar-event-request.spec.js',
  '**/toolbar-icon-packs.spec.js',
  '**/records-research-web-search.spec.js',
  '**/relative-date-calendar.spec.js',
  '**/desktop-services-panel.spec.js',
  '**/length-versus-mass-units.spec.js',
  '**/file-listing-and-light-code-theme.spec.js',
  '**/mixed-script-wikipedia-lookup.spec.js',
  '**/train-meeting-word-problem.spec.js',
  '**/clock-time-duration.spec.js',
  '**/authorship-fact-query.spec.js',
  '**/sidebar-section-isolation.spec.js',
  '**/neural-inference-concept.spec.js',
  '**/macos-gatekeeper-screenshots.spec.js',
  '**/issue-479-site.spec.js',
  '**/telegraphic-how-to.spec.js',
  '**/elided-procedural-how-to.spec.js',
  '**/visible-thinking-preview.spec.js',
  '**/ocr-market-price-check.spec.js',
  '**/repository-traffic-prompt.spec.js',
  '**/unresolved-term-web-search.spec.js',
  '**/install-how-to-discovery.spec.js',
  '**/issue-511-cold-start.spec.js',
  '**/terminal-command-mode.spec.js',
  '**/tool-permissions-and-approval.spec.js',
  '**/agent-cli-chat-rendering.spec.js',
  '**/text-attachment-originality.spec.js',
  '**/issue-541-demo-mode.spec.js',
  '**/issue-541-permissions.spec.js',
  '**/issue-541-theme.spec.js',
  '**/desktop-updates-and-version.spec.js',
  '**/issue-550-chakra-migration.spec.js',
  '**/issue-554-site.spec.js',
  '**/repository-lookup-language-follow-up.spec.js',
  '**/issue-672-theme-snapshots.spec.js',
  '**/issue-672-animation-override.spec.js',
  '**/issue-672-reasoning-hierarchy.spec.js',
  '**/issue-672-migration-replay.spec.js',
  '**/issue-541-permissions-cold-start.spec.js',
  '**/issue-676-thinking-narrative.spec.js',
  '**/natural-language-settings-control.spec.js',
  '**/computer-use-permissions.spec.js',
  '**/conversational-requirement-recovery.spec.js',
  '**/ranked-provenance-answers.spec.js',
  '**/browser-memory-programs.spec.js',
  '**/desktop-web-search-without-agent.spec.js',
  '**/desktop-agent-selector.spec.js',
  '**/source-first-translation.spec.js',
  '**/dialogue-fact-checking.spec.js',
  '**/failure-detection-and-report-offer.spec.js',
  '**/research-fusion-routing.spec.js',
  '**/proof-program-translation.spec.js',
  '**/published-search-boundaries.spec.js',
  '**/formal-language-projections.spec.js',
  '**/reasoning-and-panel-polish.spec.js',
];

// R1188-U10: long specs start first. A spec file whose tests took at least
// this many seconds in CI (data/meta/playwright-test-durations.lino, recorded
// by `node experiments/formal_ai_subagent/ci-durations.mjs --playwright-files
// --write`) runs in the `chromium-long` project. Playwright dispatches the
// test groups of the projects in their order here, so those specs start
// before the rest. The legs themselves are planned longest-first by
// scripts/plan-test-shards.mjs in .github/workflows/e2e-local.yml.
const LONG_SPEC_SECONDS = 60;

/** Glob patterns of the recorded long spec files that still exist. */
function longSpecs() {
  let text = '';
  try {
    text = readFileSync(path.join(__dirname, '../../../data/meta/playwright-test-durations.lino'), 'utf8');
  } catch {
    return [];
  }
  const long = [];
  let spec = null;
  for (const line of text.split('\n')) {
    const test = /^ {2}test "tests\/([^"]+)"$/.exec(line);
    const seconds = /^ {4}seconds (\d+(?:\.\d+)?)$/.exec(line);
    if (test) {
      spec = test[1];
    } else if (seconds && spec && Number(seconds[1]) >= LONG_SPEC_SECONDS) {
      long.push(`**/${spec}`);
    }
  }
  return long.filter((pattern) => existsSync(path.join(__dirname, 'tests', pattern.slice('**/'.length))));
}

const LONG_SPECS = longSpecs();

module.exports = defineConfig({
  testDir: './tests',
  testMatch: TEST_MATCH,
  // Per-test cap. A single app spec navigates, waits for the worker to boot,
  // and asserts on one answer — comfortably under 30s even on a cold worker.
  timeout: 30_000,
  // Whole-suite cap so a hung worker or server can never wedge CI indefinitely;
  // it aborts the run instead of waiting for the job-level kill.
  //
  // Issue #977: this was 15 minutes -- exactly the `timeout-minutes: 15` of the
  // `E2E Tests (local web app)` job, which also has to pay for checkout, bun
  // install, the web bundle build, `npm ci` and the browser install. The job
  // clock therefore always ran out first, and a job killed by `timeout-minutes`
  // is reported as **cancelled**, not failed: run 31073507682 died at test
  // 159/468 and the pipeline showed a green-ish "cancelled" instead of a red
  // failure. `if: failure()` never fired either, so no Playwright report was
  // uploaded. The job cap is now 40 minutes, and this one is deliberately kept
  // well below the remaining budget so *Playwright* aborts first, exits
  // non-zero, and leaves a report behind.
  //
  // R1188-U9/U11: the suite now runs as three `--shard` legs of a 30-minute
  // job, so each leg's Playwright aborts at 20 minutes, before the job clock.
  globalTimeout: 20 * 60_000,
  // Issue #977: the suite is 468 tests. Playwright's default is half the
  // available cores (2 on a 4-vCPU ubuntu-latest runner), which left the suite
  // unable to finish in any reasonable budget. These specs are I/O-bound
  // (navigate, wait for the wasm worker, assert), so one worker per vCPU is the
  // right trade. Locally the default is kept so a dev machine is not saturated.
  workers: process.env.CI ? 4 : undefined,
  // R1188-U11: parallel at test level, not only by file: a spec file's tests
  // spread over the workers. The two files whose tests share state (a server
  // or screenshots made in beforeAll) opt back to `mode: 'default'`.
  fullyParallel: true,
  // Fail individual web-first assertions fast (default is 5s) so flakes surface
  // quickly rather than each burning the full per-test budget.
  expect: { timeout: 10_000 },
  // Issue #672 (F1): snapshot baselines live in a single reviewable directory
  // next to the specs. The default template appends `{-projectName}` and a
  // platform suffix, which would fork one baseline per OS — pointless for the
  // computed-colour tables this suite snapshots (CSS colours do not vary by
  // platform) and a trap for contributors on macOS whose run would silently
  // write a second baseline instead of failing against the committed one.
  snapshotPathTemplate: '{testDir}/__snapshots__/{testFileName}/{arg}{ext}',
  retries: 1,
  reporter: [['html', { open: 'never' }], ['list']],
  use: {
    baseURL: BASE_URL,
    trace: 'on-first-retry',
    // Bound navigation/action waits so a stuck page errors promptly.
    navigationTimeout: 15_000,
    actionTimeout: 10_000,
    // Issue #541 (R5/R6): freshly produced assistant messages stage a reasoning-
    // then-body reveal that hides the answer body via `.is-revealing { display:
    // none }` for the configured animation budget (default 2 s). Headless tests
    // read `innerText()` immediately, which would return an empty string during
    // that window and flake. Emulating prefers-reduced-motion makes
    // `usePrefersReducedMotion()` return true, which short-circuits
    // `useMessageReveal` to "show everything at once" — matching what users with
    // reduced-motion preferences see, and giving tests deterministic text.
    reducedMotion: 'reduce',
  },
  webServer: {
    // The seed mirror under js/seed/ is generated from the canonical
    // data/seed/ tree on every server start so we never serve stale data.
    command:
      `bun run --cwd ../../.. build:web && ../../../scripts/sync-seed.sh && npx serve ../../../js --listen ${PORT} --no-clipboard`,
    url: ORIGIN,
    reuseExistingServer: false,
    timeout: 15_000,
  },
  projects: [
    {
      name: 'chromium-long',
      testMatch: LONG_SPECS,
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'chromium',
      testIgnore: LONG_SPECS,
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
