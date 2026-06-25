// @ts-check
// Issue #550 (M3): "must fully transition to https://chakra-ui.com and JSX."
//
// The web app source is now src/web/app/main.jsx — authored in modern JSX and
// bundled to src/web/app.js by the bun bundler (`build:web`). The app mounts
// under <ChakraProvider>, renders through Chakra's chakra.* styled factory, and
// is themed by the --fa-* → Chakra semanticTokens bridge in theme.js.
//
// These tests load the *real* shipped app (app.js, served from /app/) and assert
// the migration is live end-to-end, so the Chakra/JSX runtime cannot silently
// regress back to hand-written React.createElement without the bundle.
const { test, expect } = require('@playwright/test');

test.describe('Issue #550 (M3): the app is migrated to Chakra UI + JSX', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    // The bun-built bundle renders asynchronously; wait for the first node.
    await page.waitForSelector('#root *', { timeout: 15_000 });
  });

  test('React mounts the app from the bun-built JSX bundle', async ({ page }) => {
    // #root is populated only if createRoot(...).render(<App/>) ran, i.e. the
    // JSX in main.jsx compiled to a working app.js via the bun bundler.
    const rootChildren = await page.evaluate(
      () => document.getElementById('root')?.childElementCount ?? 0,
    );
    expect(rootChildren).toBeGreaterThan(0);
  });

  test('the Chakra/Emotion CSS-in-JS runtime is live (styles in JavaScript)', async ({
    page,
  }) => {
    // Chakra is built on Emotion; once <ChakraProvider> mounts and any chakra.*
    // element renders, Emotion injects <style data-emotion> tags into the head.
    // Their presence is direct proof the Chakra runtime shipped — the exact claim
    // an earlier draft wrongly called "CSP-blocked" (the app page carries no CSP).
    const emotionStyles = await page.locator('style[data-emotion]').count();
    expect(emotionStyles).toBeGreaterThan(0);
  });

  test('the consolidated ToolbarButton renders topbar controls through chakra.*', async ({
    page,
  }) => {
    // ToolbarButton (one component for all 11 topbar controls) renders chakra.a
    // for links and chakra.button otherwise, preserving the original className /
    // data-testid contract so styles.css stays authoritative.
    const sourceCode = page.locator('[data-testid="source-code"]');
    const download = page.locator('[data-testid="download-link"]');
    await expect(sourceCode).toBeVisible();
    await expect(download).toBeVisible();
    // The link variant must be a real anchor (chakra.a), not a div.
    expect(await sourceCode.evaluate((el) => el.tagName)).toBe('A');
    expect(await sourceCode.evaluate((el) => el.className)).toContain(
      'source-code-button',
    );
  });
});
