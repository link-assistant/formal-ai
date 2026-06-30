// Issue #404: verify the browser worker mirror recognizes calendar create/schedule
// requests (RU/EN/HI/ZH) and exports a real .ics VEVENT + no-login Google Calendar
// render URL, while NOT hijacking installation-conversion prompts that merely embed
// the word "book"/"books" (issue #404 vs #423 false-positive guard).
//
// Run: `node experiments/issue-404-js-calendar-create.mjs`

import fs from "node:fs";
import vm from "node:vm";
import { TextDecoder, TextEncoder } from "node:util";

const src = fs.readFileSync(
  new URL("../src/web/formal_ai_worker.js", import.meta.url),
  "utf8",
);

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async (url) => {
  const cleanUrl = String(url).split("?")[0];
  if (cleanUrl.startsWith("seed/")) {
    const body = fs.readFileSync(new URL(`../data/${cleanUrl}`, import.meta.url), "utf8");
    return { ok: true, status: 200, text: async () => body };
  }
  throw new Error(`no fetch for ${url}`);
};
sandbox.location = { search: "", origin: "http://localhost" };
sandbox.TextEncoder = TextEncoder;
sandbox.TextDecoder = TextDecoder;
sandbox.crypto = globalThis.crypto;
sandbox.URL = URL;
vm.createContext(sandbox);
sandbox.importScripts = (...urls) => {
  for (const url of urls) {
    const workerSource = fs.readFileSync(
      new URL(`../src/web/${url}`, import.meta.url),
      "utf8",
    );
    vm.runInContext(workerSource, sandbox, { filename: url });
  }
};
vm.runInContext(src, sandbox, { filename: "formal_ai_worker.js" });
await sandbox.loadSeed();

const failures = [];
function check(name, condition, detail = "") {
  console.log(
    `${condition ? "PASS" : "FAIL"}: ${name}${detail ? ` :: ${detail}` : ""}`,
  );
  if (!condition) failures.push(name);
}

const norm = (p) => sandbox.normalizePrompt(p);
const gate = (p) => sandbox.mentionsCalendarCreateRequest(norm(p));
const create = (p) => sandbox.tryCalendarCreateEvent(p, norm(p), {});

// --- Positive: the four multilingual create-event prompts from the spec tests.
const russian = "Забей мне 18 число в 17:00 по грузии на встречу с Леваном";
const english = "schedule meeting with Levan on the 18th at 5pm Georgia time";
const hindi = "18 तारीख को शाम 5 बजे लेवान के साथ मीटिंग शेड्यूल करें";
const chinese = "18号下午5点和Levan安排会议";

for (const [label, prompt] of [
  ["russian", russian],
  ["english", english],
  ["hindi", hindi],
  ["chinese", chinese],
]) {
  check(`${label}: gate recognizes create request`, gate(prompt) === true);
  const hit = create(prompt);
  check(
    `${label}: routes to calendar_create_event`,
    hit?.intent === "calendar_create_event",
    hit?.intent,
  );
  check(
    `${label}: embeds importable .ics VEVENT`,
    typeof hit?.content === "string" &&
      hit.content.includes("BEGIN:VEVENT"),
  );
  check(
    `${label}: offers no-login Google Calendar render URL`,
    hit?.content?.includes("calendar.google.com/calendar/render"),
  );
}

// Russian/English carry the "по грузии"/"Georgia" alias → IANA Asia/Tbilisi.
check(
  "russian: TZID resolves to Asia/Tbilisi",
  create(russian)?.content?.includes("TZID=Asia/Tbilisi"),
);
check(
  "english: TZID resolves to Asia/Tbilisi",
  create(english)?.content?.includes("TZID=Asia/Tbilisi"),
);
check(
  "russian: full VCALENDAR wrapper present",
  create(russian)?.content?.includes("BEGIN:VCALENDAR"),
);

// --- Issue #595: Russian spoken-hour prompts from the calendar-events parent.
const issue595 = [
  [
    "#502 elliptical follow-up",
    "А можешь на 10 часов по Грузии с Марией?",
    "С марией",
    "10:00",
    "Asia/Tbilisi",
  ],
  [
    "#503 reordered meeting",
    "Создай встречу на 10 часов с Марией",
    "С марией",
    "10:00",
    "UTC",
  ],
  [
    "#504 event noun only",
    "Встречу с Марией на 10 часов",
    "С марией",
    "10:00",
    "UTC",
  ],
  [
    "#522 Georgia spoken hour",
    "Поставь мне встречу с Леваном на 5 часов по Грузии",
    "С леваном",
    "05:00",
    "Asia/Tbilisi",
  ],
];
for (const [label, prompt, expectedTitle, expectedTime, expectedTimezone] of issue595) {
  check(`${label}: gate recognizes spoken-hour create`, gate(prompt) === true);
  const hit = create(prompt);
  check(
    `${label}: routes to calendar_create_event`,
    hit?.intent === "calendar_create_event",
    hit?.intent,
  );
  check(
    `${label}: title preserves participant "${expectedTitle}"`,
    hit?.content?.includes(`«${expectedTitle}»`) ||
      hit?.content?.includes(`SUMMARY:${expectedTitle}`),
    hit?.content?.slice(0, 120),
  );
  check(
    `${label}: spoken hour becomes ${expectedTime}`,
    hit?.content?.includes(expectedTime),
    hit?.content?.slice(0, 120),
  );
  check(
    `${label}: timezone is ${expectedTimezone}`,
    hit?.content?.includes(expectedTimezone),
    hit?.content?.slice(0, 120),
  );
}

// --- Negative: installation-conversion prompts that embed "book"/"books" must NOT
// be hijacked by the calendar create gate (the original false positive).
const installationNegatives = [
  "Convert this README.md installation guide for EbookFoundation/free-programming-books into a sh script",
  "Convert this README.md installation guide for trimstray/the-book-of-secret-knowledge into a PowerShell script",
];
for (const prompt of installationNegatives) {
  check(
    `negative: gate rejects "${prompt.slice(0, 48)}…"`,
    gate(prompt) === false,
  );
  check(
    `negative: handler returns nothing for "${prompt.slice(0, 48)}…"`,
    !create(prompt),
  );
}

// A bare digit with no day word and no clock must not trip the gate either.
check(
  "negative: bare digit without clock/day word is not a create request",
  gate("install version 3 of the package") === false,
);

// --- Issue #435: relative-date scheduling ("на завтра") with no digit/day word.
const issue435 = [
  ["ru #435 prompt", "Можешь поставить мне созвон в кальндарь на завтра?", "Созвон"],
  ["ru short", "поставь созвон на завтра", "Созвон"],
  ["en", "schedule a call for tomorrow", "Call"],
  ["zh", "明天安排一个通话", "通话"],
];
for (const [label, prompt, expectedTitle] of issue435) {
  check(`${label}: gate recognizes relative-date create`, gate(prompt) === true);
  const hit = create(prompt);
  check(
    `${label}: routes to calendar_create_event`,
    hit?.intent === "calendar_create_event",
    hit?.intent,
  );
  check(
    `${label}: title is the event noun "${expectedTitle}"`,
    hit?.content?.includes(`«${expectedTitle}»`) ||
      hit?.content?.includes(`「${expectedTitle}」`) ||
      hit?.content?.includes(`SUMMARY:${expectedTitle}`),
    hit?.content?.slice(0, 80),
  );
  check(
    `${label}: records parsed_relative_offset evidence`,
    hit?.evidence?.some((e) => e.startsWith("calendar:parsed_relative_offset")),
  );
  // Tomorrow = today (UTC) + 1 day.
  const base = new Date();
  const expected = new Date(
    Date.UTC(base.getUTCFullYear(), base.getUTCMonth(), base.getUTCDate() + 1),
  );
  const iso = `${expected.getUTCFullYear()}-${String(expected.getUTCMonth() + 1).padStart(2, "0")}-${String(expected.getUTCDate()).padStart(2, "0")}`;
  check(
    `${label}: date resolves to tomorrow (${iso})`,
    hit?.evidence?.some((e) => e === `calendar:parsed_date:${iso}`),
    hit?.evidence?.find((e) => e.startsWith("calendar:parsed_date:")),
  );
}

console.log(
  `\n${failures.length === 0 ? "ALL PASS" : `FAILURES: ${failures.join(", ")}`}`,
);
process.exit(failures.length === 0 ? 0 : 1);
