// @ts-check
// Preserve the cause of startup failures instead of reporting only a disabled composer.
async function withWorkerDiagnostics(page, testInfo, action) {
  const events = [];
  const address = value => {
    try { const url = new URL(value); return url.origin + url.pathname; }
    catch { return String(value).slice(0,300); }
  };
  const record = (kind, details) => {
    if (events.length < 40) events.push({ kind, ...details });
  };
  const listeners = {
    pageerror: error => record('pageerror', { message: String(error).slice(0,1200) }),
    console: message => {
      if (['error','warning'].includes(message.type())) record('console', { level: message.type(), message: message.text().slice(0,1200) });
    },
    requestfailed: request => record('requestfailed', { url: address(request.url()), error: request.failure()?.errorText }),
    response: response => {
      if (response.status() >= 400) record('http', { url: address(response.url()), status: response.status() });
    },
  };
  for (const [name, listener] of Object.entries(listeners)) page.on(name, listener);
  try {
    return await action();
  } catch (error) {
    let state;
    try {
      state = await page.evaluate(() => ({
        status: document.querySelector('.status')?.textContent,
        composerDisabled: document.querySelector('[data-testid="chat-composer-input"]')?.hasAttribute('disabled'),
        engineError: document.querySelector('[data-testid="engine-unavailable"]')?.getAttribute('title'),
        assetVersion: window.FORMAL_AI_ASSET_VERSION,
      }));
    } catch (diagnosticError) {
      state = { diagnosticError: String(diagnosticError).slice(0,500) };
    }
    const body = JSON.stringify({ events, state }, null, 2);
    console.error('WorkerStartupDiagnostics\n' + body);
    try { await testInfo.attach('worker-startup-diagnostics', { body, contentType: 'application/json' }); }
    catch (attachmentError) { console.error('Worker diagnostics attachment failed:', String(attachmentError)); }
    throw error;
  } finally {
    for (const [name, listener] of Object.entries(listeners)) page.off(name, listener);
  }
}

module.exports = { withWorkerDiagnostics };
