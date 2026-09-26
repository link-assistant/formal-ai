// Issue #1138 plan 01 L13: one browser-side, need-kind-parameterised source
// walk. Procedure and concept extractors decide what captured bytes mean; this
// module alone owns registry selection, bounds, recursion, accessibility and
// content-addressed capture provenance.

const SOURCE_WALK_TIER_WEIGHTS = Object.freeze({
  original_first_party: 100,
  original_journalism: 85,
  independent_corroboration: 50,
  unoriginal: 0,
});
const SOURCE_WALK_ROLE_ORDER = Object.freeze({ primary: 0, secondary: 1 });
const SOURCE_WALK_ACCESSIBILITY_TTL_SECONDS = 7 * 24 * 60 * 60;

let cachedSourceWalkRegistry = null;
const sourceWalkAccessibility = new Map();
const sourceWalkCaptureCache = new Map();

function sourceWalkChildValue(node, name) {
  const child = (node.children || []).find((item) => item.name === name);
  return child && child.value ? String(child.value) : "";
}

function sourceWalkList(value) {
  return String(value || "").replace(/[()]/gu, " ").split(/\s+/u).filter(Boolean);
}

function sourceWalkTierFromPrimacy(primacy) {
  return {
    direct_artifact: "original_first_party",
    self_published: "original_first_party",
    first_hand_record: "original_journalism",
    editorial_synthesis: "independent_corroboration",
    citation: "independent_corroboration",
  }[primacy] || "unoriginal";
}

function sourceWalkRegistry() {
  if (cachedSourceWalkRegistry) return cachedSourceWalkRegistry;
  const raw = seedRawText(SEED_RAW, "sources-registry.lino");
  if (!raw || !self.FormalAiSeed) return [];
  const root = self.FormalAiSeed.parse(raw);
  const sections = (root.children || []).filter((node) => node.name === "sources_registry");
  const nodes = (sections.length ? sections : [root]).flatMap((section) =>
    (section.children || []).filter((node) => node.name === "source"),
  );
  cachedSourceWalkRegistry = nodes.map((node, registryIndex) => {
    const value = (name) => sourceWalkChildValue(node, name);
    const primacy = (node.children || []).find((child) => child.name === "primacy");
    return {
      id: String(node.value || ""),
      name: value("name"),
      kind: value("kind"),
      serviceGroup: value("service_group"),
      settingsKey: value("settings_key"),
      defaultEnabled: value("default_enabled") !== "false",
      needKinds: sourceWalkList(value("need_kinds")),
      extractor: value("extractor"),
      howToRole: value("how_to_role") || "none",
      api: value("api"),
      languageApi: value("language_api"),
      apiLanguages: sourceWalkList(value("api_language")),
      licenseName: value("license_name"),
      licenseUrl: value("license_url"),
      tier: value("source_tier") || sourceWalkTierFromPrimacy(primacy ? primacy.value : ""),
      registryIndex,
    };
  });
  return cachedSourceWalkRegistry;
}

function sourceWalkPercentEncode(value) {
  let encoded = "";
  for (const byte of new TextEncoder().encode(String(value == null ? "" : value))) {
    const character = String.fromCharCode(byte);
    encoded += /[A-Za-z0-9\-_.~]/u.test(character)
      ? character
      : `%${byte.toString(16).toUpperCase().padStart(2, "0")}`;
  }
  return encoded;
}

function sourceWalkTierWeight(tier) {
  return SOURCE_WALK_TIER_WEIGHTS[tier] ?? 0;
}

function sourceWalkPageTitle(subject, hyphenated = false) {
  return String(subject || "").trim()
    .split(/[\s\x00-\x2f\x3a-\x40\x5b-\x60\x7b-\x7f]+/u)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(hyphenated ? "-" : " ");
}

function sourceWalkEntryUrl(record, subject, language = "") {
  if (language && record.apiLanguages.length > 0 && !record.apiLanguages.includes(language)) {
    return null;
  }
  const primary = record.apiLanguages[0] || "";
  const template = record.languageApi && language && language !== primary
    ? record.languageApi
    : record.api;
  const host = String(template || "").split("://")[1]?.split("/")[0] || "";
  const bindings = {
    title: sourceWalkPageTitle(subject, host.includes("wikihow")),
    query: subject,
    lemma: subject,
    language: language || primary,
  };
  let url = String(template || "");
  for (const [name, value] of Object.entries(bindings)) {
    if (value) url = url.split(`{${name}}`).join(sourceWalkPercentEncode(value));
  }
  return url && !url.includes("{") ? url : null;
}

function sourceWalkServiceAllowed(preferences, record) {
  const setting = preferences ? preferences[record.settingsKey] : undefined;
  if (setting === true) return true;
  if (setting === false) return false;
  return record.defaultEnabled;
}

function sourceWalkOutcome(sourceId, status, detail) {
  return { sourceId, status, detail: String(detail || ""), pages: 0, items: 0 };
}

// A missing page is evidence about this subject, not evidence that the service
// is down. Caching a 404 against the service id would suppress later subjects
// in the same bounded walk, including a captured definition that another
// clause can still resolve.
function sourceWalkResourceMissing(error) {
  return /^http_(?:404|410)$/u.test(String(error || ""));
}

function sourceWalkCandidates(kind, subject, language, preferences, bounds, entryUrl) {
  const selected = [];
  const skipped = [];
  for (const record of sourceWalkRegistry()) {
    if (record.serviceGroup !== "external_trusted" || !record.needKinds.includes(kind)) continue;
    if (!sourceWalkServiceAllowed(preferences, record)) {
      skipped.push(sourceWalkOutcome(record.id, "disabled", record.settingsKey));
      continue;
    }
    const url = entryUrl(record, subject, language);
    if (!url) {
      skipped.push(sourceWalkOutcome(record.id, "unbound_template", record.api));
      continue;
    }
    selected.push({ record, url });
  }
  selected.sort((left, right) => {
    if (kind === "procedure") {
      const role = (SOURCE_WALK_ROLE_ORDER[left.record.howToRole] ?? 255)
        - (SOURCE_WALK_ROLE_ORDER[right.record.howToRole] ?? 255);
      if (role) return role;
    }
    const tier = sourceWalkTierWeight(right.record.tier)
      - sourceWalkTierWeight(left.record.tier);
    if (tier) return tier;
    return kind === "procedure"
      ? left.record.id.localeCompare(right.record.id)
      : left.record.registryIndex - right.record.registryIndex;
  });
  return { selected: selected.slice(0, bounds.maxServices), skipped };
}

function sourceWalkNowSeconds() {
  return Math.floor(Date.now() / 1000);
}

function sourceWalkObserveService(sourceId, status, detail, now) {
  sourceWalkAccessibility.set(sourceId, {
    sourceId,
    status,
    detail: String(detail || ""),
    checkedAt: typeof now === "number" ? now : sourceWalkNowSeconds(),
    ttlSeconds: SOURCE_WALK_ACCESSIBILITY_TTL_SECONDS,
  });
}

function sourceWalkServiceNeedsRefresh(sourceId, now) {
  const record = sourceWalkAccessibility.get(sourceId);
  if (!record) return true;
  const at = typeof now === "number" ? now : sourceWalkNowSeconds();
  return at - record.checkedAt > record.ttlSeconds;
}

function sourceWalkServiceKnownUnreachable(sourceId, now) {
  const record = sourceWalkAccessibility.get(sourceId);
  return Boolean(record && record.status === "unreachable"
    && !sourceWalkServiceNeedsRefresh(sourceId, now));
}

function sourceWalkInvalidateService(sourceId) {
  return sourceWalkAccessibility.delete(sourceId);
}

function sourceWalkInvalidateAllServices() {
  const count = sourceWalkAccessibility.size;
  sourceWalkAccessibility.clear();
  return count;
}

function sourceWalkAccessibilityLino() {
  const lines = ["service_accessibility"];
  for (const id of Array.from(sourceWalkAccessibility.keys()).sort()) {
    const record = sourceWalkAccessibility.get(id);
    lines.push(`  service ${id}`);
    lines.push(`    status ${record.status}`);
    lines.push(`    checked_at ${record.checkedAt}`);
    lines.push(`    ttl_seconds ${record.ttlSeconds}`);
    lines.push(`    detail "${record.detail.replace(/"/gu, '\\"')}"`);
  }
  return `${lines.join("\n")}\n`;
}

async function sourceWalkSha256Hex(text) {
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function sourceWalkFetchCapture(url) {
  const cached = sourceWalkCaptureCache.get(url);
  if (cached) return { ...cached, cached: true };
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

async function sourceWalkSources(kind, subject, language, preferences, bounds, extractor) {
  const entryUrl = extractor.entryUrl || sourceWalkEntryUrl;
  const candidates = sourceWalkCandidates(
    kind,
    subject,
    language,
    preferences,
    bounds,
    entryUrl,
  );
  const walked = { subject, items: [], outcomes: candidates.skipped, bounds };
  const now = typeof extractor.now === "number" ? extractor.now : sourceWalkNowSeconds();
  for (const { record, url: firstUrl } of candidates.selected) {
    if (sourceWalkServiceKnownUnreachable(record.id, now)) {
      const known = sourceWalkAccessibility.get(record.id);
      walked.outcomes.push(sourceWalkOutcome(
        record.id,
        "unreachable_cached",
        known ? known.detail : "",
      ));
      continue;
    }
    const outcome = sourceWalkOutcome(record.id, extractor.emptyStatus || "no_items", firstUrl);
    const queue = [{ url: firstUrl, depth: 0 }];
    const visited = [];
    const sourceItems = [];
    while (queue.length > 0) {
      const { url, depth } = queue.shift();
      if (outcome.pages >= bounds.maxPagesPerService || visited.includes(url)) continue;
      visited.push(url);
      // eslint-disable-next-line no-await-in-loop -- capture order is evidence.
      const capture = await sourceWalkFetchCapture(url);
      if (!capture.ok) {
        const missing = sourceWalkResourceMissing(capture.error);
        outcome.status = missing
          ? "not_found"
          : (url === firstUrl ? "unreachable" : "fallback_failed");
        outcome.detail = `${capture.error} url=${url}`;
        if (url === firstUrl && !missing) {
          sourceWalkObserveService(record.id, "unreachable", capture.error, now);
        }
        break;
      }
      outcome.pages += 1;
      sourceWalkObserveService(record.id, "reachable", `captured ${url}`, now);
      const age = now - Number.parseInt(capture.fetchedAt, 10);
      if (Number.isFinite(age) && age > bounds.maxCaptureAgeSeconds) {
        outcome.detail = `stale_capture age_seconds=${age} url=${url}`;
      }
      const read = extractor.read(record, capture, depth, sourceItems.length, bounds) || {};
      sourceItems.push(...(read.items || []));
      if (read.detail) outcome.detail = read.detail;
      if (depth < bounds.maxDepth) {
        for (const follow of read.follow || []) queue.push({ url: follow, depth: depth + 1 });
      }
      if (extractor.stopAtMaxItems
        && walked.items.length + sourceItems.length >= bounds.maxItems) break;
    }
    if (sourceItems.length > 0) outcome.status = "contributed";
    outcome.items = sourceItems.length;
    walked.items.push(...sourceItems);
    walked.outcomes.push(outcome);
    if (extractor.stopAtMaxItems && walked.items.length >= bounds.maxItems) break;
  }
  if (extractor.stopAtMaxItems) walked.items = walked.items.slice(0, bounds.maxItems);
  return walked;
}
