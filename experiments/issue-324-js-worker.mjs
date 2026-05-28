import fs from "node:fs";
import vm from "node:vm";

const src = fs.readFileSync(new URL("../src/web/formal_ai_worker.js", import.meta.url), "utf8");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => { throw new Error("no importScripts in node"); };
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async () => { throw new Error("no fetch"); };
sandbox.location = { search: "", origin: "http://localhost" };
import { TextEncoder, TextDecoder } from "node:util";
sandbox.TextEncoder = TextEncoder;
sandbox.TextDecoder = TextDecoder;
sandbox.crypto = globalThis.crypto;
sandbox.URL = URL;
vm.createContext(sandbox);
vm.runInContext(src, sandbox, { filename: "formal_ai_worker.js" });

const fail = [];
function check(name, cond, extra) {
  console.log(`${cond ? "PASS" : "FAIL"}: ${name}${extra ? " :: " + extra : ""}`);
  if (!cond) fail.push(name);
}

// 1. detectLanguage
check("detect ru", sandbox.detectLanguage("Напиши мне программу на Rust") === "ru");

// 2. single-turn explicit list files with path argument (ru)
const p1 = sandbox.writeProgramParameters(
  sandbox.normalizeProgramPrompt("Напиши программу на Rust которая выводит файлы по пути из аргумента")
);
console.log("single-turn params:", JSON.stringify(p1));

// 3. follow-up recovery
const history = [
  { role: "user", content: "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории" },
  { role: "assistant", content: "Вот минимальная программа на языке Rust (..." },
];
const detected = sandbox.writeProgramParameters(
  sandbox.normalizeProgramPrompt("Сделай так, чтобы программа принимала путь как аргумент")
);
console.log("follow-up detected (pre-recovery):", JSON.stringify(detected));
if (detected) {
  const rec = sandbox.recoverWriteProgramParameters(
    detected,
    "Сделай так, чтобы программа принимала путь как аргумент",
    history
  );
  console.log("recovered:", JSON.stringify(rec));
  check("recovered task list_files_arg", rec.task === "list_files_arg", rec.task);
  check("recovered language rust", rec.language === "rust", rec.language);
}

// 4. tryWriteProgram follow-up in Russian
const res = sandbox.tryWriteProgram(
  "Сделай так, чтобы программа принимала путь как аргумент",
  history,
  "ru"
);
console.log("\n=== tryWriteProgram follow-up (ru) ===");
console.log("intent:", res && res.intent);
console.log(res && res.content);
if (res) {
  check("follow-up intent write_program", res.intent === "write_program", res.intent);
  check("follow-up has rust fence", res.content.includes("```rust"));
  check("follow-up has env::args", res.content.includes("env::args"));
  check("follow-up russian intro", res.content.includes("Вот минимальная программа на языке"));
  check("follow-up no missing", !res.content.includes("missing"));
}

// 4b. tryWriteProgram follow-up in Chinese
const zhHistory = [
  { role: "user", content: "用 Rust 编写一个列出当前目录中文件的程序" },
  { role: "assistant", content: "这是一个最小的 Rust 程序（..." },
];
const zhRes = sandbox.tryWriteProgram(
  "制作程序，使其接受路径作为参数",
  zhHistory,
  "zh"
);
console.log("\n=== tryWriteProgram follow-up (zh) ===");
console.log("intent:", zhRes && zhRes.intent);
if (zhRes) {
  check("zh follow-up intent write_program", zhRes.intent === "write_program", zhRes.intent);
  check("zh follow-up has rust fence", zhRes.content.includes("```rust"));
  check("zh follow-up has env::args", zhRes.content.includes("env::args"));
  check("zh follow-up no missing", !zhRes.content.includes("missing"));
}

// 4c. tryWriteProgram follow-up in Hindi
const hiHistory = [
  { role: "user", content: "Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो" },
  { role: "assistant", content: "यहाँ Rust में एक न्यूनतम प्रोग्राम है (..." },
];
const hiRes = sandbox.tryWriteProgram(
  "इसे ऐसा बनाओ कि प्रोग्राम पथ को तर्क के रूप में स्वीकार करे",
  hiHistory,
  "hi"
);
console.log("\n=== tryWriteProgram follow-up (hi) ===");
console.log("intent:", hiRes && hiRes.intent);
if (hiRes) {
  check("hi follow-up intent write_program", hiRes.intent === "write_program", hiRes.intent);
  check("hi follow-up has rust fence", hiRes.content.includes("```rust"));
  check("hi follow-up has env::args", hiRes.content.includes("env::args"));
  check("hi follow-up no missing", !hiRes.content.includes("missing"));
}

// 5. responseLanguageFor
check("respLang default last_message", sandbox.responseLanguageFor("ru", {}, {}) === "ru");
check("respLang preferred", sandbox.responseLanguageFor("ru", { responseLanguage: "preferred", preferredLanguage: "en" }, {}) === "en");
check("respLang ui explicit", sandbox.responseLanguageFor("ru", { responseLanguage: "ui", uiLanguage: "zh" }, {}) === "zh");
check("respLang ui auto->browser", sandbox.responseLanguageFor("en", { responseLanguage: "ui", uiLanguage: "auto" }, { browserLanguages: ["ru-RU"] }) === "ru");
check("respLang ui auto->detected", sandbox.responseLanguageFor("hi", { responseLanguage: "ui", uiLanguage: "auto" }, {}) === "hi");

console.log("\n" + (fail.length ? "FAILURES: " + fail.join(", ") : "ALL CHECKS PASSED"));
process.exit(fail.length ? 1 : 0);
