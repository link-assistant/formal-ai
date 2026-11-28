// Issue #1580: a passage cue belongs to its quotation; real surrounding tasks
// remain independent. Literal expectations are checked before recording traces.
export const TASK_CUE_CASES = [
  ['passage-cue-en', 'Read this and answer: "Marisol owns pears and plums." Which fruit?', 'en', null],
  ['passage-cue-ru', 'Прочитай и ответь: «У Марины груши и сливы». Какие фрукты?', 'ru', null],
  ['passage-cue-hi', 'पढ़ें और उत्तर दें: "सीमा के पास आम और सेब हैं।" कौन से फल?', 'hi', null],
  ['passage-cue-zh', '阅读并回答：「小明有梨并且有李子。」有什么水果？', 'zh', null],
  ['passage-cue-es', 'Lee esto y responde: "Marisol tiene peras y ciruelas." ¿Qué fruta?', 'es', null],
  ['passage-cue-case-spacing', 'READ THIS AND ANSWER  :  “Pears and plums.” Which fruit?', 'en', null],
  ['passage-cue-preceding-task', 'Count a and read this and answer: "Pears and plums." Which fruit?', 'en',
    ['Count a', 'read this and answer: "Pears and plums." Which fruit?']],
  ['passage-cue-following-task', 'Read this and answer: "Pears and plums." and count a', 'en',
    ['Read this and answer: "Pears and plums."', 'count a']],
  ['passage-cue-no-quotation', 'Read this and answer briefly', 'en', ['Read this', 'answer briefly']],
  ['passage-cue-escaped-opener', 'Read this and answer: \\"Pears\\" and count a', 'en',
    ['Read this', 'answer: \\"Pears\\"', 'count a']],
  ['passage-cue-word-boundary', 'Preread this and answer: "Pears and plums."', 'en',
    ['Preread this', 'answer: "Pears and plums."']],
  ['passage-cue-inside-quotation', 'Quote "read this and answer" and count a', 'en',
    ['Quote "read this and answer"', 'count a']],
];

/** Table-host inputs for full JavaScript/native result and event parity. */
export const CUE_CASES = TASK_CUE_CASES.map(([name, text, language, parts]) => ({
  name,
  text,
  language,
  answers: Object.fromEntries((parts ?? [text]).map((part, index) => [part,
    { intent: 'answered', content: String(index + 1), confidence: 1 }])),
  cached: {},
  withheld: [],
  bounds: {},
}));
