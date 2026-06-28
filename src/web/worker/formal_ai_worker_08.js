// Worker module 9 of 21. Loaded by ../formal_ai_worker.js.
function startsWithProofVerb(normalized, verb) {
  if (!normalized.startsWith(verb)) return false;
  const tail = normalized.slice(verb.length);
  if (!tail) return true;
  return !/[\p{L}\p{N}]/u.test(tail.charAt(0));
}

function hasProofRequestShape(normalized) {
  const text = String(normalized || "").trim();
  if (!text) return false;
  // A proof request is recognised structurally from the meaning lexicon, not
  // from words baked into this file: a clause-initial bare directive verb
  // (proof_directive, with the verb-boundary check), a request-frame lead in any
  // language that needs no `that` clause (proof_request_lead), or a mid-prompt
  // proof assertion marker in any language (proof_marker).
  return (
    bareLiterals(ROLE_PROOF_DIRECTIVE).some((verb) => startsWithProofVerb(text, verb)) ||
    prefixLiterals(ROLE_PROOF_REQUEST_LEAD).some((lead) => text.startsWith(lead)) ||
    lexiconMentionsRoleSubstring(ROLE_PROOF_MARKER, text)
  );
}

function extractProofClaim(normalized) {
  const trimmed = String(normalized || "").trim();
  // The claim scaffolds (each ending in the … slot) come from the
  // proof_claim_scaffold role in declaration order, so the first matching
  // prefix wins exactly as before — every that/что/कि variant is listed ahead
  // of its shorter sibling in the lexicon. Comma variants are absent: the
  // normaliser rewrites the comma to a space, making them unreachable here.
  const prefixes = prefixLiterals(ROLE_PROOF_CLAIM_SCAFFOLD);
  for (const prefix of prefixes) {
    if (trimmed.startsWith(prefix)) {
      return stripProofClaimNoise(trimmed.slice(prefix.length));
    }
  }
  for (const prefix of prefixes) {
    const index = trimmed.indexOf(prefix);
    if (index <= 0) continue;
    const before = trimmed.charAt(index - 1);
    if (isProofIntroBoundary(before)) {
      return stripProofClaimNoise(trimmed.slice(index + prefix.length));
    }
  }
  return trimmed;
}

function matchesEuclidPrimeClaim(claim) {
  const lower = String(claim || "").toLowerCase();
  return (
    lower.includes("infinitely many primes") ||
    lower.includes("infinitely many prime numbers") ||
    lower.includes("infinitude of primes") ||
    lower.includes("prime numbers are infinite") ||
    lower.includes("euclid") ||
    lower.includes("евклид") ||
    (lower.includes("прост") && lower.includes("бесконеч")) ||
    lower.includes("अनंत अभाज्य") ||
    lower.includes("अनन्त अभाज्य") ||
    lower.includes("अभाज्य संख्याएँ अनंत") ||
    lower.includes("अभाज्य संख्याएं अनंत") ||
    lower.includes("अभाज्य संख्याएँ अनन्त") ||
    lower.includes("अभाज्य संख्याएं अनन्त") ||
    lower.includes("无穷多素数") ||
    lower.includes("无穷多个素数") ||
    lower.includes("素数有无穷") ||
    lower.includes("素数无穷") ||
    lower.includes("無窮多素數") ||
    lower.includes("素數有無窮") ||
    lower.includes("素數無窮") ||
    lower.includes("欧几里得")
  );
}

function euclidPrimeProofBody(language) {
  if (language === "ru") {
    return [
      "Как я понял запрос: трактуем запрос как формальное утверждение «Простых чисел бесконечно много.» и доказываем методом «от противного» в relative-meta-logic.",
      "",
      "Доказательство (метод: от противного).",
      "",
      "Утверждение: Простых чисел бесконечно много.",
      "1. Работаем в элементарной теории чисел, формализуемой в арифметике Пеано (PA): простое число — это целое число больше 1, положительные делители которого только 1 и оно само. В доказательстве используется теорема PA: у каждого целого числа больше 1 есть простой делитель; это формальный контекст для тактики от противного в relative-meta-logic.",
      "2. Предположим противное: простых чисел конечное число; обозначим их p₁, p₂, …, pₙ.",
      "3. Рассмотрим число N = p₁·p₂·…·pₙ + 1. Это целое число, большее единицы.",
      "4. По основной теореме арифметики у N есть простой делитель q. Если q = pᵢ для некоторого i, то pᵢ делит и p₁·p₂·…·pₙ, и N, а значит делит их разность, равную 1 — противоречие.",
      "5. Значит, q — простое число, не входящее в список p₁, …, pₙ, что противоречит предположению о полноте списка.",
      "",
      "Предположение несостоятельно, следовательно простых чисел бесконечно много. ∎",
    ].join("\n");
  }
  if (language === "hi") {
    return [
      "मैंने प्रश्न को कैसे समझा: प्रश्न को औपचारिक कथन \"अभाज्य संख्याएँ अनंत हैं।\" मानकर relative-meta-logic में \"विरोधाभास\" विधि से प्रमाणित कर रहे हैं।",
      "",
      "प्रमाण (विधि: विरोधाभास)।",
      "",
      "कथन: अभाज्य संख्याएँ अनंत हैं।",
      "1. हम प्राथमिक संख्या-सिद्धांत में काम करते हैं, जिसे Peano arithmetic (PA) में औपचारिक किया जा सकता है: अभाज्य वह पूर्णांक है जो 1 से बड़ा है और जिसके धनात्मक भाजक केवल 1 और वही संख्या हैं। प्रमाण PA के इस प्रमेय का उपयोग करता है कि 1 से बड़े हर पूर्णांक का कोई अभाज्य भाजक होता है; यही relative-meta-logic की contradiction युक्ति का औपचारिक संदर्भ है।",
      "2. विरोधाभास हेतु मान लीजिए कि अभाज्य संख्याएँ केवल सीमित संख्या में हैं: p₁, p₂, …, pₙ।",
      "3. संख्या N = p₁·p₂·…·pₙ + 1 लीजिए। N एक से बड़ा पूर्णांक है।",
      "4. अंकगणित की मूल प्रमेय से N का कोई अभाज्य भाजक q है। यदि किसी i के लिए q = pᵢ हो, तो pᵢ संख्या p₁·p₂·…·pₙ और N दोनों को विभाजित करेगा, अर्थात उनका अंतर 1 भी विभाजित करेगा — असंभव।",
      "5. अतः q एक ऐसा अभाज्य है जो सूची p₁, …, pₙ में नहीं है, जो सूची के पूर्ण होने की परिकल्पना का खंडन करता है।",
      "",
      "अतः परिकल्पना असत्य है और अभाज्य संख्याएँ अनंत हैं। ∎",
    ].join("\n");
  }
  if (language === "zh") {
    return [
      "对问题的理解: 把问题视为形式命题“素数有无穷多个。”, 在 relative-meta-logic 中用“反证法”方法证明。",
      "",
      "证明 (方法: 反证法)。",
      "",
      "命题: 素数有无穷多个。",
      "1. 在可由 Peano arithmetic (PA) 形式化的初等数论中工作: 素数是大于 1 的整数, 其正因数只有 1 和自身。证明使用 PA 中的定理: 每个大于 1 的整数都有素因数; 这就是 relative-meta-logic 反证策略的形式上下文。",
      "2. 假设素数仅有有限多个, 记为 p₁、p₂、…、pₙ。",
      "3. 构造数 N = p₁·p₂·…·pₙ + 1。N 是大于 1 的整数。",
      "4. 由算术基本定理, N 有一个素因数 q。若 q = pᵢ, 则 pᵢ 同时整除 p₁·p₂·…·pₙ 与 N, 因而整除二者之差 1, 矛盾。",
      "5. 因此 q 是不在 p₁, …, pₙ 中的素数, 与假设矛盾。",
      "",
      "假设不成立, 故素数有无穷多个。∎",
    ].join("\n");
  }
  return [
    "How I interpreted the request: treating the request as the formal claim \"There are infinitely many prime numbers.\" and discharging it by contradiction inside relative-meta-logic.",
    "",
    "Proof (method: contradiction).",
    "",
    "Statement: There are infinitely many prime numbers.",
    "1. Work in elementary number theory, formalizable in Peano arithmetic (PA): a prime is an integer greater than 1 whose only positive divisors are 1 and itself. The proof uses the PA theorem that every integer greater than 1 has a prime divisor; this is the formal context for the relative-meta-logic contradiction tactic.",
    "2. Assume for contradiction that only finitely many primes exist; call them p₁, p₂, …, pₙ.",
    "3. Form the number N = p₁·p₂·…·pₙ + 1. Then N is an integer greater than 1.",
    "4. By the fundamental theorem of arithmetic, N has a prime divisor q. If q = pᵢ for some i, then pᵢ divides both p₁·p₂·…·pₙ and N, so pᵢ divides their difference, which is 1 — impossible.",
    "5. Hence q is a prime not in the list p₁, …, pₙ, contradicting the assumption that the list was complete.",
    "",
    "The assumption fails, so there are infinitely many primes. ∎",
  ].join("\n");
}

function genericProofPlanBody(prompt, language) {
  if (language === "ru") {
    return [
      "Как я понял запрос: утверждение нужно доказать, но для финального исполнения не хватает выбранной формальной системы.",
      "",
      "План доказательства (метод: формализация и проверка).",
      "",
      `Утверждение: ${String(prompt || "").trim()}`,
      "1. Зафиксировать систему аксиом, например PA для арифметики, ZFC для теории множеств или конкретную теорию предметной области.",
      "2. Перевести утверждение в закрытую формулу этой системы.",
      "3. Выбрать тактику relative-meta-logic: rewrite, induction, contradiction или поиск контрпримера.",
      "4. Проверить каждый шаг и вернуть либо сертификат доказательства, либо контрпример.",
      "",
      "Чтобы выполнить доказательство полностью, нужен явный набор аксиом и точная формулировка утверждения.",
    ].join("\n");
  }
  if (language === "hi") {
    return [
      "मैंने प्रश्न को कैसे समझा: यह प्रमाण का अनुरोध है, पर अंतिम निष्पादन के लिए चुनी हुई औपचारिक प्रणाली चाहिए।",
      "",
      "प्रमाण योजना (विधि: औपचारिकरण और सत्यापन)।",
      "",
      `कथन: ${String(prompt || "").trim()}`,
      "1. कोई अभिगृहीत प्रणाली चुनें, जैसे arithmetic के लिए PA, set theory के लिए ZFC, या किसी क्षेत्र-विशेष की theory।",
      "2. कथन को उस प्रणाली के बंद सूत्र में अनुवादित करें।",
      "3. relative-meta-logic की tactic चुनें: rewrite, induction, contradiction, या counterexample search।",
      "4. प्रत्येक चरण जाँचें और proof certificate या counterexample लौटाएँ।",
      "",
      "प्रमाण पूरा करने के लिए सटीक axiom set और closed statement चाहिए।",
    ].join("\n");
  }
  if (language === "zh") {
    return [
      "对问题的理解: 该提示要求证明, 但最终执行需要选定的形式系统。",
      "",
      "证明计划 (方法: 形式化与验证)。",
      "",
      `命题: ${String(prompt || "").trim()}`,
      "1. 固定一个公理系统, 例如 arithmetic 用 PA, set theory 用 ZFC, 或某个领域专用理论。",
      "2. 将命题翻译成该系统中的闭公式。",
      "3. 选择 relative-meta-logic 策略: rewrite、induction、contradiction 或 counterexample search。",
      "4. 检查每一步, 并返回证明证书或反例。",
      "",
      "要完成证明, 需要精确的 axiom set 和 closed statement。",
    ].join("\n");
  }
  return [
    "How I interpreted the request: the prompt asks for a proof, but final execution needs a chosen formal system.",
    "",
    "Proof plan (method: formalization and verification).",
    "",
    `Statement: ${String(prompt || "").trim()}`,
    "1. Fix an axiom system, for example PA for arithmetic, ZFC for set theory, or a domain-specific theory.",
    "2. Translate the claim into a closed formula in that system.",
    "3. Choose a relative-meta-logic tactic: rewrite, induction, contradiction, or counterexample search.",
    "4. Check each step and return either a proof certificate or a counterexample.",
    "",
    "To finish the proof, provide the exact axiom set and a closed statement.",
  ].join("\n");
}

function tryProofRequest(prompt, normalized, language) {
  if (!hasProofRequestShape(normalized)) return null;
  const claim = extractProofClaim(normalized);
  if (matchesEuclidPrimeClaim(claim)) {
    return {
      intent: "proof_request",
      content: euclidPrimeProofBody(language),
      confidence: 0.85,
      evidence: [
        "policy:proof_request",
        "pipeline:planned:relative-meta-logic",
        "proof_outcome:proven",
        "proof_method:contradiction",
        `language:${language}`,
      ],
    };
  }
  return {
    intent: "proof_request",
    content: genericProofPlanBody(prompt, language),
    confidence: 0.6,
    evidence: [
      "policy:proof_request",
      "pipeline:planned:relative-meta-logic",
      "proof_outcome:partial_plan",
      `language:${language}`,
    ],
  };
}

// WEEKDAY_CYCLE keeps only the rendering surfaces (display names + Russian
// case forms) used to phrase an answer. Issue #386: the *recognition* words
// (the former `aliases`, plus the next/previous/today/day/question markers)
// are no longer hardcoded here — they live as self-describing meanings in
// data/seed/meanings-calendar.lino under the `calendar_*` roles, embedded
// below in MEANINGS_LINO. The detection functions query the lexicon by role
// and map a matched weekday slug back to its cycle entry; mirrors
// src/solver_handlers/calendar.rs.
const WEEKDAY_CYCLE = [
  {
    slug: "monday",
    en: "Monday",
    ru: "понедельник",
    hi: "सोमवार",
    zh: "星期一",
    ruGenitive: "понедельника",
    ruInstrumental: "понедельником",
  },
  {
    slug: "tuesday",
    en: "Tuesday",
    ru: "вторник",
    hi: "मंगलवार",
    zh: "星期二",
    ruGenitive: "вторника",
    ruInstrumental: "вторником",
  },
  {
    slug: "wednesday",
    en: "Wednesday",
    ru: "среда",
    hi: "बुधवार",
    zh: "星期三",
    ruGenitive: "среды",
    ruInstrumental: "средой",
  },
  {
    slug: "thursday",
    en: "Thursday",
    ru: "четверг",
    hi: "गुरुवार",
    zh: "星期四",
    ruGenitive: "четверга",
    ruInstrumental: "четвергом",
  },
  {
    slug: "friday",
    en: "Friday",
    ru: "пятница",
    hi: "शुक्रवार",
    zh: "星期五",
    ruGenitive: "пятницы",
    ruInstrumental: "пятницей",
  },
  {
    slug: "saturday",
    en: "Saturday",
    ru: "суббота",
    hi: "शनिवार",
    zh: "星期六",
    ruGenitive: "субботы",
    ruInstrumental: "субботой",
  },
  {
    slug: "sunday",
    en: "Sunday",
    ru: "воскресенье",
    hi: "रविवार",
    zh: "星期日",
    ruGenitive: "воскресенья",
    ruInstrumental: "воскресеньем",
  },
];

function hasCalendarCjkCharacter(term) {
  return /[\u4e00-\u9fff]/u.test(term);
}

function isCalendarWordCharacter(character) {
  return /[\p{L}\p{N}_]/u.test(character);
}

function containsCalendarTerm(text, term) {
  if (hasCalendarCjkCharacter(term)) {
    return String(text || "").includes(term);
  }
  let index = String(text || "").indexOf(term);
  while (index !== -1) {
    const before = index > 0 ? Array.from(text.slice(0, index)).pop() : "";
    const after = Array.from(text.slice(index + term.length))[0] || "";
    if (
      (!before || !isCalendarWordCharacter(before)) &&
      (!after || !isCalendarWordCharacter(after))
    ) {
      return true;
    }
    index = text.indexOf(term, index + term.length);
  }
  return false;
}

// Issue #386: calendar recognition is driven entirely by the self-describing
// `calendar_*` meanings (see data/seed/meanings-calendar.lino, embedded in
// MEANINGS_LINO). day-reference / today / weekday words match with the
// boundary-aware containsCalendarTerm; direction and question words match as
// loose substrings (parity with raw `str::contains` in calendar.rs). The
// boundary vs. substring split per role mirrors the Rust handler exactly.
function mentionsWeekdayContext(normalized) {
  return wordsForRole(ROLE_CALENDAR_DAY_REFERENCE).some((word) =>
    containsCalendarTerm(normalized, word),
  );
}

function mentionsCurrentDayQuestion(normalized) {
  const mentionsToday = wordsForRole(ROLE_CALENDAR_TODAY).some((word) =>
    containsCalendarTerm(normalized, word),
  );
  if (!mentionsToday) return false;
  const asksForDay = wordsForRole(ROLE_CALENDAR_DAY_REFERENCE).some((word) =>
    containsCalendarTerm(normalized, word),
  );
  const questionLike = wordsForRole(ROLE_CALENDAR_QUESTION).some((word) =>
    normalized.includes(word),
  );
  return asksForDay && questionLike;
}

function detectWeekdayOperation(normalized) {
  const hasNext = wordsForRole(ROLE_CALENDAR_DIRECTION_NEXT).some((marker) =>
    normalized.includes(marker),
  );
  const hasPrevious = wordsForRole(ROLE_CALENDAR_DIRECTION_PREVIOUS).some(
    (marker) => normalized.includes(marker),
  );
  if (hasNext && !hasPrevious) return "next";
  if (hasPrevious && !hasNext) return "previous";
  return null;
}

function detectWeekday(normalized) {
  for (const meaning of meaningsWithRole(ROLE_CALENDAR_WEEKDAY)) {
    if (meaning.words.some((word) => containsCalendarTerm(normalized, word))) {
      const entry = WEEKDAY_CYCLE.find((weekday) => weekday.slug === meaning.slug);
      if (entry) return entry;
    }
  }
  return null;
}

function shiftWeekday(weekday, operation) {
  const index = WEEKDAY_CYCLE.indexOf(weekday);
  const offset = operation === "next" ? 1 : -1;
  return WEEKDAY_CYCLE[(index + offset + WEEKDAY_CYCLE.length) % WEEKDAY_CYCLE.length];
}

function validCalendarTimeZone(candidate) {
  const timeZone = cleanContextValue(candidate);
  if (!timeZone) return "";
  try {
    new Intl.DateTimeFormat("en-US", { timeZone }).format(new Date(0));
    return timeZone;
  } catch (_error) {
    return "";
  }
}

function resolvedCalendarTimeZone(userContext) {
  const fromContext = validCalendarTimeZone(userContext && userContext.timeZone);
  if (fromContext) return fromContext;
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || "";
  } catch (_error) {
    return "";
  }
}

function calendarDateInTimeZone(date, timeZone) {
  const options = {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  };
  if (timeZone) options.timeZone = timeZone;
  const parts = new Intl.DateTimeFormat("en-CA", options).formatToParts(date);
  const value = (type) => parts.find((part) => part.type === type)?.value || "";
  const year = Number(value("year"));
  const month = Number(value("month"));
  const day = Number(value("day"));
  if (!Number.isFinite(year) || !Number.isFinite(month) || !Number.isFinite(day)) {
    return null;
  }
  const iso = `${String(year).padStart(4, "0")}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
  const dayIndex = new Date(Date.UTC(year, month - 1, day)).getUTCDay();
  const weekday = WEEKDAY_CYCLE[(dayIndex + 6) % 7];
  return { iso, weekday };
}

function currentCalendarDate(userContext) {
  const reference = new Date();
  const timeZone = resolvedCalendarTimeZone(userContext);
  return {
    timeZone: timeZone || "local",
    date: calendarDateInTimeZone(reference, timeZone),
  };
}

function renderCurrentDay(language, weekday, isoDate, timeZone) {
  if (language === "ru") {
    return `Сегодня ${weekday.ru}, ${isoDate} (${timeZone}).`;
  }
  if (language === "hi") {
    return `आज ${weekday.hi} है, ${isoDate} (${timeZone}).`;
  }
  if (language === "zh") {
    return `今天是${weekday.zh}，${isoDate}（${timeZone}）。`;
  }
  return `Today is ${weekday.en}, ${isoDate} (${timeZone}).`;
}

function renderWeekdayRelation(language, operation, source, result) {
  const delta = operation === "next" ? "+1" : "-1";
  if (language === "ru") {
    if (operation === "next") {
      return `После ${source.ruGenitive} наступает ${result.ru}. Я сдвинул ${source.ru} на ${delta} в семидневном календарном цикле.`;
    }
    return `Перед ${source.ruInstrumental} идёт ${result.ru}. Я сдвинул ${source.ru} на ${delta} в семидневном календарном цикле.`;
  }
  if (language === "hi") {
    if (operation === "next") {
      return `${source.hi} के बाद ${result.hi} आता है। मैं सात दिनों के कैलेंडर चक्र में ${source.hi} को ${delta} दिन सरकाता हूँ।`;
    }
    return `${source.hi} से पहले ${result.hi} आता है। मैं सात दिनों के कैलेंडर चक्र में ${source.hi} को ${delta} दिन सरकाता हूँ।`;
  }
  if (language === "zh") {
    if (operation === "next") {
      return `${source.zh}之后是${result.zh}。我在七天的日历循环中将${source.zh}移动${delta}天。`;
    }
    return `${source.zh}之前是${result.zh}。我在七天的日历循环中将${source.zh}移动${delta}天。`;
  }
  if (operation === "next") {
    return `The day after ${source.en} is ${result.en}. I move ${source.en} by ${delta} in the seven-day calendar cycle.`;
  }
  return `The day before ${source.en} is ${result.en}. I move ${source.en} by ${delta} in the seven-day calendar cycle.`;
}

// --- Issue #404: tryCalendarCreateEvent (full parallel of Rust try_calendar_create_event).
// Must be defined before tryCalendarReasoning. All recognition uses wordsForRole(ROLE_*)
// + containsCalendarTerm (or the documented loose includes for directions/questions/fallbacks)
// exactly like mentionsWeekdayContext etc. Base date for rollover uses a UTC mirror of
// Rust current_utc_date (create does not inherit the browser userContext tz used only by
// the "today" path). Returns the same {intent, content, confidence, evidence} shape.
function mentionsCalendarCreateRequest(normalized) {
  const hasDayRef = wordsForRole(ROLE_CALENDAR_DAY_REFERENCE).some((w) =>
    containsCalendarTerm(normalized, w),
  );
  // A clock time anchors the event just as well as a day word; the lexicon
  // carries localized surfaces ("17:00", "5pm", "शाम 5 बजे", "下午5点") and the
  // scanner covers any bare "HH:MM"/"в HH". Mirrors Rust has_clock.
  const hasClock =
    wordsForRole(ROLE_CALENDAR_TIME).some((w) =>
      containsCalendarTerm(normalized, w),
    ) || extractClockTime(normalized) !== null;
  // A relative-date word ("завтра", "tomorrow", "послезавтра", "后天", …)
  // anchors the event to a specific day just as well as a day number or a
  // clock time (issue #435: "поставь мне созвон ... на завтра" carries no
  // digit at all). Surfaces live in data/seed/meanings-calendar.lino.
  const hasRelativeDate = wordsForRole(ROLE_CALENDAR_RELATIVE_DATE).some((w) =>
    containsCalendarTerm(normalized, w),
  );
  // A genuine scheduling request anchors to a concrete date/time cue: a
  // day-reference word, a clock time, or a relative-date word. Bare digits
  // (e.g. the "1." / "2." of a numbered installation-guide list) are NOT a
  // date signal — they were the source of false positives that hijacked
  // installation-conversion prompts such as
  // "…the-book-of-secret-knowledge…" (issue #404 vs #423).
  const hasDateSignal = hasDayRef || hasClock || hasRelativeDate;
  if (!hasDateSignal) return false;
  const hasAction =
    wordsForRole(ROLE_CALENDAR_SCHEDULE_ACTION).some((w) =>
      containsCalendarTerm(normalized, w),
    ) ||
    wordsForRole(ROLE_CALENDAR_EVENT).some((w) =>
      containsCalendarTerm(normalized, w),
    );
  if (hasAction) return true;
  // Rust fallback heuristic (classic RU/EN patterns). Word-boundary matching
  // keeps "the-book-of-secret-knowledge" from masquerading as a schedule verb.
  const hasScheduleVerb = [
    "забей",
    "поставь",
    "создай",
    "добавь",
    "schedule",
    "book",
    "add to",
  ].some((verb) => containsCalendarTerm(normalized, verb));
  // The date/time anchor is already guaranteed by hasDateSignal above, so a
  // recognized schedule verb is enough to confirm a create request here.
  return hasScheduleVerb;
}

function extractDayNumber(normalized) {
  for (const word of wordsForRole(ROLE_CALENDAR_DAY_REFERENCE)) {
    if (!containsCalendarTerm(normalized, word)) continue;
    const pos = normalized.indexOf(word);
    if (pos !== -1) {
      const prefix = normalized.slice(0, pos);
      let digits = "";
      for (let i = prefix.length - 1; i >= 0; i--) {
        const ch = prefix[i];
        if (/\d/.test(ch)) digits = ch + digits;
        else if (digits) break;
      }
      const n = parseInt(digits, 10);
      if (n >= 1 && n <= 31) return n;
    }
  }
  // bare leading number
  let num = "";
  for (const ch of normalized) {
    if (/\d/.test(ch)) num += ch;
    else if (num) break;
  }
  const n = parseInt(num, 10);
  if (n >= 1 && n <= 31) return n;
  return null;
}

function computeTargetDateWithRollover(base, day) {
  let y = base.year,
    m = base.month,
    d = day;
  if (d < base.day) {
    m += 1;
    if (m > 12) {
      m = 1;
      y += 1;
    }
  }
  const maxDay = m === 2 ? 28 : [4, 6, 9, 11].includes(m) ? 30 : 31;
  if (d > maxDay) d = maxDay;
  return [y, m, d];
}

// Resolve a relative-date word to a whole-day offset from today. The surfaces
// and their languages live in data/seed/meanings-calendar.lino; this code knows
// only the role and the stable English meaning slugs that name the offset
// (calendar_tomorrow → +1 day, calendar_day_after_tomorrow → +2 days). Mirrors
// the Rust relative_date_offset (issue #435).
function relativeDateOffset(normalized) {
  for (const meaning of meaningsWithRole(ROLE_CALENDAR_RELATIVE_DATE)) {
    const matches = meaning.words.some((word) =>
      containsCalendarTerm(normalized, word),
    );
    if (!matches) continue;
    if (meaning.slug === "calendar_tomorrow") return 1;
    if (meaning.slug === "calendar_day_after_tomorrow") return 2;
    return null;
  }
  return null;
}

// Apply a whole-day offset to a {year, month, day} base via UTC date math.
// Mirrors date_from_unix_days(base.days_since_unix_epoch + offset) in Rust.
function offsetCalendarDate(base, offset) {
  const d = new Date(Date.UTC(base.year, base.month - 1, base.day + offset));
  return [d.getUTCFullYear(), d.getUTCMonth() + 1, d.getUTCDate()];
}

// Fallback title built from the matched event noun ("созвон" → "Созвон",
// "meeting" → "Meeting") when no explicit "на <subject>" / "for <subject>"
// phrase was given. Mirrors the Rust extract_event_title (issue #435).
function extractEventTitle(normalized) {
  for (const word of wordsForRole(ROLE_CALENDAR_EVENT)) {
    if (containsCalendarTerm(normalized, word)) return capitalizeFirst(word);
  }
  return null;
}

function extractClockTime(normalized) {
  const bytes = normalized.split("");
  for (let i = 0; i < bytes.length - 2; i++) {
    if (/\d/.test(bytes[i]) && /\d/.test(bytes[i + 1])) {
      let h = parseInt(bytes[i] + bytes[i + 1], 10);
      let j = i + 2;
      if (j < bytes.length && (bytes[j] === ":" || bytes[j] === ".")) j++;
      if (j + 1 < bytes.length && /\d/.test(bytes[j]) && /\d/.test(bytes[j + 1])) {
        const min = parseInt(bytes[j] + bytes[j + 1], 10);
        if (h <= 23 && min <= 59) {
          if (h === 0) h = 0;
          if (h === 24) h = 0;
          return [h, min];
        }
      }
    }
  }
  const vPos = normalized.indexOf("в ");
  if (vPos !== -1) {
    const tail = normalized.slice(vPos + 2);
    let num = "";
    for (const ch of tail) {
      if (/\d/.test(ch)) num += ch;
      else if (num) break;
    }
    const h = parseInt(num, 10);
    if (h <= 23) return [h, 0];
  }
  return null;
}

function resolveTimezone(normalized) {
  const hit = wordsForRole(ROLE_CALENDAR_TIMEZONE_ALIAS).some((w) =>
    containsCalendarTerm(normalized, w),
  );
  if (hit) return "Asia/Tbilisi";
  if (
    normalized.includes("asia/tbilisi") ||
    normalized.includes("tbilisi") ||
    normalized.includes("по грузии")
  )
    return "Asia/Tbilisi";
  return null;
}

function defaultTitle(language) {
  if (language === "ru") return "Событие";
  if (language === "hi") return "घटना";
  if (language === "zh") return "事件";
  return "Event";
}

function capitalizeFirst(value) {
  if (!value) return value;
  return value.charAt(0).toUpperCase() + value.slice(1);
}

// Trim a candidate title down to its subject: stop at the first time/date/zone
// boundary, sentence punctuation, or day number, then capitalize. Mirrors the
// Rust tidy_title.
function tidyTitle(candidate) {
  let end = candidate.length;
  for (const boundary of [
    " on the ",
    " on ",
    " at ",
    " в ",
    " по ",
    " на ",
    " 在 ",
    "下午",
    "上午",
    " को ",
    " शाम",
  ]) {
    const pos = candidate.indexOf(boundary);
    if (pos !== -1) end = Math.min(end, pos);
  }
  const punct = candidate.search(/[.!?,]/);
  if (punct !== -1) end = Math.min(end, punct);
  const digit = candidate.search(/\d/);
  if (digit !== -1) end = Math.min(end, digit);
  const trimmed = stripActionWords(candidate.slice(0, end).trim()).trim();
  if (!trimmed) return null;
  // A bare relative-date word ("завтра", "tomorrow", …) is a date cue, never a
  // title — reject it so the caller falls back to the event noun (issue #435).
  if (
    wordsForRole(ROLE_CALENDAR_RELATIVE_DATE).some(
      (word) => word.toLowerCase() === trimmed.toLowerCase(),
    )
  )
    return null;
  return capitalizeFirst(trimmed);
}

// Remove schedule-action verb fragments that can trail (Hindi is verb-final)
// or lead (Chinese has no word spaces) the subject so the .ics SUMMARY keeps
// only the event and its participant. Mirrors the Rust strip_action_words.
function stripActionWords(value) {
  let out = value;
  for (const fragment of [
    "शेड्यूल करें",
    "कैलेंडर में जोड़ें",
    "बनाएँ",
    "बनाओ",
    "安排",
    "添加到日历",
    "创建",
  ]) {
    out = out.split(fragment).join("");
  }
  return out.split(/\s+/).filter(Boolean).join(" ");
}

function extractTitle(normalized) {
  for (const marker of [
    "на ",
    "for ",
    "встречу ",
    "meeting with ",
    "call with ",
    "के साथ ",
    "和",
  ]) {
    const pos = normalized.indexOf(marker);
    if (pos !== -1) {
      const rest = normalized.slice(pos + marker.length).trim();
      const title = tidyTitle(rest);
      if (title) return title;
    }
  }
  for (const verb of ["забей", "поставь", "создай", "добавь"]) {
    const pos = normalized.indexOf(verb);
    if (pos !== -1) {
      const rest = normalized.slice(pos + verb.length).trimStart();
      const title = tidyTitle(rest);
      if (title && title.length < 60) return title;
    }
  }
  return null;
}

// --- Real, portable calendar artifacts (parity with Rust ScheduledEvent). ---

function pad2(value) {
  return String(value).padStart(2, "0");
}

function pad4(value) {
  return String(value).padStart(4, "0");
}

function isoDate(year, month, day) {
  return `${pad4(year)}-${pad2(month)}-${pad2(day)}`;
}

function startStamp(year, month, day, hour, minute) {
  return `${pad4(year)}${pad2(month)}${pad2(day)}T${pad2(hour)}${pad2(minute)}00`;
}

function daysInMonth(year, month) {
  if (month === 2) {
    return year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0) ? 29 : 28;
  }
  return [4, 6, 9, 11].includes(month) ? 30 : 31;
}

function addMinutes(year, month, day, hour, minute, minutes) {
  const total = hour * 60 + minute + minutes;
  let dayCarry = Math.floor(total / (24 * 60));
  const newMinute = total % 60;
  const newHour = Math.floor(total / 60) % 24;
  let y = year,
    m = month,
    d = day;
  while (dayCarry > 0) {
    if (d < daysInMonth(y, m)) {
      d += 1;
    } else {
      d = 1;
      m += 1;
      if (m > 12) {
        m = 1;
        y += 1;
      }
    }
    dayCarry -= 1;
  }
  return [y, m, d, newHour, newMinute];
}

function icsEscape(value) {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/;/g, "\\;")
    .replace(/,/g, "\\,")
    .replace(/\n/g, "\\n");
}

function icsDtstamp() {
  const d = new Date();
  return (
    `${pad4(d.getUTCFullYear())}${pad2(d.getUTCMonth() + 1)}${pad2(d.getUTCDate())}` +
    `T${pad2(d.getUTCHours())}${pad2(d.getUTCMinutes())}${pad2(d.getUTCSeconds())}Z`
  );
}

function buildIcs(event) {
  const start = startStamp(event.year, event.month, event.day, event.hour, event.minute);
  const [ey, em, ed, eh, emin] = addMinutes(
    event.year,
    event.month,
    event.day,
    event.hour,
    event.minute,
    event.durationMinutes,
  );
  const end = startStamp(ey, em, ed, eh, emin);
  const uid = `${start}-${event.timeZone}@formal-ai`;
  const lines = [
    "BEGIN:VCALENDAR",
    "VERSION:2.0",
    "PRODID:-//formal-ai//calendar//EN",
    "CALSCALE:GREGORIAN",
    "METHOD:PUBLISH",
    "BEGIN:VEVENT",
    `UID:${uid}`,
    `DTSTAMP:${icsDtstamp()}`,
    `DTSTART;TZID=${event.timeZone}:${start}`,
    `DTEND;TZID=${event.timeZone}:${end}`,
    `SUMMARY:${icsEscape(event.title)}`,
    "END:VEVENT",
    "END:VCALENDAR",
  ];
  return lines.join("\r\n") + "\r\n";
}

function buildGoogleCalendarUrl(event) {
  const start = startStamp(event.year, event.month, event.day, event.hour, event.minute);
  const [ey, em, ed, eh, emin] = addMinutes(
    event.year,
    event.month,
    event.day,
    event.hour,
    event.minute,
    event.durationMinutes,
  );
  const end = startStamp(ey, em, ed, eh, emin);
  return (
    "https://calendar.google.com/calendar/render?action=TEMPLATE" +
    `&text=${encodeURIComponent(event.title)}` +
    `&dates=${start}/${end}` +
    `&ctz=${encodeURIComponent(event.timeZone)}`
  );
}

function renderCreateConfirmation(language, event, ics, googleUrl) {
  const iso = isoDate(event.year, event.month, event.day);
  const time = `${pad2(event.hour)}:${pad2(event.minute)}`;
  const tz = event.timeZone;
  const title = event.title;
  const minutes = event.durationMinutes;
  if (language === "ru") {
    return (
      `Создать событие «${title}» на ${event.day} число (${iso}). Время: ${time}, часовой пояс: ${tz}. Длительность ${minutes} минут.\n` +
      `Импортируйте этот файл .ics в любой календарь:\n${ics}\n` +
      `Или откройте в Google Календаре (вход не требуется):\n${googleUrl}\n` +
      `Ответьте «да», чтобы подтвердить.`
    );
  }
  if (language === "hi") {
    return (
      `${iso} (${time}, समय क्षेत्र ${tz}) पर «${title}» कार्यक्रम बनाएँ। अवधि ${minutes} मिनट।\n` +
      `इस .ics फ़ाइल को किसी भी कैलेंडर में आयात करें:\n${ics}\n` +
      `या Google Calendar में खोलें (लॉगिन आवश्यक नहीं):\n${googleUrl}\n` +
      `पुष्टि के लिए «हाँ» उत्तर दें।`
    );
  }
  if (language === "zh") {
    return (
      `在 ${iso}（${time}，时区 ${tz}）创建事件「${title}」。时长 ${minutes} 分钟。\n` +
      `将此 .ics 文件导入任何日历：\n${ics}\n` +
      `或在 Google 日历中打开（无需登录）：\n${googleUrl}\n` +
      `回复「是」以确认。`
    );
  }
  return (
    `Create event «${title}» on ${iso}. Time: ${time}, timezone: ${tz}. Duration ${minutes} minutes.\n` +
    `Import this .ics file into any calendar:\n${ics}\n` +
    `Or open it in Google Calendar (no login required):\n${googleUrl}\n` +
    `Reply 'yes' to confirm.`
  );
}

function currentUtcCalendarBase() {
  const d = new Date();
  return { year: d.getUTCFullYear(), month: d.getUTCMonth() + 1, day: d.getUTCDate() };
}

function tryCalendarCreateEvent(prompt, normalized, userContext = {}) {
  if (!mentionsCalendarCreateRequest(normalized)) return null;
  const base = currentUtcCalendarBase();
  const language = detectLanguage(prompt);
  // A relative-date word ("завтра", "tomorrow", "послезавтра", …) anchors the
  // event to a day offset from today (issue #435). It takes priority over a
  // bare day number so "поставь созвон на завтра" lands on tomorrow rather
  // than today's date.
  const relativeOffset = relativeDateOffset(normalized);
  let year, month, d;
  if (relativeOffset !== null) {
    [year, month, d] = offsetCalendarDate(base, relativeOffset);
  } else {
    const day = extractDayNumber(normalized) || base.day;
    [year, month, d] = computeTargetDateWithRollover(base, day);
  }
  const [hour, minute] = extractClockTime(normalized) || [17, 0];
  const tz = resolveTimezone(normalized) || "UTC";
  // Prefer an explicit "на <subject>" / "for <subject>" title; otherwise fall
  // back to the matched event noun ("созвон" → "Созвон") before the localized
  // default, so a title-less request still proposes a meaningful event.
  const title =
    extractTitle(normalized) ||
    extractEventTitle(normalized) ||
    defaultTitle(language);
  const event = {
    title,
    year,
    month,
    day: d,
    hour,
    minute,
    timeZone: tz,
    durationMinutes: 60,
  };
  const ics = buildIcs(event);
  const googleUrl = buildGoogleCalendarUrl(event);
  const body = renderCreateConfirmation(language, event, ics, googleUrl);
  const evidence = [
    "calendar:clock:browser",
    `calendar:parsed_date:${isoDate(year, month, d)}`,
    `calendar:parsed_time:${pad2(hour)}:${pad2(minute)}`,
    `calendar:parsed_time_zone:${tz}`,
    `calendar:parsed_title:${title}`,
    `calendar:parsed_duration_minutes:${event.durationMinutes}`,
  ];
  if (relativeOffset !== null) {
    evidence.push(`calendar:parsed_relative_offset:${relativeOffset}`);
  }
  if (normalized.includes("число") || normalized.includes("number")) {
    evidence.push("calendar:parsed_via:day_number");
  }
  evidence.push(`calendar:ics:${ics}`);
  evidence.push(`calendar:google_calendar_url:${googleUrl}`);
  evidence.push(`language:${language}`);
  return {
    intent: "calendar_create_event",
    content: body,
    confidence: 0.95,
    evidence,
  };
}

function tryCalendarReasoning(prompt, normalized, userContext = {}) {
  // Calendar create/schedule (issue #404) must be attempted before the weekday-relation
  // gate (and before the current-day question) so that "18 число ... забей / поставь"
  // claims are handled by the action path and do not fall through to the existing
  // weekday-only logic. Mirrors src/solver_handlers/calendar.rs exactly.
  const create = tryCalendarCreateEvent(prompt, normalized, userContext);
  if (create) return create;
  if (mentionsCurrentDayQuestion(normalized)) {
    const language = detectLanguage(prompt);
    const resolved = currentCalendarDate(userContext);
    if (!resolved.date) return null;
    return {
      intent: "calendar_current_day",
      content: renderCurrentDay(
        language,
        resolved.date.weekday,
        resolved.date.iso,
        resolved.timeZone,
      ),
      confidence: 1.0,
      evidence: [
        "calendar:clock:browser",
        `calendar:today:${resolved.date.iso}`,
        `calendar:weekday:${resolved.date.weekday.slug}`,
        `calendar:time_zone:${resolved.timeZone}`,
        `language:${language}`,
      ],
    };
  }
  if (!mentionsWeekdayContext(normalized)) return null;
  const operation = detectWeekdayOperation(normalized);
  if (!operation) return null;
  const source = detectWeekday(normalized);
  if (!source) return null;
  const result = shiftWeekday(source, operation);
  const language = detectLanguage(prompt);
  return {
    intent: "calendar_weekday_relation",
    content: renderWeekdayRelation(language, operation, source, result),
    confidence: 1.0,
    evidence: [
      "calendar:cycle:monday,tuesday,wednesday,thursday,friday,saturday,sunday",
      `calendar:subject_weekday:${source.slug}`,
      `calendar:operation:${operation}:${source.slug}`,
      `calendar:result_weekday:${result.slug}`,
      `language:${language}`,
    ],
  };
}

function renderConceptInContext(language, context, record) {
  const contextNormalized = normalizeConceptTerm(context);
  const contextRecord = resolveContextRecord(contextNormalized);
  const contextLabel =
    (contextRecord && contextLabelFor(contextRecord, language)) || context;
  const sameAsLabel =
    String(contextLabel).trim().toLowerCase() ===
    String(context).trim().toLowerCase();
  const intentVariant = sameAsLabel
    ? "concept_lookup_in_context_no_alias"
    : "concept_lookup_in_context";
  const variantTable = MULTILINGUAL_ANSWERS[intentVariant] || {};
  const baseTable = MULTILINGUAL_ANSWERS.concept_lookup_in_context || {};
  const templateEntry =
    variantTable[language] ||
    variantTable.en ||
    baseTable[language] ||
    baseTable.en ||
    null;
  const template = templateEntry
    ? (typeof templateEntry === "string" ? templateEntry : templateEntry.text)
    : "In the context of {context} ({context_label}), {term} ({category}) means: {summary}\n\nSource: {source} ({source_kind}).";
  const localized = localizedConceptFor(record, language);
  const term = (localized && localized.term) || record.term;
  const summary = (localized && localized.summary) || record.summary;
  const source = (localized && localized.source) || record.source;
  const sourceKind =
    (localized && localized.sourceKind) || record.sourceKind;
  const sourceMarkup = renderSourceLink(source);
  return template
    .replace(/\{context_label\}/g, contextLabel)
    .replace(/\{context\}/g, context)
    .replace(/\{term\}/g, term)
    .replace(/\{category\}/g, record.category)
    .replace(/\{summary\}/g, summary)
    .replace(/\{source\}/g, sourceMarkup)
    .replace(/\{source_kind\}/g, sourceKind);
}

function renderConceptPlain(language, record) {
  const localized = localizedConceptFor(record, language);
  const term = (localized && localized.term) || record.term;
  const summary = (localized && localized.summary) || record.summary;
  const source = (localized && localized.source) || record.source;
  const sourceKind =
    (localized && localized.sourceKind) || record.sourceKind;
  const sourceMarkup = renderSourceLink(source);
  return `${term} (${record.category}): ${summary}\n\nSource: ${sourceMarkup} (${sourceKind}).`;
}

function tryConceptLookup(prompt) {
  const query = extractConceptQuery(prompt);
  if (!query) return null;
  const evidence = [`concept_lookup:request:${query.term}`];
  if (query.context) {
    evidence.push(`concept_lookup:context:${query.context}`);
  }
  if (query.responseLanguage) {
    evidence.push(`concept_lookup:response-language:${query.responseLanguage}`);
    evidence.push(`language_to:${query.responseLanguage}`);
  }
  const lookup = lookupConceptQuery(query);
  if (!lookup) {
    // Surface the miss in evidence so the demo's trace panel can show why
    // the handler declined the prompt. Returning null lets later handlers
    // (Wikipedia lookup, fallback) still get a chance.
    return null;
  }
  const record = lookup.record;
  const language = query.responseLanguage || detectLanguage(prompt);
  const localized = localizedConceptFor(record, language);
  const effectiveSource = (localized && localized.source) || record.source;
  // Issue #21: emit the percent-decoded IRI form for the trace panel.
  const humanSource = humanizeUrl(effectiveSource);
  evidence.push(`concept_lookup:hit:${record.slug}`);
  evidence.push(`source:${humanSource}`);
  if (record.wikidata) {
    evidence.push(`wikidata:${record.wikidata}`);
  }
  if (lookup.contextMatch && lookup.context) {
    evidence.push(`concept_lookup:context-match:${lookup.context}`);
    const body = renderConceptInContext(language, lookup.context, record);
    return {
      intent: "concept_lookup_in_context",
      content: body,
      confidence: 0.9,
      evidence,
    };
  }
  if (lookup.context) {
    evidence.push(`concept_lookup:context-mismatch:${lookup.context}`);
  }
  const body = renderConceptPlain(language, record);
  return {
    intent: "concept_lookup",
    content: body,
    confidence: 0.9,
    evidence,
  };
}

function extractDefinitionMergeTerm(prompt, allowPlainConcept) {
  // The intent is two meanings together: a definition_merge_action ("merge",
  // "combine", "fuse", …) applied to a definition_artifact_request
  // ("definition", "translation", "wikipedia", …). Both are matched as raw
  // substrings of the normalized prompt, so inflected forms in every supported
  // language are caught with no per-word list in code. Mirrors
  // extract_definition_merge_term in src/solver_handlers/definition_merge.rs.
  const text = String(prompt || "");
  const normalized = normalizePrompt(text);
  const asksMerge = lexiconMentionsRoleSubstring(ROLE_DEFINITION_MERGE_ACTION, normalized);
  const asksDefinition = lexiconMentionsRoleSubstring(
    ROLE_DEFINITION_ARTIFACT_REQUEST,
    normalized,
  );
  if (!asksMerge || !asksDefinition) {
    if (allowPlainConcept) {
      const query = extractConceptQuery(text);
      if (query && !query.context) return query.term;
    }
    return null;
  }

  // The introducing phrases ("definitions of", "translation for", …) are
  // definition_merge_marker prefix word forms; the literal before each slot
  // marker is the phrase to locate. They are declared in the lexicon in the
  // original priority order, so the first prefix that appears in the prompt
  // wins and the text after it becomes the term.
  const lower = text.toLowerCase();
  for (const form of roleWordForms(ROLE_DEFINITION_MERGE_MARKER)) {
    if (form.slot !== "prefix") continue;
    const marker = form.before;
    const index = lower.indexOf(marker);
    if (index < 0) continue;
    const candidate = trimDefinitionMergeTail(text.slice(index + marker.length));
    if (candidate) return candidate.toLowerCase();
  }
  const query = extractConceptQuery(text);
  return query ? query.term : null;
}

function trimDefinitionMergeTail(value) {
  // The boundary words that end the term ("from", "using", "with", …) are
  // definition_merge_tail_boundary meanings; we reconstruct each as a
  // space-padded token and cut at the earliest one we find. Only the English
  // surface forms are consulted here: this is an English-frame heuristic, and
  // the term itself may be in any language (e.g. the Russian preposition "в" is
  // part of the term "реклама в Telegram", not a boundary). The other languages
  // remain in the seed so the meaning stays fully self-describing. The quote and
  // punctuation trim sets are typographic and stay in code. Mirrors
  // trim_definition_merge_tail in src/solver_handlers/definition_merge.rs.
  const text = String(value || "");
  const lower = text.toLowerCase();
  let end = text.length;
  for (const word of wordsForRoleInLanguages(ROLE_DEFINITION_MERGE_TAIL_BOUNDARY, ["en"])) {
    const index = lower.indexOf(` ${word} `);
    if (index >= 0) end = Math.min(end, index);
  }
  return text
    .slice(0, end)
    .trim()
    .replace(/^['"`“”«»]+|['"`“”«»]+$/g, "")
    .replace(/[?。.!,;:]+$/g, "")
    .trim();
}

function inferredSourceLanguage(source) {
  const value = String(source || "");
  if (value.includes("://ru.wikipedia.org/")) return "ru";
  if (value.includes("://hi.wikipedia.org/")) return "hi";
  if (value.includes("://zh.wikipedia.org/")) return "zh";
  return "en";
}

function normalizeDefinitionFact(value) {
  return String(value || "")
    .toLocaleLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, "");
}
