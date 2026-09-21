// @ts-check
// Issue #672 (F2): "Migration replay UI for partial profile transfers".
//
// The desktop app pins its userData directory and copies a legacy profile
// forward on first launch (issue #541, R3). Two things were wrong with that as
// shipped: the copied set omitted the stores Chromium keeps authentication
// state in (fixed in `desktop/lib/data-migration.cjs`, covered by
// `desktop/scripts/data-migration.test.mjs`), and the whole operation was
// invisible — `migrate()`'s result was discarded, so a user who lost their
// session had no way to know a transfer had happened, let alone ask for another
// pass.
//
// This spec covers the renderer half: the notice the desktop shell now shows,
// the replay button that drives `FormalAiDesktop.replayDataMigration()`, and
// the two ways the notice must stay out of the way — no bridge (web build) and
// nothing to report (a clean install).
//
// The `FormalAiDesktop` bridge is stubbed in an init script, which is how every
// desktop-surface spec in this suite reaches the Electron code path from the
// web-only CI job (see `issue-541-permissions.spec.js`).

const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';

const PREFERENCES = [
  'demo_preferences',
  '  demoMode "off"',
  '  greetingVariations "off"',
  '  diagnosticsMode "off"',
  '  uiLanguage "en"',
].join('\n');

const DESKTOP_STATUS = {
  shell: 'Electron',
  apiBase: '',
  staticBase: '',
  graphUrl: '',
  traceUrl: '',
  memory: 'formal_ai_bundle',
  agentModeDefault: false,
  toolCallPolicy: 'explicit-permission',
  apiReady: false,
};

/**
 * Boot the app with a desktop bridge whose migration channels answer with the
 * supplied fixtures. `replay` may be a list, in which case successive calls pop
 * the next entry — that is how the "replay changes the notice" case asserts the
 * transition rather than a static render.
 */
async function bootDesktop(page, { status, replay, omitChannels = false } = {}) {
  await page.addInitScript(
    ({ prefKey, preferences, desktopStatus, migrationStatus, replayResults, omit }) => {
      try {
        window.localStorage.setItem(prefKey, preferences);
      } catch (_error) {
        // localStorage can be unavailable in hardened browser contexts.
      }
      const calls = { status: 0, replay: 0 };
      window.__migrationCalls = calls;
      const bridge = {
        getStatus: async () => desktopStatus,
        ensureAgentServer: async () => desktopStatus,
        setToolGrants: async (grants) => ({ ...(grants || {}) }),
        invokeTool: async () => ({ ok: false, executed: false }),
        runAgentProvider: async () => ({ ok: false, executed: false }),
      };
      if (!omit) {
        bridge.dataMigrationStatus = async () => {
          calls.status += 1;
          return migrationStatus;
        };
        bridge.replayDataMigration = async () => {
          const index = Math.min(calls.replay, replayResults.length - 1);
          calls.replay += 1;
          return replayResults[index];
        };
      }
      window.FormalAiDesktop = bridge;
    },
    {
      prefKey: PREF_KEY,
      preferences: PREFERENCES,
      desktopStatus: DESKTOP_STATUS,
      migrationStatus: status || null,
      replayResults: replay && replay.length ? replay : [status || null],
      omit: omitChannels,
    },
  );
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
}

// What the main process reports after a real first-launch transfer.
const MIGRATED_STATUS = {
  known: true,
  migrated: true,
  reason: 'copied-legacy',
  copied: ['IndexedDB', 'Local Storage', 'Cookies', 'Service Worker'],
  migratedFrom: '/home/user/.config/formal-ai Desktop',
  version: 2,
  dataVersion: 2,
  error: null,
};

test.describe('Issue #672 (F2): desktop profile migration notice', () => {
  test('a completed transfer is reported with the profile it came from and what moved', async ({
    page,
  }) => {
    await bootDesktop(page, { status: MIGRATED_STATUS });

    const notice = page.locator('[data-testid="data-migration-notice"]');
    await expect(notice).toBeVisible();
    await expect(notice).toHaveAttribute('data-reason', 'copied-legacy');
    // The user is told WHERE the data came from — the leaf profile name, not
    // the absolute path, which is noise in a notice.
    await expect(page.locator('[data-testid="data-migration-body"]')).toContainText(
      'formal-ai Desktop',
    );
    // ...and exactly what moved, including the auth state F2 added. Without the
    // widened set this line would stop at "Local Storage" and the user would be
    // logged out with no explanation.
    const items = page.locator('[data-testid="data-migration-items"]');
    await expect(items).toContainText('Cookies');
    await expect(items).toContainText('Service Worker');
  });

  test('replay runs another pass and reports that nothing was left to copy', async ({
    page,
  }) => {
    await bootDesktop(page, {
      status: MIGRATED_STATUS,
      replay: [
        {
          known: true,
          migrated: false,
          reason: 'replayed',
          copied: [],
          migratedFrom: '/home/user/.config/formal-ai Desktop',
          version: 2,
          dataVersion: 2,
          error: null,
        },
      ],
    });

    const notice = page.locator('[data-testid="data-migration-notice"]');
    await expect(notice).toBeVisible();
    await page.locator('[data-testid="data-migration-replay"]').click();

    await expect(notice).toHaveAttribute('data-reason', 'replayed');
    await expect(page.locator('[data-testid="data-migration-body"]')).toContainText(
      'Nothing was left to transfer',
    );
    expect(await page.evaluate(() => window.__migrationCalls.replay)).toBe(1);
  });

  test('a replay that fills a gap lists the newly transferred entries', async ({ page }) => {
    await bootDesktop(page, {
      status: MIGRATED_STATUS,
      replay: [
        {
          known: true,
          migrated: true,
          reason: 'replayed',
          copied: ['Cookies'],
          migratedFrom: '/home/user/.config/formal-ai Desktop',
          version: 2,
          dataVersion: 2,
          error: null,
        },
      ],
    });

    await page.locator('[data-testid="data-migration-replay"]').click();
    await expect(page.locator('[data-testid="data-migration-items"]')).toHaveText(
      'Transferred: Cookies',
    );
  });

  test('a failed transfer surfaces the error instead of claiming success', async ({
    page,
  }) => {
    await bootDesktop(page, {
      status: {
        known: true,
        migrated: false,
        reason: 'failed',
        copied: [],
        migratedFrom: null,
        error: 'EACCES: permission denied',
      },
    });

    const notice = page.locator('[data-testid="data-migration-notice"]');
    await expect(notice).toBeVisible();
    await expect(notice).toHaveClass(/is-failed/);
    await expect(page.locator('[data-testid="data-migration-body"]')).toContainText(
      'EACCES: permission denied',
    );
  });

  test('dismissing hides the notice for the rest of the session', async ({ page }) => {
    await bootDesktop(page, { status: MIGRATED_STATUS });

    const notice = page.locator('[data-testid="data-migration-notice"]');
    await expect(notice).toBeVisible();
    await page.locator('[data-testid="data-migration-dismiss"]').click();
    await expect(notice).toHaveCount(0);
  });

  test('a clean install is never interrupted by the notice', async ({ page }) => {
    // Nothing to migrate is the overwhelmingly common case: showing a banner
    // about it would be pure noise on every fresh install.
    await bootDesktop(page, {
      status: {
        known: true,
        migrated: false,
        reason: 'no-legacy-data',
        copied: [],
        migratedFrom: null,
        error: null,
      },
    });
    await expect(page.locator('[data-testid="mode-status"]')).toBeVisible();
    await expect(page.locator('[data-testid="data-migration-notice"]')).toHaveCount(0);
  });

  test('the web build, which has no bridge at all, never renders the notice', async ({
    page,
  }) => {
    await page.addInitScript(
      ({ prefKey, preferences }) => {
        try {
          window.localStorage.setItem(prefKey, preferences);
        } catch (_error) {
          // localStorage can be unavailable in hardened browser contexts.
        }
      },
      { prefKey: PREF_KEY, preferences: PREFERENCES },
    );
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="data-migration-notice"]')).toHaveCount(0);
  });

  test('an older desktop build without the migration channels degrades quietly', async ({
    page,
  }) => {
    // The renderer ships independently of the shell, so it must tolerate a
    // bridge that predates these channels rather than throwing during boot.
    const errors = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await bootDesktop(page, { status: MIGRATED_STATUS, omitChannels: true });
    await expect(page.locator('[data-testid="desktop-shell-status"]')).toBeVisible();
    await expect(page.locator('[data-testid="data-migration-notice"]')).toHaveCount(0);
    expect(errors, 'boot must not throw without the migration channels').toEqual([]);
  });
});
