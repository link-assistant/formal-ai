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
    const read = async (signal) => {
      const response = await fetch(url, { method: "GET", mode: "cors", signal });
      if (!response?.ok) throw new Error(`http_${response?.status || 0}`);
      return bounded ? howToReadBoundedResponse(response, signal) : response.text();
    };
    const text = bounded ? await howToWithinCaptureDeadline(read) : await read();
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
