// Worker module 24. Issue #991 browser mirror of `src/how_to_guide.rs`,
// `src/how_to_guide/extract.rs`, `src/how_to_guide/render.rs`, and
// `src/service_accessibility.rs`.
//
// The Rust solver and this worker are the two production runtimes, and issue
// #991 requires them to execute the *same* bounded source-selection and
// guide-synthesis contract: pick the enabled, relevant services out of
// `seed/sources-registry.lino`, walk them recursively inside declared
// depth/page/time bounds, keep exact provenance (URL, sha256, fetch time,
// license, tier, depth) on every accepted step, resolve copies and
// contradictions with the issue #709 source-tier policy, and remember per
// service accessibility for at least seven days.
//
// Nothing here is hardcoded per service: every endpoint, license, tier, role,
// and settings key comes from the registry, so enabling a service in the seed
// is enough for it to contribute, and the settings opt-out stays authoritative.
//
// `tests/web/issue-991-how-to-synthesis.test.mjs` replays the same committed
// captures under `tests/fixtures/issue-991/` that the Rust offline replay in
// `tests/unit/issue_991_how_to_synthesis.rs` uses, which is what holds the two
// runtimes to one contract rather than to two similar implementations.

/** Declared retrieval bounds; mirrors `GuideBounds::default()`. */
const HOW_TO_GUIDE_BOUNDS = {
  maxDepth: 2,
  maxPagesPerService: 4,
  maxServices: 4,
  maxSteps: 12,
  maxCaptureAgeSeconds: 60 * 60 * 24 * 60,
};

/** Fewer accepted steps than this is not a procedure. */
const HOW_TO_MIN_ACCEPTED_STEPS = 2;

/** How many hex characters of a digest a reader-facing citation shows. */
const HOW_TO_DIGEST_PREFIX = 12;

/** A step shorter than this is a caption or a navigation label. */
const HOW_TO_MIN_STEP_CHARS = 40;
/** Steps are compacted to at most this many characters. */
const HOW_TO_MAX_STEP_CHARS = 180;

/** Issue #709 tier weights, mirrored from `SourceTier::weight_percent`. */
const HOW_TO_TIER_WEIGHTS = {
  original_first_party: 100,
  original_journalism: 85,
  independent_corroboration: 50,
  unoriginal: 0,
};

/** Consultation order of the registry roles; `none` never contributes. */
const HOW_TO_ROLE_ORDER = { primary: 0, secondary: 1 };

/** Seven days — the minimum accessibility TTL issue #991 requires. */
const SERVICE_ACCESSIBILITY_TTL_SECONDS = 7 * 24 * 60 * 60;

/**
 * Words that carry no topic, so requiring a candidate to repeat them would
 * reject correct pages. Mirrors `TOPIC_STOPWORDS` in `src/how_to_guide.rs`.
 */
const HOW_TO_TOPIC_STOPWORDS = [
  "the", "and", "for", "with", "your", "you", "how", "does", "did", "are", "was", "were", "its",
  "into", "onto", "from", "that", "this",
];

let cachedHowToSourceRegistry = null;

/** Every `external_trusted` service declared in the seed registry. */
function howToSourceRegistry() {
  if (cachedHowToSourceRegistry) return cachedHowToSourceRegistry;
  const raw = seedRawText(SEED_RAW, "sources-registry.lino");
  if (!raw || !self.FormalAiSeed) return [];
  const root = self.FormalAiSeed.parse(raw);
  const registry = [];
  const sections = (root.children || []).filter((node) => node.name === "sources_registry");
  const sources = (sections.length ? sections : [root]).flatMap((section) =>
    (section.children || []).filter((node) => node.name === "source"),
  );
  for (const node of sources) {
    const value = (name) => {
      const child = (node.children || []).find((item) => item.name === name);
      return child && child.value ? String(child.value) : "";
    };
    const record = {
      id: node.value || "",
      name: value("name"),
      kind: value("kind"),
      serviceGroup: value("service_group"),
      settingsKey: value("settings_key"),
      defaultEnabled: value("default_enabled") !== "false",
      howToRole: value("how_to_role") || "none",
      tier: value("source_tier") || "independent_corroboration",
      api: value("api"),
      licenseName: value("license_name"),
      licenseUrl: value("license_url"),
    };
    if (record.serviceGroup === "external_trusted") registry.push(record);
  }
  cachedHowToSourceRegistry = registry;
  return registry;
}

/** Percent-encode with RFC 3986's unreserved set, exactly as Rust does. */
function howToPercentEncode(value) {
  const bytes = new TextEncoder().encode(String(value == null ? "" : value));
  let encoded = "";
  for (const byte of bytes) {
    const character = String.fromCharCode(byte);
    if (/[A-Za-z0-9\-_.~]/u.test(character)) encoded += character;
    else encoded += `%${byte.toString(16).toUpperCase().padStart(2, "0")}`;
  }
  return encoded;
}

/** Host of a service endpoint, used to tell wikiHow's hyphenated titles apart. */
function howToSourceHost(record) {
  const match = String(record.api || "").split("://")[1];
  return match ? match.split("/")[0] : String(record.api || "");
}

/** The tier weight of a source, defaulting to independent corroboration. */
function howToTierWeight(tier) {
  const weight = HOW_TO_TIER_WEIGHTS[tier];
  return typeof weight === "number" ? weight : HOW_TO_TIER_WEIGHTS.independent_corroboration;
}

/**
 * The task as a wiki page title: `install docker` becomes `Install-Docker` for
 * wikiHow's hyphenated titles and `Install Docker` elsewhere.
 */
function howToPageTitle(task, hyphenated) {
  return String(task || "")
    .split(/[^0-9A-Za-z]+/u)
    .filter((word) => word.length > 0)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(hyphenated ? "-" : " ");
}

/**
 * Bind the registry's API template for this task, or `null` when a required
 * placeholder cannot be filled from the task alone (GitHub needs an owner and
 * a repository, for instance).
 */
function howToEntryUrl(record, task) {
  const hyphenated = howToSourceHost(record).includes("wikihow");
  let url = String(record.api || "");
  const parameters = [
    ["title", howToPageTitle(task, hyphenated)],
    ["query", task],
    ["lemma", task],
  ];
  for (const [name, value] of parameters) {
    url = url.split(`{${name}}`).join(howToPercentEncode(value));
  }
  return url.includes("{") ? null : url;
}

/** A MediaWiki full-text search URL on the same wiki as the entry endpoint. */
function howToSearchUrl(record, task) {
  const base = String(record.api || "").split("?")[0];
  return `${base}?action=query&list=search&srsearch=${howToPercentEncode(task)}&srlimit=5&format=json&origin=*`;
}

/** A MediaWiki `action=parse` URL on the same wiki as the entry endpoint. */
function howToParseUrl(record, title) {
  const base = String(record.api || "").split("?")[0];
  return `${base}?action=parse&page=${howToPercentEncode(title)}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`;
}

/** The Stack Exchange answers of one question, best-voted first. */
function howToAnswersUrl(record, questionId) {
  const api = String(record.api || "");
  const base = api.split("/search")[0];
  const site = api.includes("site=") ? api.split("site=")[1].split("&")[0] : "stackoverflow";
  return `${base}/questions/${questionId}/answers?order=desc&sort=votes&site=${site}&filter=withbody`;
}

/** Whether the user's settings allow this service. */
function howToServiceAllowed(preferences, record) {
  const setting = preferences ? preferences[record.settingsKey] : undefined;
  if (setting === true) return true;
  if (setting === false) return false;
  return record.defaultEnabled;
}

/**
 * The registry sources that may contribute to `task`, in consultation order:
 * primary roles first, then higher tiers, then registry id. Sources the
 * settings opt out of, sources whose role is `none`, and sources whose API
 * template this task cannot bind never appear.
 */
function howToSelectSources(task, preferences, bounds) {
  const limits = bounds || HOW_TO_GUIDE_BOUNDS;
  return howToSourceRegistry()
    .filter((record) => record.howToRole in HOW_TO_ROLE_ORDER)
    .filter((record) => howToServiceAllowed(preferences, record))
    .filter((record) => howToEntryUrl(record, task) !== null)
    .sort((left, right) => {
      const role = HOW_TO_ROLE_ORDER[left.howToRole] - HOW_TO_ROLE_ORDER[right.howToRole];
      if (role !== 0) return role;
      const tier = howToTierWeight(right.tier) - howToTierWeight(left.tier);
      if (tier !== 0) return tier;
      return left.id < right.id ? -1 : left.id > right.id ? 1 : 0;
    })
    .slice(0, limits.maxServices);
}

/**
 * Services that exist and are procedural but were not consulted, reported so
 * the trace shows the user's choice (or an unbindable template) rather than an
 * unexplained absence.
 */
function howToSkippedSources(task, preferences) {
  const skipped = [];
  for (const record of howToSourceRegistry()) {
    if (!(record.howToRole in HOW_TO_ROLE_ORDER)) continue;
    if (!howToServiceAllowed(preferences, record)) {
      skipped.push(howToOutcome(record.id, "disabled", record.settingsKey));
    } else if (howToEntryUrl(record, task) === null) {
      skipped.push(howToOutcome(record.id, "unbound_template", record.api));
    }
  }
  return skipped;
}

function howToOutcome(sourceId, status, detail) {
  return { sourceId, status, detail: String(detail || ""), pages: 0, steps: 0 };
}

function howToOutcomeTracePayload(outcome) {
  return `source=${outcome.sourceId} status=${outcome.status} pages=${outcome.pages} steps=${outcome.steps} detail=${outcome.detail}`;
}

// --- Per-service accessibility memory (mirror of src/service_accessibility.rs)

/** Records keyed by registry id, each with the TTL it was written under. */
const howToServiceAccessibility = new Map();

function howToNowSeconds() {
  return Math.floor(Date.now() / 1000);
}

/** Record the outcome of a probe. */
function howToObserveService(sourceId, status, detail, now) {
  howToServiceAccessibility.set(sourceId, {
    sourceId,
    status,
    detail: String(detail || ""),
    checkedAt: typeof now === "number" ? now : howToNowSeconds(),
    ttlSeconds: SERVICE_ACCESSIBILITY_TTL_SECONDS,
  });
}

/** Whether a record is older than the TTL it was written under. */
function howToServiceNeedsRefresh(sourceId, now) {
  const record = howToServiceAccessibility.get(sourceId);
  if (!record) return true;
  const at = typeof now === "number" ? now : howToNowSeconds();
  return at - record.checkedAt > record.ttlSeconds;
}

/** A service known to be down stays skipped for the whole TTL. */
function howToServiceKnownUnreachable(sourceId, now) {
  const record = howToServiceAccessibility.get(sourceId);
  if (!record || record.status !== "unreachable") return false;
  return !howToServiceNeedsRefresh(sourceId, now);
}

/** Explicit invalidation: forget one service, or every service. */
function howToInvalidateService(sourceId) {
  return howToServiceAccessibility.delete(sourceId);
}

function howToInvalidateAllServices() {
  const count = howToServiceAccessibility.size;
  howToServiceAccessibility.clear();
  return count;
}

/** The records as Links Notation, the shape `service-accessibility.lino` holds. */
function howToServiceAccessibilityLino() {
  const lines = ["service_accessibility"];
  for (const id of Array.from(howToServiceAccessibility.keys()).sort()) {
    const record = howToServiceAccessibility.get(id);
    lines.push(`  service ${id}`);
    lines.push(`    status ${record.status}`);
    lines.push(`    checked_at ${record.checkedAt}`);
    lines.push(`    ttl_seconds ${record.ttlSeconds}`);
    lines.push(`    detail "${record.detail.replace(/"/gu, '\\"')}"`);
  }
  return `${lines.join("\n")}\n`;
}

// --- Capture (mirror of src/source_fetch.rs, in-session)

/** Content-addressed captures for this worker session, keyed by URL. */
const howToCaptureCache = new Map();

async function howToSha256Hex(text) {
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * Fetch one URL and record exactly what came back.
 *
 * A cached capture is replayed rather than re-requested, so a run over the same
 * task twice costs one request and produces one identical trace.
 */
async function howToFetchCapture(url) {
  const cached = howToCaptureCache.get(url);
  if (cached) return { ...cached, cached: true };
  if (typeof fetch !== "function") {
    return { ok: false, url, error: "fetch_unavailable" };
  }
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
      sha256: await howToSha256Hex(text),
      fetchedAt: String(howToNowSeconds()),
      cached: false,
    };
    howToCaptureCache.set(url, capture);
    return capture;
  } catch (error) {
    return { ok: false, url, error: error instanceof Error ? error.message : String(error) };
  }
}

// --- Payload recognition (mirror of src/how_to_guide/extract.rs)

/** Remove markup, keeping text nodes separated by single spaces. */
function howToStripHtml(value) {
  let text = "";
  let insideTag = false;
  for (const character of String(value == null ? "" : value)) {
    if (character === "<") {
      insideTag = true;
      text += " ";
    } else if (character === ">") {
      insideTag = false;
    } else if (!insideTag) {
      text += character;
    }
  }
  return text;
}

/** Decode the handful of entities MediaWiki and Stack Exchange emit. */
function howToDecodeEntities(value) {
  return String(value == null ? "" : value).replace(/&(#?[0-9A-Za-z]{1,9});/gu, (whole, entity) => {
    const named = { nbsp: " ", "#160": " ", amp: "&", quot: '"', apos: "'", "#039": "'", lt: "<", gt: ">" };
    if (entity in named) return named[entity];
    if (entity.startsWith("#")) {
      const code = Number.parseInt(entity.slice(1), 10);
      if (Number.isFinite(code)) return String.fromCodePoint(code);
    }
    return whole;
  });
}

/**
 * Strip markup and reference markers, collapse whitespace, and cut at a
 * sentence boundary so a step stays one readable instruction.
 */
function howToCompactStepText(value) {
  const text = howToDecodeEntities(howToStripHtml(value))
    .replace(/\[[0-9]+\]/gu, "")
    .split(/\s+/u)
    .filter((word) => word.length > 0)
    .join(" ");
  if (text.length <= HOW_TO_MAX_STEP_CHARS) return text;
  const sentence = text.match(
    new RegExp(`^(.{${HOW_TO_MIN_STEP_CHARS},${HOW_TO_MAX_STEP_CHARS}}?[.!?])\\s`, "u"),
  );
  if (sentence) return sentence[1].trim();
  return `${text.slice(0, HOW_TO_MAX_STEP_CHARS - 3).trim()}...`;
}

/** Decide the payload shape from the captured text alone. */
function howToClassifyPayload(text) {
  let value = null;
  try {
    value = JSON.parse(text);
  } catch {
    return { kind: "unrecognized", reason: "not_json" };
  }
  if (value && value.error) {
    return { kind: "unrecognized", reason: `api_error:${value.error.code || "api_error"}` };
  }
  if (value && value.parse) {
    const html = value.parse.text ? value.parse.text["*"] || "" : "";
    const title = value.parse.displaytitle || value.parse.title || "";
    return { kind: "parse", title: howToCompactStepText(title), html };
  }
  if (value && value.query && Array.isArray(value.query.search)) {
    return { kind: "search", titles: value.query.search.map((item) => String(item.title || "")) };
  }
  if (value && Array.isArray(value.items)) {
    return {
      kind: "items",
      entries: value.items.map((item) => ({
        title: howToCompactStepText(item.title || ""),
        link: String(item.link || ""),
        body: String(item.body || ""),
        questionId: typeof item.question_id === "number" ? item.question_id : null,
      })),
    };
  }
  if (Array.isArray(value) && value.length >= 2) {
    const strings = (entry) => (Array.isArray(entry) ? entry.filter((item) => typeof item === "string") : []);
    return { kind: "opensearch", titles: strings(value[1]), urls: strings(value[3]) };
  }
  return { kind: "unrecognized", reason: "unknown_shape" };
}

/** The inner HTML of every `<li>` element, in document order. */
function howToListItems(html) {
  const items = [];
  const source = String(html || "");
  let index = 0;
  while (index < source.length) {
    const start = source.indexOf("<li", index);
    if (start === -1) break;
    const openEnd = source.indexOf(">", start);
    if (openEnd === -1) break;
    const innerStart = openEnd + 1;
    const closing = source.indexOf("</li>", innerStart);
    const end = closing === -1 ? source.length : closing;
    items.push(source.slice(innerStart, end));
    index = end + 1;
  }
  return items;
}

/**
 * Ordered steps found in rendered HTML. List items are the only step carrier:
 * prose paragraphs describe, lists instruct. Items whose first child is bold
 * are section labels in the MediaWiki skin, so they are skipped.
 */
function howToExtractSteps(html, limit) {
  const steps = [];
  for (const item of howToListItems(html)) {
    if (item.trimStart().startsWith("<b>")) continue;
    const text = howToCompactStepText(item);
    if (text.length < HOW_TO_MIN_STEP_CHARS || steps.includes(text)) continue;
    steps.push(text);
    if (steps.length >= limit) break;
  }
  return steps;
}

/** Same-wiki article titles linked from rendered HTML, deduplicated. */
function howToWikiLinkTitles(html, limit) {
  const titles = [];
  const pattern = /href="\/wiki\/([^"]*)"/gu;
  let match = pattern.exec(String(html || ""));
  while (match) {
    const target = match[1];
    if (target && !target.includes(":") && !target.includes("#")) {
      const title = howToDecodeEntities(target).split("_").join(" ");
      if (!titles.includes(title)) {
        titles.push(title);
        if (titles.length >= limit) break;
      }
    }
    match = pattern.exec(String(html || ""));
  }
  return titles;
}

// --- Relevance (mirror of `matches_task` in src/how_to_guide.rs)

function howToSingular(word) {
  if (word.endsWith("s") && !word.endsWith("ss") && word.length > 3) return word.slice(0, -1);
  return word;
}

function howToTopicWords(value) {
  return String(value == null ? "" : value)
    .split(/[^0-9A-Za-z]+/u)
    .map((word) => word.toLowerCase())
    .filter((word) => word.length > 2 && !HOW_TO_TOPIC_STOPWORDS.includes(word))
    .map(howToSingular);
}

/**
 * Whether `candidate` is about `task`. A search endpoint answers with its
 * *best* matches, not with matching pages, so a candidate contributes only when
 * it repeats every topic word of the task.
 */
function howToMatchesTask(task, candidate) {
  const wanted = howToTopicWords(task);
  if (wanted.length === 0) return true;
  const offered = howToTopicWords(candidate);
  return wanted.every((word) => offered.includes(word));
}

// --- The walk

function howToPushSteps(record, capture, depth, found, steps) {
  for (const text of found) {
    if (steps.some((step) => step.text === text)) continue;
    steps.push({
      text,
      sourceId: record.id,
      sourceName: record.name,
      sourceUrl: capture.url,
      sha256: capture.sha256,
      fetchedAt: capture.fetchedAt,
      cached: Boolean(capture.cached),
      tier: record.tier,
      licenseName: record.licenseName,
      licenseUrl: record.licenseUrl,
      depth,
      position: steps.length + 1,
    });
  }
}

/**
 * Walk one service inside the declared bounds, returning its candidate steps.
 *
 * Only the service's *declared* entry endpoint speaks for the service: wikiHow
 * answers `action=parse` and returns 500 on `list=search`, so letting the
 * fallback's failure mark the whole service unreachable would blank its working
 * endpoint for the seven-day accessibility TTL.
 */
async function howToCaptureService(record, task, entryUrl, bounds, outcome, now) {
  const queue = [{ url: entryUrl, depth: 0 }];
  const visited = [];
  const steps = [];
  const isWiki = String(record.api || "").includes("api.php");
  while (queue.length > 0) {
    const { url, depth } = queue.shift();
    if (outcome.pages >= bounds.maxPagesPerService || visited.includes(url)) continue;
    visited.push(url);
    const capture = await howToFetchCapture(url);
    if (!capture.ok) {
      if (url === entryUrl) {
        howToObserveService(record.id, "unreachable", capture.error, now);
        outcome.status = "unreachable";
      } else {
        outcome.status = "fallback_failed";
      }
      outcome.detail = `${capture.error} url=${url}`;
      break;
    }
    outcome.pages += 1;
    howToObserveService(record.id, "reachable", `captured ${url}`, now);
    const age = now - Number.parseInt(capture.fetchedAt, 10);
    if (Number.isFinite(age) && age > bounds.maxCaptureAgeSeconds) {
      outcome.detail = `stale_capture age_seconds=${age} url=${url}`;
    }
    const payload = howToClassifyPayload(capture.text);
    if (payload.kind === "parse") {
      const found = howToExtractSteps(payload.html, bounds.maxSteps);
      if (found.length === 0 && depth < bounds.maxDepth) {
        for (const title of howToWikiLinkTitles(payload.html, bounds.maxPagesPerService)) {
          if (howToMatchesTask(task, title)) {
            queue.push({ url: howToParseUrl(record, title), depth: depth + 1 });
          }
        }
      }
      howToPushSteps(record, capture, depth, found, steps);
    } else if (payload.kind === "items") {
      // Depth 0 is the question search, where relevance still has to be judged;
      // deeper captures are the answers to a question already judged relevant.
      const relevant =
        depth === 0
          ? payload.entries.filter(
              (entry) => howToMatchesTask(task, entry.title) || howToMatchesTask(task, entry.link),
            )
          : payload.entries;
      if (relevant.length === 0) outcome.detail = `no_relevant_result url=${url}`;
      const before = steps.length;
      for (const entry of relevant) {
        howToPushSteps(record, capture, depth, howToExtractSteps(entry.body, bounds.maxSteps), steps);
      }
      if (steps.length === before && depth < bounds.maxDepth) {
        // A question body states the problem; the procedure is in the answers.
        for (const entry of relevant) {
          if (entry.questionId) {
            queue.push({ url: howToAnswersUrl(record, entry.questionId), depth: depth + 1 });
          }
        }
      }
    } else if (payload.kind === "opensearch" || payload.kind === "search") {
      const relevant = payload.titles
        .filter((title) => howToMatchesTask(task, title))
        .slice(0, bounds.maxPagesPerService);
      if (relevant.length === 0) outcome.detail = `no_relevant_result url=${url}`;
      if (depth < bounds.maxDepth) {
        for (const title of relevant) {
          queue.push({ url: howToParseUrl(record, title), depth: depth + 1 });
        }
      }
    } else {
      outcome.detail = `unreadable_payload reason=${payload.reason} url=${url}`;
      // A title guess that misses is not a dead end: the same wiki can be
      // searched for the task, and the hits parsed one hop deeper.
      if (isWiki && String(payload.reason).startsWith("api_error") && depth < bounds.maxDepth) {
        queue.push({ url: howToSearchUrl(record, task), depth: depth + 1 });
      }
    }
  }
  return steps;
}

/**
 * Identical bytes under two URLs mean one of them is a copy. The higher tier
 * keeps the capture; the copy contributes nothing, exactly as the issue #709
 * policy decides it for search results.
 */
function howToApplyCopiedSourcePolicy(steps, guide) {
  const owner = new Map();
  for (const step of steps) {
    const weight = howToTierWeight(step.tier);
    const existing = owner.get(step.sha256);
    if (!existing) {
      owner.set(step.sha256, { url: step.sourceUrl, weight });
    } else if (existing.url !== step.sourceUrl && weight > existing.weight) {
      owner.set(step.sha256, { url: step.sourceUrl, weight });
      if (!guide.copies.includes(existing.url)) guide.copies.push(existing.url);
    } else if (existing.url !== step.sourceUrl) {
      if (!guide.copies.includes(step.sourceUrl)) guide.copies.push(step.sourceUrl);
    }
  }
  return steps.filter((step) => !guide.copies.includes(step.sourceUrl));
}

/** The action a step describes: the first three meaningful words, lowercased. */
function howToActionKey(text) {
  return String(text || "")
    .split(/\s+/u)
    .map((word) => word.replace(/[^0-9A-Za-z]/gu, "").toLowerCase())
    .filter((word) => word.length > 0)
    .slice(0, 3)
    .join(" ");
}

/**
 * Two sources describing the same action differently is a contradiction, not
 * two steps. The higher tier wins and the disagreement is recorded.
 */
function howToApplyConflictPolicy(steps, guide) {
  const kept = [];
  for (const step of steps) {
    const action = howToActionKey(step.text);
    const index = kept.findIndex((existing) => howToActionKey(existing.text) === action);
    if (index === -1) {
      kept.push(step);
      continue;
    }
    if (kept[index].sourceId === step.sourceId || kept[index].text === step.text) continue;
    if (howToTierWeight(step.tier) > howToTierWeight(kept[index].tier)) {
      guide.conflicts.push({
        action,
        keptSource: step.sourceId,
        droppedSource: kept[index].sourceId,
        droppedText: kept[index].text,
      });
      kept[index] = step;
    } else {
      guide.conflicts.push({
        action,
        keptSource: kept[index].sourceId,
        droppedSource: step.sourceId,
        droppedText: step.text,
      });
    }
  }
  return kept;
}

/**
 * Presentation order: higher tiers first, then shallower captures (a page the
 * service answered directly is more direct evidence than one reached by
 * following a search result), then source id and the source's own order.
 */
function howToOrderSteps(steps, maxSteps) {
  return steps
    .slice()
    .sort((left, right) => {
      const tier = howToTierWeight(right.tier) - howToTierWeight(left.tier);
      if (tier !== 0) return tier;
      if (left.depth !== right.depth) return left.depth - right.depth;
      if (left.sourceId !== right.sourceId) return left.sourceId < right.sourceId ? -1 : 1;
      return left.position - right.position;
    })
    .slice(0, maxSteps);
}

/** Whether the run found enough corroborated procedure to answer with. */
function howToGuideIsSufficient(guide) {
  return Boolean(guide) && guide.steps.length >= HOW_TO_MIN_ACCEPTED_STEPS;
}

/** Synthesise a guide for `task` from the enabled registry services. */
async function synthesizeHowToGuide(task, preferences, bounds, now) {
  const limits = bounds || HOW_TO_GUIDE_BOUNDS;
  const at = typeof now === "number" ? now : howToNowSeconds();
  const guide = {
    task: String(task || "").trim(),
    steps: [],
    outcomes: howToSkippedSources(String(task || "").trim(), preferences),
    conflicts: [],
    copies: [],
    bounds: limits,
  };
  let collected = [];
  for (const record of howToSelectSources(guide.task, preferences, limits)) {
    if (howToServiceKnownUnreachable(record.id, at)) {
      const known = howToServiceAccessibility.get(record.id);
      guide.outcomes.push(howToOutcome(record.id, "unreachable_cached", known ? known.detail : ""));
      continue;
    }
    const entryUrl = howToEntryUrl(record, guide.task);
    if (entryUrl === null) continue;
    const outcome = howToOutcome(record.id, "no_steps", entryUrl);
    // eslint-disable-next-line no-await-in-loop -- the bounds are per service and
    // the accessibility record is written between services, so the walk is serial.
    const steps = await howToCaptureService(record, guide.task, entryUrl, limits, outcome, at);
    if (steps.length > 0) outcome.status = "contributed";
    outcome.steps = steps.length;
    guide.outcomes.push(outcome);
    collected = collected.concat(steps);
  }
  collected = howToApplyCopiedSourcePolicy(collected, guide);
  collected = howToApplyConflictPolicy(collected, guide);
  guide.steps = howToOrderSteps(collected, limits.maxSteps);
  return guide;
}

// --- Projections (mirror of src/how_to_guide/render.rs)

function howToBoundsTracePayload(bounds) {
  return `max_depth=${bounds.maxDepth} max_pages_per_service=${bounds.maxPagesPerService} max_services=${bounds.maxServices} max_steps=${bounds.maxSteps} max_capture_age_seconds=${bounds.maxCaptureAgeSeconds}`;
}

function howToStepProvenance(step) {
  return `source=${step.sourceId} url=${step.sourceUrl} sha256=${step.sha256} fetched_at=${step.fetchedAt} cached=${step.cached} tier=${step.tier} license=${step.licenseName} depth=${step.depth} position=${step.position}`;
}

/** Deterministic evidence lines, one per decision, in a stable order. */
function howToGuideEvidence(guide) {
  const lines = [`how_to:bounds task=${guide.task} ${howToBoundsTracePayload(guide.bounds)}`];
  for (const outcome of guide.outcomes) lines.push(`how_to:source ${howToOutcomeTracePayload(outcome)}`);
  for (const copy of guide.copies) lines.push(`how_to:copied_source url=${copy} tier=unoriginal`);
  for (const conflict of guide.conflicts) {
    lines.push(
      `conflict:source_disagreement action=${conflict.action} kept=${conflict.keptSource} dropped=${conflict.droppedSource}`,
    );
  }
  guide.steps.forEach((step, index) => {
    lines.push(`how_to:step rank=${index + 1} ${howToStepProvenance(step)}`);
  });
  if (!howToGuideIsSufficient(guide)) {
    lines.push(
      `how_to:insufficient_evidence steps=${guide.steps.length} required=${HOW_TO_MIN_ACCEPTED_STEPS}`,
    );
  }
  return lines;
}

/** One seeded `how_to_guide_*` phrase with its named fields substituted. */
function howToGuideChrome(intent, language, values) {
  let rendered = answerFor(intent, language || "en");
  for (const name of Object.keys(values)) rendered = rendered.split(`{${name}}`).join(String(values[name]));
  return rendered;
}

/**
 * The guide as a reader sees it: numbered steps with the source of each.
 *
 * Every fragment of prose is looked up from `data/seed/multilingual-responses-procedure.lino`,
 * exactly as `src/how_to_guide/render.rs` looks it up, so the two runtimes cannot drift apart on
 * wording and a seeded language renders that evidence rather than a translation of it.
 */
function howToGuideMarkdown(guide, language) {
  const say = (intent, values) => howToGuideChrome(intent, language, values || {});
  const renderStep = (step, index) =>
    say("how_to_guide_step", { rank: index + 1, text: step.text, source: step.sourceName, license: step.licenseName, digest: step.sha256.slice(0, HOW_TO_DIGEST_PREFIX) });
  const sections = [say("how_to_guide_heading", { task: guide.task })];
  sections.push(
    howToGuideIsSufficient(guide)
      ? guide.steps.map(renderStep).join("\n")
      : say("how_to_guide_insufficient_evidence", { steps: guide.steps.length, required: HOW_TO_MIN_ACCEPTED_STEPS }),
  );
  const sources = [say("how_to_guide_sources_heading")];
  for (const { sourceId: source, status, pages, steps, detail } of guide.outcomes)
    sources.push(say("how_to_guide_source_outcome", { source, status, pages, steps, detail }));
  for (const step of guide.steps) {
    const citation = say("how_to_guide_citation", { source: step.sourceId, url: step.sourceUrl, license: step.licenseName, license_url: step.licenseUrl });
    if (!sources.includes(citation)) sources.push(citation);
  }
  sections.push(sources.join("\n"));
  if (guide.conflicts.length > 0) {
    const conflicts = guide.conflicts.map((conflict) =>
      say("how_to_guide_conflict", { action: conflict.action, kept: conflict.keptSource, dropped: conflict.droppedSource, text: conflict.droppedText }));
    sections.push([say("how_to_guide_conflicts_heading")].concat(conflicts).join("\n"));
  }
  const copies = guide.copies.map((copy) => say("how_to_guide_copy", { url: copy }));
  if (copies.length > 0) sections.push([say("how_to_guide_copies_heading")].concat(copies).join("\n"));
  sections.push(say("how_to_guide_bounds", { bounds: howToBoundsTracePayload(guide.bounds) }));
  return sections.join("\n\n");
}

/**
 * The `procedural_how_to` answer envelope, or `null` when the run did not find
 * enough evidence to assert a procedure.
 *
 * `tryProceduralHowTo` in worker module 17 calls this first and falls through to
 * its existing discovery plan on `null`, which is the same relationship
 * `try_how_to_procedure` has with the pre-existing Rust handler: the synthesis
 * answers when it can, and never removes an answer that used to be produced.
 */
async function trySynthesizedHowToGuide(task, preferences) {
  const subject = String((task && task.task) || "").trim();
  if (!subject) return null;
  let guide = null;
  try {
    guide = await synthesizeHowToGuide(subject, preferences);
  } catch {
    return null;
  }
  if (!howToGuideIsSufficient(guide)) return null;
  const evidence = [`procedural_how_to:stage:multi_source_synthesis`].concat(howToGuideEvidence(guide));
  for (const step of guide.steps) {
    const source = `source:${step.sourceUrl}`;
    if (!evidence.includes(source)) evidence.push(source);
  }
  return {
    intent: "procedural_how_to",
    content: howToGuideMarkdown(guide),
    confidence: 0.9,
    evidence,
    diagnostics: "",
    query: `how to ${subject}`,
    guide,
    formalizedObject: `HOWTO:${subject}`,
  };
}
