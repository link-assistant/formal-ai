// Generate the macOS Gatekeeper screenshots embedded in the /download page's
// macOS instructions (issue #479).
//
// macOS Gatekeeper dialogs cannot be captured on a hosted macOS CI runner (there
// is no scriptable way to trigger Gatekeeper), so we render a faithful, on-brand
// reproduction of the three macOS 15 (Sequoia) dialogs from a self-contained
// fixture (tests/e2e/fixtures/macos-gatekeeper.html) and screenshot each element
// at devicePixelRatio 2 (retina), mirroring how vk-bot-desktop ships static PNG
// screenshots in its macOS instructions.
//
// The three captures map 1:1 to download.js's installMacosSettingsStep1/2/3:
//   macos-gatekeeper-not-opened.png   -> Step 1 (double-click, click Done)
//   macos-gatekeeper-open-anyway.png  -> Step 2 (Privacy & Security → Open Anyway)
//   macos-gatekeeper-confirm.png      -> Step 3 (confirm Open Anyway)
//
// Usage: node tests/e2e/scripts/generate-macos-screenshots.mjs
import { chromium } from 'playwright';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '..', '..', '..');
const FIXTURE = path.join(__dirname, '..', 'fixtures', 'macos-gatekeeper.html');
const OUT_DIR = path.join(REPO_ROOT, 'src', 'web', 'download', 'assets', 'screenshots');

const SHOTS = [
  { id: '#shot-done', file: 'macos-gatekeeper-not-opened.png' },
  { id: '#shot-settings', file: 'macos-gatekeeper-open-anyway.png' },
  { id: '#shot-confirm', file: 'macos-gatekeeper-confirm.png' },
];

async function main() {
  const browser = await chromium.launch();
  try {
    // deviceScaleFactor: 2 => retina-density PNGs, matching the vk-bot-desktop
    // originals and the existing app-preview captures.
    const context = await browser.newContext({ deviceScaleFactor: 2 });
    const page = await context.newPage();
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto(pathToFileURL(FIXTURE).href, { waitUntil: 'networkidle' });
    // Let the system font fall back / layout settle before capturing.
    await page.waitForTimeout(150);

    for (const { id, file } of SHOTS) {
      const element = page.locator(id);
      await element.waitFor({ state: 'visible' });
      const outPath = path.join(OUT_DIR, file);
      // omitBackground keeps the transparent stage padding transparent so the
      // floating dialog's drop shadow reads naturally over the page background.
      await element.screenshot({ path: outPath, omitBackground: true });
      console.log('wrote ' + path.relative(REPO_ROOT, outPath));
    }
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
