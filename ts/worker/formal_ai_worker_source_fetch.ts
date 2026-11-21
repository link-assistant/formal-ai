// Provider-owned source capture retrieval: verified cache, offline policy, and live response status.
async function sourceWalkFetchCapture(url, { online = true, bounded = false, indexed = false, maxAgeSeconds = Infinity } = {}) {
  const key = indexed ? `indexed:${url}` : url;
  const cached = sourceWalkCaptureCache.get(key);
  if (cached && (!online || sourceWalkNowSeconds() - Number(cached.fetchedAt) <= maxAgeSeconds)) {
    if (cached.url !== url || typeof cached.text !== "string" || cached.sha256 !== await sourceWalkSha256Hex(cached.text))
      return { ok: false, url, error: "invalid_capture" };
    return { ...cached, cached: true };
  }
  if (!online) return { ok: false, url, failureKind: "offline_cache_miss", error: answerFor("source-capture-offline-cache-miss", "en").replace("{url}", () => String(url)) };
  if (typeof fetch !== "function") return { ok: false, url, error: "fetch_unavailable" };
  try {
    const read = async (signal) => {
      const request = { method: "GET", mode: "cors", signal };
      if (indexed) Object.assign(request, { credentials: "omit", redirect: "error", referrerPolicy: "no-referrer" });
      const response = await fetch(url, request);
      if (!response?.ok) throw new Error(`http_${response?.status || 0}`);
      if (indexed && (response.redirected || (response.url && response.url !== url))) throw new Error("redirected_capture");
      return bounded ? howToReadBoundedResponse(response, signal) : response.text();
    };
    const text = bounded ? await howToWithinCaptureDeadline(read) : await read();
    const capture = {
      ok: true, url, text, sha256: await sourceWalkSha256Hex(text),
      fetchedAt: String(sourceWalkNowSeconds()), cached: false,
    };
    sourceWalkCaptureCache.set(key, Object.freeze({ ...capture }));
    return capture;
  } catch (error) {
    return { ok: false, url, error: error instanceof Error ? error.message : String(error) };
  }
}
