// Provider-owned source capture retrieval: verified cache/seed, offline policy, and live response status.
// Native counterpart: source_fetch::CachedSourceClient. Body bytes never classify the provider failure.
async function sourceWalkFetchCapture(url, { online = true } = {}) {
  const cached = sourceWalkCaptureCache.get(url);
  if (cached) return { ...cached, cached: true };
  const seeded = await sourceWalkSeedCapture(url);
  if (seeded) return seeded;
  if (!online) return { ok: false, url, failureKind: "offline_cache_miss", error: answerFor("source-capture-offline-cache-miss", "en").replace("{url}", () => String(url)) };
  if (typeof fetch !== "function") return { ok: false, url, error: "fetch_unavailable" };
  try {
    const response = await fetch(url, { method: "GET", mode: "cors" });
    if (!response || !response.ok) {
      return { ok: false, url, error: `http_${response ? response.status : 0}` };
    }
    const text = await response.text();
    const capture = {
      ok: true,
      url,
      text,
      sha256: await sourceWalkSha256Hex(text),
      fetchedAt: String(sourceWalkNowSeconds()),
      cached: false,
    };
    sourceWalkCaptureCache.set(url, capture);
    return capture;
  } catch (error) {
    return { ok: false, url, error: error instanceof Error ? error.message : String(error) };
  }
}
