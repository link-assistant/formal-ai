// A failed installation does not claim offline readiness; the cache requires the seed.
//
// The offline installation precaches every asset of the app: a second copy of
// each worker module and seed, several hundred requests. Started on page load,
// it competed with the worker's cold start for the same server and cores, and
// the engine became ready about twice as late. It now starts once the page has
// loaded and the app has announced the ready engine (`formal-ai-ready`), so it
// never delays the first answer. app/index.html runs this script before
// app.js, so the listener is in place before the worker can report ready.
if ("serviceWorker" in navigator && window.isSecureContext && !window.FormalAiDesktop) {
  const loaded = new Promise((resolve) => window.addEventListener("load", resolve, { once: true }));
  const engineReady = new Promise((resolve) => window.addEventListener("formal-ai-ready", resolve, { once: true }));
  Promise.all([loaded, engineReady]).then(async () => {
    try {
      const registration = await navigator.serviceWorker.register(new URL("service-worker.js", document.baseURI), {
        scope: new URL("./", document.baseURI).pathname,
      });
      window.FORMAL_AI_OFFLINE_REGISTRATION = registration;
    } catch (error) {
      console.warn("formal-ai offline installation failed:", error);
    }
  });
}
