// Formalize a fetched page in the browser worker (R1188-U18).
//
// The browser twin of `try_page_formalization_with_client` and
// `answer_page_formalization` in rust/src/solver_handlers/web_requests.rs.
// "formalize <url>" (and "formalize this page: <url>") is a fetch request:
// the `page_formalization` meaning of data/seed/meanings-web-navigation.lino
// carries both the `http_fetch` role, so the fetch recognizer claims the
// prompt, and the `page_formalization_action` role, read here, which turns
// the fetched page into statements instead of showing its body. The page is
// fetched through the URL-fetch path `tryFetch` uses
// (`FormalAIWebSearchComponent.fetchUrl`); its prose is read by the JavaScript
// root's own page formalizer (js/agentic/crate/page_formalization.mjs, through
// formal_ai_worker_crate_modules.js), and the answer is its `pageAnswer`, in
// the request's language. With no network, or when the fetch fails, the answer
// says so and nothing is formalized.

/** The role naming a request to formalize a page (Rust `ROLE_PAGE_FORMALIZATION_ACTION`). */
const ROLE_PAGE_FORMALIZATION_ACTION = "page_formalization_action";

/** The crate module that formalizes a page and renders the answer. */
const PAGE_FORMALIZATION_MODULE = "crate/page_formalization.mjs";

/** The page blocks that are prose (Rust `page_prose`). */
const PAGE_PROSE_KINDS = Object.freeze(["paragraph", "list_item"]);

/**
 * Whether the request asks for the page at its URL to be formalized (Rust
 * `is_page_formalization_prompt`).
 * @param {string} prompt
 * @param {string} normalized
 * @returns {boolean}
 */
function isPageFormalizationPrompt(prompt, normalized) {
  const raw = String(prompt || "").trimStart().toLowerCase();
  return roleEvidencesWebIntent(ROLE_PAGE_FORMALIZATION_ACTION, [String(normalized || ""), raw]);
}

/**
 * The prose of a fetched page: the text of its paragraphs and list items, one
 * block per line, in page order (Rust `web_formalize::page_prose`).
 * @param {string} text
 * @param {string} url
 * @returns {string}
 */
function pageProse(text, url) {
  return formalizePage(text, null, url).blocks
    .filter((block) => PAGE_PROSE_KINDS.includes(block.kind) && String(block.text).trim() !== "")
    .map((block) => block.text)
    .join("\n");
}

/**
 * The seeded response `intent` in `language` with `values` filled in (Rust
 * `seed::render_response`).
 * @param {string} intent
 * @param {string} language
 * @param {Record<string, string|number>} values
 * @returns {string}
 */
function pageFormalizationResponse(intent, language, values) {
  let text = answerFor(intent, language);
  for (const [name, value] of Object.entries(values)) text = text.split(`{${name}}`).join(String(value));
  return text;
}

/**
 * The page the request names, fetched, or why it could not be.
 * @param {string} url
 * @param {Array<string>} evidence
 * @returns {Promise<{text: string}|{offline: true}|{error: string}>}
 */
async function pageFormalizationFetch(url, evidence) {
  const offline = typeof fetch !== "function" || (self.navigator && self.navigator.onLine === false);
  if (offline) return { offline: true };
  try {
    const response = await self.FormalAIWebSearchComponent.fetchUrl(url, evidence);
    if (!response || !response.ok) return { error: `HTTP ${response ? response.status : 0}` };
    return { text: await response.text() };
  } catch (error) {
    return { error: error instanceof Error ? error.message : String(error) };
  }
}

/**
 * Formalize the page a request names (Rust `try_page_formalization_with_client`):
 * null when the request names no URL or does not ask for a formalization.
 * @param {string} prompt
 * @returns {Promise<object|null>}
 */
async function tryPageFormalization(prompt) {
  const normalized = normalizePrompt(prompt);
  const url = extractHttpFetchUrl(prompt, normalized);
  if (url === null || !isPageFormalizationPrompt(prompt, normalized)) return null;
  const language = detectLanguage(prompt);
  const evidence = [`page_formalization:request:${url}`];
  const fetched = await pageFormalizationFetch(url, evidence);
  const answer = (content, link, confidence) => ({
    intent: "page_formalization",
    content,
    confidence,
    evidence: [...evidence, link],
  });
  if (fetched.offline) {
    const content = pageFormalizationResponse("page_formalization_offline", language, { url });
    return answer(content, "response:page_formalization_unavailable", 0);
  }
  if (fetched.error !== undefined) {
    evidence.push(`error:fetch:${fetched.error}`);
    const content = pageFormalizationResponse("page_formalization_fetch_failed", language, { url, error: fetched.error });
    return answer(content, "response:page_formalization_unavailable", 0);
  }
  const prose = pageProse(fetched.text, url);
  const { formalizePage: formalizePageText, pageAnswer } = crateModule(PAGE_FORMALIZATION_MODULE);
  const report = formalizePageText(prose, detectLanguage(prose));
  evidence.push(
    `page_formalization:report:sentences=${report.sentences.length} statements=${report.statements} `
      + `covered=${report.covered} terms=${report.terms} unknown=${report.unknown}`,
  );
  const render = (intent, values) => pageFormalizationResponse(intent, language, values);
  return answer(pageAnswer(report, url, render), "response:page_formalization", 0.9);
}
