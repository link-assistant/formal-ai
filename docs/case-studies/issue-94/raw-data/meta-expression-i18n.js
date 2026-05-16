const dictionaries = {
  en: {
    'nav.analyse': 'Analyse',
    'nav.compare': 'Compare',
    'nav.check': 'Check',
    'nav.uniqueness': 'Uniqueness',
    'nav.formalize': 'Formalize',
    'nav.translate': 'Translate',
    'nav.preferences': 'Preferences',
    'nav.reportIssue': 'Report Issue',
    'prefs.locale': 'Language',
    'prefs.theme': 'Theme',
    'prefs.theme.auto': 'Theme: auto',
    'prefs.theme.light': 'Theme: light',
    'prefs.theme.dark': 'Theme: dark',
    'analyse.statement': 'Statement',
    'analyse.analyze': 'Analyze',
    'analyse.examples': 'Examples',
    'analyse.shuffle': 'Shuffle',
    'analyse.showAll': 'Show all',
    'analyse.showLess': 'Show fewer',
    'analyse.belief': 'Belief',
    'analyse.beliefCurrent': 'Current statement',
    'analyse.beliefFalse': 'False',
    'analyse.beliefTrue': 'True',
    'analyse.beliefReset': 'Reset',
    'analyse.interpretations': 'Interpretations',
    'analyse.alternatives': 'Alternatives',
    'analyse.alternativesHint':
      'More precise rephrasings of the current statement.',
    'analyse.dependencies': 'Dependencies',
    'analyse.dependenciesHint':
      'The selected interpretation may only be true if these conditions hold.',
    'analyse.definitions': 'Definitions',
    'analyse.definitionsHint':
      'Wikidata-backed definitions for each phrase in the statement.',
    'analyse.confirmations': 'Confirmations',
    'analyse.refutations': 'Refutations',
    'analyse.opposite': 'Opposite',
    'analyse.oppositeHint':
      'Click to analyze the negation of the current statement.',
    'analyse.correctness': 'Correctness',
    'analyse.confidence': 'Confidence',
    'analyse.level': 'Level',
    'analyse.result': 'Result',
    'analyse.supports': 'Supports',
    'analyse.refutes': 'Refutes',
    'analyse.unknowns': 'Unknowns',
    'analyse.fixtureEvidence': 'Fixture evidence',
    'analyse.reasoningSteps': 'Reasoning Steps',
    'analyse.strategy': 'Strategy',
    'analyse.lino': 'Lino',
    'analyse.reportIssue': 'Report Issue',
    'compare.heading': 'Compare',
    'compare.hint':
      'Enter two or more claims about the same subject to see how their Correctness and Confidence stack up.',
    'compare.add': 'Add claim',
    'compare.run': 'Compare',
    'check.heading': 'Check',
    'check.hint': 'Paste text to color detected statements by correctness.',
    'check.text': 'Text',
    'check.live': 'Live Wikimedia evidence',
    'check.runButton': 'Check',
    'check.copyMarkdown': 'Copy as Markdown',
    'check.copyLino': 'Copy as Links Notation',
    'check.result': 'Result',
    'check.resultPlaceholder': 'Click Check to color detected statements here.',
    'check.markdownPayload': 'Markdown',
    'check.linoPayload': 'Links Notation',
    'uniqueness.heading': 'Uniqueness',
    'uniqueness.hint':
      'Search each detected statement across public sources and estimate whether it should be cited, quoted, or reworded.',
    'uniqueness.text': 'Text',
    'uniqueness.runButton': 'Search',
    'uniqueness.copyMarkdown': 'Copy as Markdown',
    'uniqueness.copyLino': 'Copy as Links Notation',
    'uniqueness.result': 'Result',
    'uniqueness.resultPlaceholder':
      'Click Search to inspect detected statements here.',
    'uniqueness.matches': 'Matches',
    'uniqueness.markdownPayload': 'Markdown',
    'uniqueness.linoPayload': 'Links Notation',
    'formalize.heading': 'Formalize',
    'formalize.hint':
      'Paste any text. Each phrase becomes a Wikipedia-or-Wikidata link whose title carries the Q/P id. Toggle the link target mode, then copy the result as Markdown or Links Notation.',
    'formalize.text': 'Text',
    'formalize.maxNgram': 'Max n-gram size',
    'formalize.linkTarget': 'Link target',
    'formalize.linkWikipedia': 'Wikipedia (fallback Wikidata)',
    'formalize.linkWikidata': 'Wikidata only',
    'formalize.linkLocal': 'Local viewer',
    'formalize.sources': 'Sources',
    'formalize.sourceWikidata': 'Wikidata',
    'formalize.sourceWiktionary': 'Wiktionary (definitions)',
    'formalize.sourceWordnet': 'WordNet (Wiktionary)',
    'formalize.sourceFandom': 'Fandom wiki:',
    'formalize.loadSample': 'Load repo sample',
    'formalize.pickExample': '— pick an example —',
    'formalize.displayMode': 'Interpretation display',
    'formalize.displayName': 'Name',
    'formalize.displayMeaning': 'Meaning',
    'formalize.displayNameMeaning': 'Name + meaning',
    'formalize.displayId': 'Id only',
    'formalize.displayReplace': 'Replace',
    'formalize.overrides': 'Lazy overrides (Links Notation)',
    'formalize.overridesHint':
      'Each entry pins a phrase to a specific entity, bypassing live lookups. Useful for reproducible demos and to fill gaps the knowledge graphs miss. Indented Links Notation (.lino) is preferred; legacy JSON arrays are still accepted.',
    'formalize.runButton': 'Formalize',
    'formalize.copyMarkdown': 'Copy as Markdown',
    'formalize.copyLino': 'Copy as Links Notation',
    'formalize.result': 'Result',
    'formalize.resultPlaceholder':
      'Click Formalize to see the wikified output here.',
    'formalize.contexts': 'Contexts',
    'formalize.contextsHint':
      'Click a context to reinterpret the same text through that lens.',
    'formalize.bigContexts': 'Big-context categories',
    'formalize.bigContextsHint':
      'Worlds and domains the text touches (Math, Geography, Star Wars, Genshin Impact, ...). The first item is the dominant world.',
    'formalize.topInterpretations': 'Top 10 Interpretations',
    'formalize.topInterpretationsHint':
      'Click an interpretation to use it as the active result. The current choice is always present in the list and highlighted.',
    'formalize.markdownPayload': 'Markdown',
    'formalize.linoPayload': 'Links Notation',
    'formalize.activeBadge': 'Active',
    'formalize.score': 'score',
    'formalize.empty': 'No alternative interpretations.',
    'translate.heading': 'Translate',
    'translate.hint':
      'Formalize source text with Wikidata, then translate each sentence through labels and transformation rules. Unresolved parts remain variables with questions.',
    'translate.text': 'Text',
    'translate.from': 'From',
    'translate.to': 'To',
    'translate.runButton': 'Translate',
    'translate.copyMarkdown': 'Copy as Markdown',
    'translate.copyLino': 'Copy as Links Notation',
    'translate.formalized': 'Formalized input',
    'translate.formalizedPlaceholder':
      'Click Translate to see the formalized source here.',
    'translate.result': 'Result',
    'translate.resultPlaceholder': 'Click Translate to see the output here.',
    'translate.questions': 'Questions',
    'translate.questionsEmpty': 'No unresolved variables.',
    'translate.markdownPayload': 'Markdown',
    'translate.linoPayload': 'Links Notation',
    'translate.cstPayload': 'Translation CST',
    'translate.stepsPayload': 'Translation steps',
    'preferences.heading': 'Preferences',
    'preferences.worldview': 'Worldview',
    'preferences.religions': 'Religions',
    'preferences.context': 'Context',
    'preferences.evidence': 'Evidence scoring',
    'preferences.lino': 'Links Notation',
    'preferences.export': 'Export',
    'preferences.import': 'Import',
    'preferences.reset': 'Reset',
  },
  ru: {
    'nav.analyse': 'Анализ',
    'nav.compare': 'Сравнение',
    'nav.check': 'Проверка',
    'nav.uniqueness': 'Уникальность',
    'nav.formalize': 'Формализация',
    'nav.translate': 'Перевод',
    'nav.preferences': 'Настройки',
    'nav.reportIssue': 'Сообщить о проблеме',
    'prefs.locale': 'Язык',
    'prefs.theme': 'Тема',
    'prefs.theme.auto': 'Тема: авто',
    'prefs.theme.light': 'Тема: светлая',
    'prefs.theme.dark': 'Тема: тёмная',
    'analyse.statement': 'Утверждение',
    'analyse.analyze': 'Проанализировать',
    'analyse.examples': 'Примеры',
    'analyse.shuffle': 'Перемешать',
    'analyse.showAll': 'Показать все',
    'analyse.showLess': 'Показать меньше',
    'analyse.belief': 'Уверенность',
    'analyse.beliefCurrent': 'Текущее утверждение',
    'analyse.beliefFalse': 'Ложь',
    'analyse.beliefTrue': 'Истина',
    'analyse.beliefReset': 'Сброс',
    'analyse.interpretations': 'Интерпретации',
    'analyse.alternatives': 'Альтернативы',
    'analyse.alternativesHint':
      'Более точные переформулировки текущего утверждения.',
    'analyse.dependencies': 'Зависимости',
    'analyse.dependenciesHint':
      'Выбранная интерпретация может быть истинной только при выполнении этих условий.',
    'analyse.definitions': 'Определения',
    'analyse.definitionsHint':
      'Определения каждой фразы в утверждении из Wikidata.',
    'analyse.confirmations': 'Подтверждения',
    'analyse.refutations': 'Опровержения',
    'analyse.opposite': 'Противоположность',
    'analyse.oppositeHint':
      'Нажмите, чтобы проанализировать отрицание текущего утверждения.',
    'analyse.correctness': 'Корректность',
    'analyse.confidence': 'Уверенность',
    'analyse.level': 'Уровень',
    'analyse.result': 'Результат',
    'analyse.supports': 'Подтверждают',
    'analyse.refutes': 'Опровергают',
    'analyse.unknowns': 'Неизвестно',
    'analyse.fixtureEvidence': 'Локальные доказательства',
    'analyse.reasoningSteps': 'Шаги рассуждения',
    'analyse.strategy': 'Стратегия',
    'analyse.lino': 'Lino',
    'analyse.reportIssue': 'Сообщить о проблеме',
    'compare.heading': 'Сравнение',
    'compare.hint':
      'Введите два или более утверждений об одном предмете, чтобы сравнить их корректность и уверенность.',
    'compare.add': 'Добавить',
    'compare.run': 'Сравнить',
    'check.heading': 'Проверка',
    'check.hint':
      'Вставьте текст, чтобы раскрасить найденные утверждения по корректности.',
    'check.text': 'Текст',
    'check.live': 'Живые данные Wikimedia',
    'check.runButton': 'Проверить',
    'check.copyMarkdown': 'Скопировать как Markdown',
    'check.copyLino': 'Скопировать как Links Notation',
    'check.result': 'Результат',
    'check.resultPlaceholder':
      'Нажмите «Проверить», чтобы раскрасить найденные утверждения.',
    'check.markdownPayload': 'Markdown',
    'check.linoPayload': 'Links Notation',
    'uniqueness.heading': 'Уникальность',
    'uniqueness.hint':
      'Ищет каждое найденное утверждение в публичных источниках и оценивает, нужно ли его цитировать, брать в кавычки или переформулировать.',
    'uniqueness.text': 'Текст',
    'uniqueness.runButton': 'Искать',
    'uniqueness.copyMarkdown': 'Скопировать как Markdown',
    'uniqueness.copyLino': 'Скопировать как Links Notation',
    'uniqueness.result': 'Результат',
    'uniqueness.resultPlaceholder':
      'Нажмите «Искать», чтобы проверить найденные утверждения.',
    'uniqueness.matches': 'Совпадения',
    'uniqueness.markdownPayload': 'Markdown',
    'uniqueness.linoPayload': 'Links Notation',
    'formalize.heading': 'Формализация',
    'formalize.hint':
      'Вставьте любой текст. Каждая фраза станет ссылкой на Wikipedia или Wikidata, в title которой будет идентификатор Q/P. Выберите режим ссылки и скопируйте результат как Markdown или Links Notation.',
    'formalize.text': 'Текст',
    'formalize.maxNgram': 'Макс. размер n-граммы',
    'formalize.linkTarget': 'Цель ссылки',
    'formalize.linkWikipedia': 'Wikipedia (с откатом на Wikidata)',
    'formalize.linkWikidata': 'Только Wikidata',
    'formalize.linkLocal': 'Локальный просмотрщик',
    'formalize.sources': 'Источники',
    'formalize.sourceWikidata': 'Wikidata',
    'formalize.sourceWiktionary': 'Wiktionary (определения)',
    'formalize.sourceWordnet': 'WordNet (Wiktionary)',
    'formalize.sourceFandom': 'Fandom-вики:',
    'formalize.loadSample': 'Загрузить пример из репозитория',
    'formalize.pickExample': '— выберите пример —',
    'formalize.displayMode': 'Отображение интерпретаций',
    'formalize.displayName': 'Название',
    'formalize.displayMeaning': 'Значение',
    'formalize.displayNameMeaning': 'Название + значение',
    'formalize.displayId': 'Только id',
    'formalize.displayReplace': 'Замена',
    'formalize.overrides': 'Переопределения (Links Notation)',
    'formalize.overridesHint':
      'Каждая запись закрепляет фразу за конкретной сущностью, минуя живой поиск. Полезно для воспроизводимых демо и заполнения пробелов в графах знаний. Предпочтителен формат Links Notation (.lino), но и устаревшие JSON-массивы поддерживаются.',
    'formalize.runButton': 'Формализовать',
    'formalize.copyMarkdown': 'Скопировать как Markdown',
    'formalize.copyLino': 'Скопировать как Links Notation',
    'formalize.result': 'Результат',
    'formalize.resultPlaceholder':
      'Нажмите «Формализовать», чтобы увидеть результат.',
    'formalize.contexts': 'Контексты',
    'formalize.contextsHint':
      'Нажмите контекст, чтобы переинтерпретировать тот же текст через эту призму.',
    'formalize.bigContexts': 'Крупные контексты',
    'formalize.bigContextsHint':
      'Миры и области, которых касается текст (Математика, География, Star Wars, Genshin Impact, ...). Первый элемент — доминирующий мир.',
    'formalize.topInterpretations': 'Топ-10 интерпретаций',
    'formalize.topInterpretationsHint':
      'Нажмите интерпретацию, чтобы использовать её как активный результат. Текущий выбор всегда присутствует в списке и подсвечен.',
    'formalize.markdownPayload': 'Markdown',
    'formalize.linoPayload': 'Links Notation',
    'formalize.activeBadge': 'Активна',
    'formalize.score': 'оценка',
    'formalize.empty': 'Других интерпретаций нет.',
    'translate.heading': 'Перевод',
    'translate.hint':
      'Сначала формализует исходный текст через Wikidata, затем переводит каждое предложение через метки и правила преобразования. Нераспознанные части остаются переменными с вопросами.',
    'translate.text': 'Текст',
    'translate.from': 'С языка',
    'translate.to': 'На язык',
    'translate.runButton': 'Перевести',
    'translate.copyMarkdown': 'Скопировать как Markdown',
    'translate.copyLino': 'Скопировать как Links Notation',
    'translate.formalized': 'Формализованный ввод',
    'translate.formalizedPlaceholder':
      'Нажмите «Перевести», чтобы увидеть формализацию источника.',
    'translate.result': 'Результат',
    'translate.resultPlaceholder':
      'Нажмите «Перевести», чтобы увидеть результат.',
    'translate.questions': 'Вопросы',
    'translate.questionsEmpty': 'Нераспознанных переменных нет.',
    'translate.markdownPayload': 'Markdown',
    'translate.linoPayload': 'Links Notation',
    'translate.cstPayload': 'CST перевода',
    'translate.stepsPayload': 'Шаги перевода',
    'preferences.heading': 'Настройки',
    'preferences.worldview': 'Картина мира',
    'preferences.religions': 'Религии',
    'preferences.context': 'Контекст',
    'preferences.evidence': 'Оценка источников',
    'preferences.lino': 'Links Notation',
    'preferences.export': 'Экспорт',
    'preferences.import': 'Импорт',
    'preferences.reset': 'Сброс',
  },
};

export const SUPPORTED_LOCALES = Object.freeze(Object.keys(dictionaries));
const localeStorageKey = 'meta-expression.locale.v1';

export function detectLocale(navigatorLanguage = '') {
  const candidate = String(navigatorLanguage || '').toLowerCase();
  for (const code of SUPPORTED_LOCALES) {
    if (candidate === code || candidate.startsWith(`${code}-`)) {
      return code;
    }
  }
  return 'en';
}

export function loadLocale() {
  try {
    const stored = globalThis.localStorage?.getItem(localeStorageKey);
    if (stored && SUPPORTED_LOCALES.includes(stored)) {
      return stored;
    }
  } catch {
    // ignore storage errors
  }
  const navLang =
    globalThis.navigator?.language ??
    (Array.isArray(globalThis.navigator?.languages)
      ? globalThis.navigator.languages[0]
      : '');
  return detectLocale(navLang);
}

export function persistLocale(locale) {
  try {
    globalThis.localStorage?.setItem(localeStorageKey, locale);
  } catch {
    // ignore storage errors
  }
}

export function translate(locale, key) {
  const dict = dictionaries[locale] ?? dictionaries.en;
  return dict[key] ?? dictionaries.en[key] ?? key;
}

export function applyTranslations(root, locale) {
  const scope = root ?? globalThis.document;
  if (!scope) {
    return;
  }
  scope.documentElement?.setAttribute?.('lang', locale);
  for (const node of scope.querySelectorAll('[data-i18n]')) {
    node.textContent = translate(locale, node.dataset.i18n);
  }
  for (const node of scope.querySelectorAll('[data-i18n-aria]')) {
    node.setAttribute('aria-label', translate(locale, node.dataset.i18nAria));
  }
  for (const node of scope.querySelectorAll('[data-i18n-placeholder]')) {
    node.setAttribute(
      'placeholder',
      translate(locale, node.dataset.i18nPlaceholder)
    );
  }
}

export function listLocales() {
  return SUPPORTED_LOCALES.map((code) => ({
    code,
    label: code === 'en' ? 'English' : code === 'ru' ? 'Русский' : code,
  }));
}
