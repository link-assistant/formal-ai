// Provider-owned source capture retrieval: verified cache, offline policy, and live response status.
async function sourceWalkFetchCapture(url, { online = true, bounded = false } = {}) {
  const cached = sourceWalkCaptureCache.get(url);
  if (cached) {
    if (cached.url !== url || typeof cached.text !== "string" || cached.sha256 !== await sourceWalkSha256Hex(cached.text))
      return { ok: false, url, error: "invalid_capture" };
    return { ...cached, cached: true };
  }
  if (!online) return { ok: false, url, failureKind: "offline_cache_miss", error: answerFor("source-capture-offline-cache-miss", "en").replace("{url}", () => String(url)) };
  if (typeof fetch !== "function") return { ok: false, url, error: "fetch_unavailable" };
  try {
    const options = bounded ? howToCaptureFetchOptions() : { method: "GET", mode: "cors" };
    const response = await fetch(url, options);
    if (!response || !response.ok) {
      return { ok: false, url, error: `http_${response ? response.status : 0}` };
    }
    const text = bounded ? await howToReadBoundedResponse(response, options.signal) : await response.text();
    const capture = {
      ok: true, url, text, sha256: await sourceWalkSha256Hex(text),
      fetchedAt: String(sourceWalkNowSeconds()), cached: false,
    };
    sourceWalkCaptureCache.set(url, Object.freeze({ ...capture }));
    return capture;
  } catch (error) {
    return { ok: false, url, error: error instanceof Error ? error.message : String(error) };
  }
}
