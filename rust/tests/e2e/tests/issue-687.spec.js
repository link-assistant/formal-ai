// @ts-check
const { test, expect } = require("@playwright/test");

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await input.fill(text);
  const assistants = page.locator('[data-testid="chat-message"].assistant');
  const before = await assistants.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect
    .poll(() => assistants.count(), { timeout: 15_000 })
    .toBeGreaterThan(before);
  return assistants.last();
}

test.describe("Issue #687 - seed-backed natural-language interface control", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("./");
    await expect(page.locator(".app")).toBeVisible({ timeout: 15_000 });
    const demoToggle = page.locator(".mode-toggle");
    await expect(demoToggle).toBeVisible();
    if (
      (await page.locator('[data-testid="demo-status"]').textContent()) !==
      "Manual mode"
    ) {
      await demoToggle.click();
    }
    await expect(
      page.locator('[data-testid="chat-composer-input"]'),
    ).toBeEnabled();
    const tools = page.getByRole("button", { name: "Tools", exact: true });
    if ((await tools.getAttribute("aria-expanded")) !== "true") {
      await tools.click();
    }
    await expect
      .poll(() => page.locator('[data-testid="tool-entry"]').count(), {
        timeout: 15_000,
      })
      .toBeGreaterThan(0);
    const capabilityCount = await page.evaluate(async () => {
      const loaded = await window.FormalAiSeed.loadAll();
      return loaded.interfaceCapabilities.length;
    });
    expect(capabilityCount).toBe(7);
  });

  test("controls settings that previously had no message-command route", async ({
    page,
  }) => {
    const settings = page.getByRole("button", { name: "Settings", exact: true });
    if ((await settings.getAttribute("aria-expanded")) !== "true") {
      await settings.click();
    }

    const thinkingAnswer = await sendPrompt(
      page,
      "Set thinking detail to detailed",
    );
    await expect(thinkingAnswer).toContainText(
      "Thinking detail is now detailed",
    );
    await expect(
      page.locator('[data-testid="setting-thinking-detail"]'),
    ).toHaveValue("detailed");

    await sendPrompt(page, "Set message animation to 3 seconds");
    await expect(
      page.locator('[data-testid="setting-min-message-animation"]'),
    ).toHaveValue("3000");

    await sendPrompt(page, "Set follow-up probability to 35%");
    await expect(
      page.locator('[data-testid="setting-follow-up-probability"]'),
    ).toHaveValue("0.35");

    await sendPrompt(page, "Use the Tabler toolbar icon pack");
    await expect(
      page.locator('[data-testid="setting-toolbar-icon-pack"]'),
    ).toHaveValue("tabler-icons");

    const answer = await sendPrompt(page, "Set full auto mode");
    await expect(
      page.locator('[data-testid="mode-option-fullAuto"]'),
    ).toHaveAttribute("aria-checked", "true");
    await expect(answer).toContainText("Mode is now fullAuto");

    if (process.env.ISSUE_687_SCREENSHOT_PATH) {
      await page.screenshot({
        path: process.env.ISSUE_687_SCREENSHOT_PATH,
        fullPage: true,
      });
    }
  });
});
