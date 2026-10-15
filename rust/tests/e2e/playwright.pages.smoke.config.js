// @ts-check
// Public-CDN smoke only; the original full Pages matrix remains explicitly runnable.
// Debt and restoration: docs/testing/pages-live-rate-budget.md.
const full = require('./playwright.pages.config');

module.exports = {
  ...full,
  testMatch: ['**/pages-live-smoke.spec.js'],
  workers: 1,
  fullyParallel: false,
  retries: 0,
  use: {
    ...full.use,
    // Bulk offline precaching is covered on owned CI, not against the public CDN.
    serviceWorkers: 'block',
  },
};
