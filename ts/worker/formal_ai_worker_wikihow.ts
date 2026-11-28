// wikiHow publisher discovery, article verification, and method selection.

function howToAccessibilityKey(record, url) {
  const { hostname } = new URL(url);
  return record.id === "wikihow" && hostname !== "www.wikihow.com" ? `${record.id}@${hostname}` : record.id;
}

function howToWikiHowArticle(apiUrl) {
  const api = new URL(apiUrl);
  return new URL("/" + sourceWalkPercentEncode(api.searchParams.get("page") || ""), api.origin);
}

function howToSameWikiHowArticle(value, expected) {
  try {
    const url = new URL(value);
    return ["http:", "https:"].includes(url.protocol) && url.host === expected.host
      && !url.username && !url.password
      && decodeURIComponent(url.pathname) === decodeURIComponent(expected.pathname);
  } catch {
    return false;
  }
}

function howToWaybackSnapshot(text, article) {
  const closest = JSON.parse(text)?.archived_snapshots?.closest;
  if (!closest?.available || String(closest.status) !== "200" || !/^\d{14}$/u.test(closest.timestamp)) {
    return null;
  }
  const replay = new URL(closest.url);
  const prefix = `/web/${closest.timestamp}/`;
  if (replay.host !== "web.archive.org" || !replay.pathname.startsWith(prefix)
      || !howToSameWikiHowArticle(replay.pathname.slice(prefix.length), article)) return null;
  return `https://web.archive.org/web/${closest.timestamp}id_/${article.href}`;
}

function howToWikiHowHtmlSteps(html, article, task, language, limit) {
  const canonical = html.match(/<link\b[^>]*\brel=["']canonical["'][^>]*\bhref=["']([^"']+)["']/iu);
  if (!canonical || !howToSameWikiHowArticle(canonical[1], article)) return [];
  return howToWikiHowStepLists(html, task, language, limit, false);
}

function howToMethodRank(label, task, language, markers) {
  const words = label.toLowerCase();
  const hasScope = (text, marker) => language === "zh" ? text.includes(marker)
    : text.split(/[^\p{L}\p{N}\p{M}]+/u).some(word => word.startsWith(marker));
  let rank = 1;
  for (const marker of markers) {
    if (!hasScope(words, marker)) continue;
    if (hasScope(task, marker)) return 3;
    rank = 0;
  }
  return rank > 0 && howToMatchesTask(task, label, language) ? 2 : rank;
}

function howToWikiHowStepLists(html, task, language, limit, allowLegacy = true) {
  const policy = handlerRulesPolicy("procedural_how_to", `scoped-method-markers-${language}`);
  const markers = crateModule("crate/seed.mjs").splitPipeList(policy || "").filter(Boolean);
  const requested = task.toLowerCase();
  const tags = html.matchAll(/<h[1-6](?:\s[^>]*)?>(.*?)<\/h[1-6]>|<(\/?)(ol|ul|li)(?:\s([^>]*))?>/gisu);
  let best = [];
  let bestRank = -1;
  let rank = 1;
  let depth = 0;
  let items;
  let itemStart;
  let sawOrderedList = false;
  for (const tag of tags) {
    const [raw, heading, closing, kind, attributes = ""] = tag;
    if (heading !== undefined) {
      if (depth === 0) {
        const headline = heading.match(/<span class="mw-headline"[^>]*>(.*?)<\/span>/isu);
        rank = howToMethodRank(howToCompactStepText(headline?.[1] ?? heading), requested, language, markers);
      }
    } else if (kind.toLowerCase() === "li") {
      if (depth !== 1 || !items) continue;
      if (!closing) {
        itemStart = tag.index + raw.length;
      } else if (itemStart !== undefined && items.length < limit) {
        const item = html.slice(itemStart, tag.index);
        const body = allowLegacy ? null : /<div\b[^>]*\bclass=["']step["'][^>]*>/iu.exec(item);
        const inner = body ? item.slice(body.index + body[0].length)
          .replace(/<sup\b[^>]*\bclass=["']reference\b[^>]*>[\s\S]*?<\/sup>/giu, "") : item;
        const step = howToCompactStepText(inner);
        if ((body || !inner.trimStart().startsWith("<b>")) && Array.from(step).length >= HOW_TO_MIN_STEP_CHARS
            && !items.includes(step)) items.push(step);
        itemStart = undefined;
      }
    } else if (closing) {
      depth = Math.max(0, depth - 1);
      if (depth !== 0 || !items) continue;
      if (items.length >= HOW_TO_MIN_ACCEPTED_STEPS && rank > bestRank) {
        best = items;
        bestRank = rank;
      }
      items = undefined;
      itemStart = undefined;
    } else {
      if (depth === 0 && kind.toLowerCase() === "ol") {
        sawOrderedList = true;
        if (attributes.includes("steps_list_2") || (allowLegacy && !attributes.trim())) items = [];
      }
      depth += 1;
    }
  }
  return allowLegacy && !sawOrderedList ? howToExtractSteps(html, limit) : best;
}

function howToPublisherSearchQuery(task, language) {
  const prefix = handlerRulesPolicy("procedural_how_to", `aspect-prefix-${language}`);
  if (!prefix) return task;
  const endings = String(handlerRulesPolicy("procedural_how_to", `infinitive-endings-${language}`) || "")
    .split(/\s+/u).filter(Boolean);
  const infinitive = word => endings.some(ending => word.endsWith(ending)
    && Array.from(word.slice(0, -ending.length)).length >= 3);
  const word = task.split(/[^\p{L}\p{N}\p{M}]+/u).find(infinitive);
  if (!word) return task;
  const alternate = word.startsWith(prefix) ? word.slice(prefix.length) : prefix + word;
  return infinitive(alternate) ? `${task} ${alternate}` : task;
}

function howToPublisherResultArticle(href, endpoint) {
  try {
    const url = new URL(href, endpoint);
    if (url.origin !== endpoint.origin || url.username || url.password || url.search || url.hash) return null;
    const slug = decodeURIComponent(url.pathname.slice(1));
    return slug && !/[/:�]/u.test(slug) ? { title: slug.replace(/[-_]/gu, " "), url } : null;
  } catch {
    return null;
  }
}

async function howToPublisherCapture(url, online) {
  const capture = await sourceWalkFetchCapture(url, { bounded: true, online });
  if (capture.ok || !online) return capture;
  const proxy = await howToCaptureThroughComponent(url, []);
  if (!proxy) return capture;
  const accepted = Object.freeze({ ...proxy, ok: true });
  sourceWalkCaptureCache.set(url, accepted);
  return accepted;
}

async function howToSearchWikiHowPublisher(walked, record, outcome, exact, task, language, online, now) {
  const { bounds } = walked;
  const remaining = bounds.maxPagesPerService - outcome.pages;
  if (bounds.maxDepth < 2 || remaining < 2) return;
  const endpoint = new URL(exact);
  const query = howToPublisherSearchQuery(task, language);
  const search = new URL(`/Special:LSearch?fulltext=Search&search=${sourceWalkPercentEncode(query)}`, endpoint).href;
  const capture = await howToPublisherCapture(search, online);
  const note = (detail) => {
    outcome.detail = [outcome.detail, detail].filter(Boolean).join(" | ");
  };
  if (!capture.ok) {
    note(`publisher_search_failed url=${search} reason=${capture.error}`);
    return;
  }
  outcome.pages += 1;
  let attempts = remaining - 1;
  const seen = new Set();
  for (const [, href] of capture.text.matchAll(/<a\s+class="result_link"\s+href="([^"]+)"/gu)) {
    const article = howToPublisherResultArticle(href, endpoint);
    if (!article || !howToMatchesTask(task, article.title, language)) continue;
    const api = howToParseUrl({ api: exact }, article.title);
    if (seen.has(article.url.href)) continue;
    seen.add(article.url.href);
    if (attempts === 0) break;
    const page = api === exact ? null : await howToPublisherCapture(api, online);
    if (page) attempts -= 1;
    if (page?.ok) outcome.pages += 1;
    const parsed = page?.ok ? howToReadCapture({ ...record, api }, task, page, 2, 0, bounds, language) : null;
    const items = parsed?.items ?? [];
    let source = api;
    if (items.length < HOW_TO_MIN_ACCEPTED_STEPS) {
      if (page) note(`publisher_parse_failed url=${api} reason=${page.error || parsed?.detail || "insufficient_steps"}`);
      if (attempts === 0) break;
      attempts -= 1;
      source = article.url.href;
      const html = await howToPublisherCapture(source, online);
      if (html.ok) outcome.pages += 1;
      const steps = html.ok ? howToWikiHowHtmlSteps(html.text, article.url, task, language, bounds.maxItems) : [];
      if (steps.length < HOW_TO_MIN_ACCEPTED_STEPS) {
        note(`publisher_article_failed url=${source} reason=${html.error || "no_matching_steps"}`);
        continue;
      }
      items.length = 0;
      howToPushSteps(record, html, 2, steps, items);
    }
    walked.items = walked.items.filter((item) => item.sourceId !== record.id).concat(items);
    outcome.status = "contributed";
    outcome.items = items.length;
    note(`publisher_search search=${search} url=${source}`);
    sourceWalkObserveService(howToAccessibilityKey(record, exact), "reachable", `captured ${source}`, now);
    return;
  }
}

async function howToRecoverWikiHow(walked, task, language, preferences, discovered, now) {
  const outcome = walked.outcomes.find((item) => item.sourceId === "wikihow");
  if (!outcome || ["disabled", "unbound_template"].includes(outcome.status)
      || outcome.items >= HOW_TO_MIN_ACCEPTED_STEPS
      || outcome.pages >= walked.bounds.maxPagesPerService) return;

  const record = howToSourceRegistry().find((item) => item.id === "wikihow");
  const exact = sourceWalkEntryUrl(record, task, language);
  if (!exact) return;
  const online = preferences?.offline !== true && preferences?.allowNetwork !== false;
  if (online && discovered.size === 0 && outcome.pages === 0 && walked.bounds.maxPagesPerService > 1
      && walked.bounds.maxDepth > 0 && self.FormalAIWebSearchComponent?.fetchUrl) {
    const notes = [];
    const api = howToSearchUrl({ api: exact }, task);
    outcome.pages += 1;
    const search = await howToCaptureThroughComponent(api, notes);
    if (search) {
      const read = howToReadCapture({ ...record, api }, task, search, 0, 0, walked.bounds, language);
      for (const url of read.follow) {
        const title = new URL(url).searchParams.get("page");
        if (title) discovered.add(sourceWalkEntryUrl(record, title, language));
      }
      if (read.detail) notes.push(read.detail);
    }
    outcome.detail = [outcome.detail, ...notes].filter(Boolean).join(" | ");
  }
  if (discovered.size === 0) {
    await howToSearchWikiHowPublisher(walked, record, outcome, exact, task, language, online, now);
  }
  if (!online || outcome.items >= HOW_TO_MIN_ACCEPTED_STEPS) return;
  discovered.add(exact);
  for (const candidate of discovered) {
    if (!candidate) continue;
    if (outcome.pages >= walked.bounds.maxPagesPerService || outcome.items >= HOW_TO_MIN_ACCEPTED_STEPS) break;
    await howToRecoverWikiHowArticle(walked, record, outcome, howToWikiHowArticle(candidate), task, language);
  }
}

async function howToRecoverWikiHowArticle(walked, record, outcome, article, task, language) {
  const notes = [];
  const index = await sourceWalkFetchCapture(
    `https://archive.org/wayback/available?url=${sourceWalkPercentEncode(article.href)}`, { bounded: true });
  const candidates = [];
  if (index.ok) {
    try {
      const snapshot = howToWaybackSnapshot(index.text, article);
      if (snapshot) candidates.push(snapshot);
      else notes.push("wayback:no_matching_snapshot");
    } catch {
      notes.push("wayback:invalid_availability");
    }
  } else {
    notes.push(`wayback:availability_failed ${index.error}`);
  }
  candidates.push(article.href);
  for (const url of candidates) {
    if (outcome.pages >= walked.bounds.maxPagesPerService) break;
    outcome.pages += 1;
    let capture = url === article.href ? null : await sourceWalkFetchCapture(url, { bounded: true });
    if (!capture?.ok) {
      if (capture) notes.push(`wayback:replay_failed ${capture.error}`);
      capture = await howToCaptureThroughComponent(url, notes);
    }
    if (!capture?.ok && url === article.href && !capture?.text) {
      capture = await sourceWalkFetchCapture(url, { bounded: true });
    }
    if (!capture?.text) {
      notes.push(`no_capture url=${url}`);
      continue;
    }
    const steps = howToWikiHowHtmlSteps(capture.text, article, task, language, walked.bounds.maxItems);
    if (steps.length < HOW_TO_MIN_ACCEPTED_STEPS) {
      notes.push(`no_matching_steps url=${url}`);
      continue;
    }
    const recovered = [];
    howToPushSteps(record, capture, 0, steps, recovered);
    walked.items = walked.items.filter((item) => item.sourceId !== record.id).concat(recovered);
    outcome.status = "contributed";
    outcome.items = recovered.length;
    notes.push(`recovered url=${url}`);
    break;
  }
  outcome.detail = [outcome.detail, ...notes].filter(Boolean).join(" | ");
}
