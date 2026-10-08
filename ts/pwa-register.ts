// A failed installation does not claim offline readiness; the cache requires the seed.
if ("serviceWorker" in navigator && window.isSecureContext && !window.FormalAiDesktop) {
  window.addEventListener("load", async () => {
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
