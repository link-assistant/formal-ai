import assert from "node:assert/strict";
import { test } from "node:test";

import { enhanceWithDesktopReadOnlyTool } from "../../src/web/app/desktop-read-only-tools.js";

const prompts = {
  en: [
    "search the web for formal ai", "find formal ai online", "look up formal ai",
    "google formal ai", "search online for formal ai", "do a web search for formal ai",
    "find recent formal ai pages", "look for formal ai on the internet",
    "search the internet for formal ai", "show web results for formal ai",
  ],
  ru: [
    "найди formal ai в интернете", "поищи formal ai", "выполни веб-поиск formal ai",
    "поищи formal ai в сети", "найди сайты про formal ai", "покажи результаты поиска formal ai",
    "загугли formal ai", "найди свежие страницы formal ai",
    "выполни поиск в интернете formal ai", "найди информацию о formal ai онлайн",
  ],
  hi: [
    "वेब पर formal ai खोजें", "formal ai ऑनलाइन ढूंढें", "formal ai की वेब खोज करें",
    "इंटरनेट पर formal ai खोजो", "formal ai के लिए गूगल खोज करें",
    "formal ai के वेब परिणाम दिखाएं", "formal ai की साइटें ढूंढें",
    "formal ai के नए पेज खोजें", "ऑनलाइन formal ai की जानकारी खोजें",
    "इंटरनेट से formal ai के परिणाम लाएं",
  ],
  zh: [
    "在网上搜索 formal ai", "查找 formal ai", "进行 formal ai 网页搜索",
    "用谷歌搜索 formal ai", "在互联网上查找 formal ai", "显示 formal ai 的网页结果",
    "寻找 formal ai 网站", "搜索最新的 formal ai 页面",
    "在线查找 formal ai 信息", "从网络搜索 formal ai",
  ],
};

test("recognized multilingual web-search answers use the native read-only tool", async () => {
  const calls = [];
  for (const [language, variants] of Object.entries(prompts)) {
    for (const prompt of variants) {
      const answer = {
        intent: "web_search",
        content: "browser fallback",
        toolCalls: [{ tool: "web_search", inputs: { prompt, language, query: "formal ai" } }],
      };
      const enhanced = await enhanceWithDesktopReadOnlyTool(answer, async (tool, input) => {
        calls.push({ tool, input });
        return { ok: true, executed: true, body: "native fused results", results: [] };
      });
      assert.equal(enhanced.content, "native fused results");
      assert.ok(enhanced.evidence.includes("desktop_tool:web_search"));
    }
  }
  assert.equal(calls.length, 40);
  assert.ok(calls.every((call) => call.tool === "web_search" && call.input.query === "formal ai"));
});

test("recognized fetch answers use rendered web_fetch and preserve a safe fallback", async () => {
  const answer = {
    intent: "http_fetch",
    content: "plain fetch fallback",
    toolCalls: [{ tool: "http_fetch", inputs: { url: "https://example.com/app" } }],
  };
  const enhanced = await enhanceWithDesktopReadOnlyTool(answer, async (tool, input) => ({
    ok: true,
    executed: true,
    body: `${tool}:${input.url}:rendered`,
  }));
  assert.equal(enhanced.content, "web_fetch:https://example.com/app:rendered");

  const fallback = await enhanceWithDesktopReadOnlyTool(answer, async () => ({
    ok: false,
    executed: false,
  }));
  assert.equal(fallback, answer);
});
