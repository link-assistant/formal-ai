// @ts-check
// Preserve the cause of startup failures instead of reporting only a disabled composer.
async function withWorkerDiagnostics(page, testInfo, action) {
  const events = [];
  const started = Date.now(), pending = new Map(), recentFinished = [];
  let observedRequests = 0, finishedRequests = 0;
  const networkSource = typeof page.context === 'function' ? page.context() : page;
  const address = value => {
    try { const url = new URL(value); return (url.origin + url.pathname).slice(0,300); }
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
  };
  const networkListeners = {
    request: request => {
      observedRequests++;
      if (pending.size < 64) pending.set(request, { url: address(request.url()), started: Date.now() });
    },
    requestfinished: request => {
      finishedRequests++;
      pending.delete(request);
      recentFinished.push(address(request.url()));
      if (recentFinished.length > 8) recentFinished.shift();
    },
    requestfailed: request => {
      pending.delete(request);
      record('requestfailed', { url: address(request.url()), error: request.failure()?.errorText });
    },
    response: response => {
      if (response.status() >= 400) record('http', { url: address(response.url()), status: response.status() });
    },
  };
  for (const [name, listener] of Object.entries(listeners)) page.on(name, listener);
  for (const [name, listener] of Object.entries(networkListeners)) networkSource.on(name, listener);
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
    const network = { observedRequests, finishedRequests, recentFinished,
      pending: [...pending.values()].map(item => ({ url: item.url, elapsedMs: Date.now() - item.started })) };
    const body = JSON.stringify({ events, state, elapsedMs: Date.now() - started, network }, null, 2);
    console.error('WorkerStartupDiagnostics\n' + body);
    try { await testInfo.attach('worker-startup-diagnostics', { body, contentType: 'application/json' }); }
    catch (attachmentError) { console.error('Worker diagnostics attachment failed:', String(attachmentError)); }
    throw error;
  } finally {
    for (const [name, listener] of Object.entries(listeners)) page.off(name, listener);
    for (const [name, listener] of Object.entries(networkListeners)) networkSource.off(name, listener);
  }
}

module.exports = { withWorkerDiagnostics };
