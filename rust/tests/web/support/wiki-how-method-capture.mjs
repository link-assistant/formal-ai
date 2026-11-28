import { readFileSync } from "node:fs";

const fixture = JSON.parse(readFileSync(new URL("../../fixtures/issue-1396/wiki-how-ru.json", import.meta.url), "utf8"));
const originalSteps = fixture.parse.text["*"].split("</ol>")[0] + "</ol>";

// The reported page puts a remedy section ahead of its everyday instructions.
// The exercise also checks that a nested detail does not increment the step count.
export const everydayAndQuickMethods = `
  <h3><span class="mw-editsection">Править</span><span class="mw-headline" id="remedy">Быстрые способы избавления от запора</span></h3>
  <ol class="steps_list_2">
    <li>Сначала выберите подходящее средство вместе со специалистом.</li>
    <li>Уточните у специалиста, как долго можно применять выбранное средство.</li>
  </ol>
  <h3><span class="mw-headline" id="routine">Изменение поведения при посещении уборной</span></h3>
  ${originalSteps.replace("</li>", "<ul><li>Это пояснение относится к первому шагу и должно остаться частью этого шага.</li></ul></li>")}`;

export function wikiHowMethodsCapture(title = fixture.parse.title) {
  return JSON.stringify({ parse: { title, text: { "*": everydayAndQuickMethods } } });
}

export function wikiHowArticleCapture(canonical) {
  const body = everydayAndQuickMethods.replace(
    /<li>(Не откладывайте[^.]+\.)/u,
    '<li><div class="step_num">1</div><div class="step">$1<sup class="reference"><span>Hidden source popup</span></sup></div>',
  );
  return `<link rel="canonical" href="${canonical}"><ol><li>Unrelated page navigation must not become a step.</li></ol>${body}`;
}

export function everydayMethodReply(source) {
  return `## Как какать

1. Не откладывайте поход в туалет, когда почувствуете естественный позыв. Это пояснение относится к первому шагу и должно остаться частью этого шага.
2. Сядьте удобно и поставьте ноги на невысокую устойчивую подставку.

### Источники
- wikihow → ${source} (CC BY-NC-SA 3.0: https://creativecommons.org/licenses/by-nc-sa/3.0/)`;
}
