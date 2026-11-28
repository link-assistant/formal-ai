let cachedReleaseTimelines = null;

/**
 * The phrasing blocks and timelines of data/seed/release-timelines.lino.
 * Mirrors parse_release_timelines in rust/src/seed/release_timelines.rs.
 * @returns {{phrasings: object[], timelines: object[]}}
 */
function releaseTimelineRegistry() {
  if (cachedReleaseTimelines) return cachedReleaseTimelines;
  const text = seedRawText(SEED_RAW, "release-timelines.lino");
  const registry = parseReleaseTimelineRegistry(text);
  if (text) cachedReleaseTimelines = registry;
  return registry;
}

function parseReleaseTimelineRegistry(text) {
  const root = text ? parseLinoTree(text).children.find((node) => node.name === "release_timelines") : null;
  const records = root ? root.children : [];
  const localizedPairs = (node, field) => node.children
    .filter((child) => child.name === "localized")
    .map((child) => ({ language: child.value, text: childValue(child, field) }))
    .filter((pair) => pair.language && pair.text);
  const parseTimeline = (node) => ({
    slug: node.value || node.name,
    subjectQid: childValue(node, "subject-qid"),
    sourceUrl: childValue(node, "source-url"),
    sourceLabel: childValue(node, "source-label"),
    retrievedAt: String(childValue(node, "retrieved-at") || ""),
    freshForDays: Number.parseInt(String(childValue(node, "fresh-for-days") || "").trim(), 10) || 0,
    subjects: localizedPairs(node, "subject"),
    entries: node.children.filter((child) => child.name === "entry").map((entry) => ({
      qid: entry.value,
      releaseDate: String(childValue(entry, "release-date") || ""),
      titles: localizedPairs(entry, "title"),
    })),
  });
  const series = records.find((node) => node.name === "open-series");
  return {
    query: root ? childValue(root, "query") : "",
    openSeries: series ? {
      ...parseTimeline(series),
      requests: series.children.filter(node => node.name === "request").map(node => ({
        language: node.value,
        frames: node.children.filter(field => field.name === "frame").map(field => field.value),
        search: childValue(node, "search"),
        subject: childValue(node, "subject"),
      })),
    } : null,
    phrasings: records.filter((node) => node.name === "phrasing").map((node) => ({
      language: node.value,
      releasedHeading: childValue(node, "released-heading"),
      releasedItem: childValue(node, "released-item"),
      announcedHeading: childValue(node, "announced-heading"),
      announcedItem: childValue(node, "announced-item"),
      undatedItem: childValue(node, "undated-item"),
      itemSeparator: childValue(node, "item-separator"),
      sectionEnd: childValue(node, "section-end"),
      provenanceNote: childValue(node, "provenance-note"),
      staleNote: childValue(node, "stale-note"),
    })),
    timelines: records.filter((node) => node.name === "timeline").map(parseTimeline),
  };
}

/** The pair for `language`, else the English one (Rust `title_for` / `subject_for`). */
function releaseTimelineLocalized(pairs, language, fallback) {
  const hit = pairs.find((pair) => pair.language === language) || pairs.find((pair) => pair.language === "en");
  return hit ? hit.text : fallback;
}

/** Days since the Unix epoch for an ISO day, or null (Rust `days_from_iso_date`). */
function releaseTimelineDays(date) {
  if (!/^\d{4}-\d{2}-\d{2}$/u.test(date)) return null;
  const [year, month, day] = date.split("-").map(Number);
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysPerMonth = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  if (month < 1 || month > 12 || day < 1 || day > daysPerMonth[month - 1]) return null;
  const adjustedYear = year - (month <= 2 ? 1 : 0);
  const era = Math.floor(adjustedYear / 400);
  const yearOfEra = adjustedYear - era * 400;
  const dayOfYear = Math.floor((153 * (month + (month > 2 ? -3 : 9)) + 2) / 5) + day - 1;
  const dayOfEra = yearOfEra * 365 + Math.floor(yearOfEra / 4) - Math.floor(yearOfEra / 100) + dayOfYear;
  return era * 146097 + dayOfEra - 719468;
}

/**
 * Render a source-verified timeline in `language` as of `today` (an ISO day).
 * @param {object} timeline
 * @param {string} language
 * @param {string} today
 * @param {object} registry
 * @returns {{text: string, released: object[], announced: object[], stale: boolean}|null}
 */
function renderReleaseTimeline(timeline, language, today, registry = releaseTimelineRegistry()) {
  const phrasing = registry.phrasings.find((candidate) => candidate.language === language) ||
    registry.phrasings.find((candidate) => candidate.language === "en");
  if (!timeline?.entries.length || !phrasing) return null;
  const isReleased = (entry) => entry.releaseDate !== "" && entry.releaseDate <= today;
  const released = timeline.entries.filter(isReleased).sort((left, right) =>
    (left.releaseDate < right.releaseDate ? -1 : left.releaseDate > right.releaseDate ? 1
      : left.qid < right.qid ? -1 : left.qid > right.qid ? 1 : 0));
  const announced = timeline.entries.filter((entry) => !isReleased(entry));
  const retrieved = releaseTimelineDays(timeline.retrievedAt);
  const now = releaseTimelineDays(today);
  const stale = retrieved === null || now === null || now - retrieved > timeline.freshForDays;
  const subject = releaseTimelineLocalized(timeline.subjects, language, "");
  const fill = (template, entry, position) => String(template)
    .split("{position}").join(String(position))
    .split("{title}").join(releaseTimelineLocalized(entry.titles, language, entry.qid))
    .split("{year}").join(entry.releaseDate.split("-")[0] || "")
    .split("{date}").join(entry.releaseDate);
  const sections = [];
  if (released.length > 0) {
    const items = released.map((entry, index) => fill(phrasing.releasedItem, entry, index + 1));
    sections.push(`${phrasing.releasedHeading.split("{subject}").join(subject)} ${items.join(phrasing.itemSeparator)}${phrasing.sectionEnd}`);
  }
  if (announced.length > 0) {
    const items = announced.map((entry) =>
      fill(entry.releaseDate === "" ? phrasing.undatedItem : phrasing.announcedItem, entry, 0));
    sections.push(`${phrasing.announcedHeading.split("{subject}").join(subject)} ${items.join(phrasing.itemSeparator)}${phrasing.sectionEnd}`);
  }
  sections.push(String(stale ? phrasing.staleNote : phrasing.provenanceNote)
    .split("{source}").join(timeline.sourceLabel)
    .split("{retrieved}").join(timeline.retrievedAt));
  return { text: sections.join(" "), released, announced, stale };
}

function releaseTimelineQueryUrl(timeline, query, language) {
  if (!/^Q[0-9]{1,18}$/u.test(timeline.subjectQid) || !/^[a-z]{2,3}$/u.test(language) ||
      !query.includes("{subject}") || !query.includes("{language}")) return null;
  let endpoint;
  try { endpoint = new URL(timeline.sourceUrl); } catch { return null; }
  if (endpoint.protocol !== "https:" || endpoint.username || endpoint.password ||
      endpoint.hash || endpoint.search) return null;
  endpoint.searchParams.set("query", query.replaceAll("{subject}", timeline.subjectQid)
    .replaceAll("{language}", language));
  endpoint.searchParams.set("format", "json");
  return endpoint.href;
}

function releaseTimelineBindingIdentifier(binding) {
  if (binding?.type !== "uri" || typeof binding.value !== "string") return "";
  return binding.value.match(/^https?:\/\/www\.wikidata\.org\/entity\/(Q[0-9]{1,18})$/u)?.[1] ?? "";
}

function releaseTimelineCapturedEntries(text, subject, language) {
  if (new TextEncoder().encode(text).length > 256 * 1024) return null;
  let document;
  try { document = JSON.parse(text); } catch { return null; }
  const rows = document?.results?.bindings;
  if (!Array.isArray(rows) || !rows.length || rows.length > 100) return null;
  const seen = new Set();
  const entries = [];
  for (const row of rows) {
    if (releaseTimelineBindingIdentifier(row?.subject) !== subject) return null;
    const qid = releaseTimelineBindingIdentifier(row.film);
    if (!qid || seen.has(qid)) return null;
    seen.add(qid);
    let releaseDate = "";
    if (row.firstRelease !== undefined) {
      const binding = row.firstRelease;
      if (binding?.type !== "literal" || binding.datatype !== "http://www.w3.org/2001/XMLSchema#dateTime") return null;
      const timestamp = binding.value;
      releaseDate = typeof timestamp === "string" ? timestamp.match(/^(\d{4}-\d{2}-\d{2})T/u)?.[1] : null;
      if (!releaseDate || releaseTimelineDays(releaseDate) === null) return null;
    }
    const titles = [["localTitle", language], ["title", "en"]].flatMap(([field, code]) => {
      const binding = row[field];
      const locale = binding?.["xml:lang"];
      if (binding?.type !== "literal" || (locale !== code && !(field === "title" && locale === "mul"))) return [];
      const value = binding.value;
      const title = typeof value === "string" ? value.trim() : "";
      return title && [...title].length <= 300 ? [{ language: code, text: title }] : [];
    });
    if (!titles.length) return null;
    entries.push({ qid, releaseDate, titles });
  }
  return entries;
}

async function tryReleaseTimeline(prompt, normalized, preferences, trace) {
  if (preferences?.externalServiceWikidata === false) return null;
  const language = detectLanguage(prompt);
  const registry = releaseTimelineRegistry();
  const gated = gatedFactRecord(normalized, prompt);
  const slug = gated?.record.releaseTimeline;
  const online = preferences?.offline !== true && preferences?.allowNetwork !== false;
  const options = { online, bounded: true, indexed: true, maxAgeSeconds: 3600 };
  const timeline = slug ? registry.timelines.find(candidate => candidate.slug === slug) :
    await openReleaseTimeline(normalized, language, registry, options, trace);
  if (!timeline) return null;
  const source = releaseTimelineQueryUrl(timeline, registry.query, language);
  if (!source) return null;
  trace.push(`release_timeline:request:${source}`);
  const capture = await sourceWalkFetchCapture(source, options);
  if (!capture.ok) {
    trace.push(`release_timeline:unavailable:${capture.error}`);
    return null;
  }
  const entries = releaseTimelineCapturedEntries(capture.text, timeline.subjectQid, language);
  if (!entries) {
    trace.push("release_timeline:rejected:invalid_publisher_rows");
    return null;
  }
  const fetched = new Date(Number(capture.fetchedAt) * 1000);
  if (!Number.isFinite(fetched.getTime()) || fetched.getUTCFullYear() > 9999 || fetched.getTime() < 0) return null;
  const retrievedAt = fetched.toISOString().slice(0, 10);
  const today = new Date().toISOString().slice(0, 10);
  const rendered = renderReleaseTimeline({ ...timeline, entries, retrievedAt }, language, today);
  if (!rendered) return null;
  return {
    intent: "fact_lookup", content: rendered.text, confidence: 0.9,
    evidence: [
      "response:fact_lookup", `source:${source}`, `release_timeline:${timeline.slug}`,
      ...(slug ? [`fact_lookup:hit:${gated.record.slug}`, `fact_lookup:subject_gate:${gated.gate}`] : timeline.searchEvidence),
      `wikidata:${timeline.subjectQid}`,
      `release_timeline:snapshot:${retrievedAt}`, `release_timeline:sha256:${capture.sha256}`,
      `release_timeline:released:${rendered.released.length}`,
      `release_timeline:announced:${rendered.announced.length}`,
      ...(rendered.stale ? [`release_timeline:stale:${retrievedAt}`] : []),
      `source:http:${source} fetched_at=${capture.fetchedAt} sha256=${capture.sha256} cached=${capture.cached}`,
    ],
  };
}
function releaseTimelineSeriesSubject(normalized, frame) {
  const [prefix, suffix] = frame.split("{subject}");
  if (suffix === undefined || !normalized.startsWith(prefix) || !normalized.endsWith(suffix) ||
      normalized.length <= prefix.length + suffix.length) return "";
  const subject = normalized.slice(prefix.length, normalized.length - suffix.length).trim();
  return [...subject].length <= 120 && subject.split(/\s+/u).length <= 12 ? subject : "";
}

// Whether a captured subject holds a word beyond the request's own frames and
// the seeded function words (Rust `names_a_series`): "List films in release
// order" fits "{subject} films in release order" with the directive "list" as
// its subject, which names no series (issue #1175).
function releaseTimelineNamesASeries(subject, frames) {
  const frameWords = new Set(frames.flatMap(frame => frame.replace("{subject}", " ").split(/\s+/u)).filter(Boolean));
  return subject.split(/\s+/u)
    .map(word => word.replace(/^[!-/:-@[-`{-~]+|[!-/:-@[-`{-~]+$/gu, ""))
    .filter(Boolean)
    .some(word => {
      const lowered = word.toLowerCase();
      return !frameWords.has(lowered) && claimEvidenceContentBeyond(lowered, []);
    });
}

function releaseTimelineOpenSubject(normalized, request) {
  return request.frames.map(frame => releaseTimelineSeriesSubject(normalized, frame))
    .find(subject => subject && releaseTimelineNamesASeries(subject, request.frames)) || "";
}

async function openReleaseTimeline(normalized, language, registry, options, trace) {
  const series = registry.openSeries;
  const request = series?.requests.find(entry => entry.language === language);
  if (!request) return null;
  const subject = releaseTimelineOpenSubject(normalized, request);
  if (!subject) return null;
  const captures = [];
  const query = request.search.replace("{subject}", subject);
  let candidates = await wikidataSearchEntities(query, language, options, captures);
  if (!candidates.length) candidates = await wikidataSearchEntities(subject, language, options, captures);
  if (candidates.length !== 1) {
    trace.push("release_timeline:rejected:unresolved_series");
    return null;
  }
  return {
    ...series,
    subjectQid: candidates[0].qid,
    subjects: [{ language, text: request.subject.replace("{subject}", candidates[0].label || subject) }],
    searchEvidence: captures.map(({ url, sha256 }) => `source:http:${url} sha256=${sha256}`),
  };
}
