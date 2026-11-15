# Appends the issue #1483 partial-acceptance report intents to both seed copies and the meanings.
entries = [
 ("work-item-report-partial", {
  "en": "Partly done: {subject}. The session stopped at `{reason}` after a change that an independent check accepted; the requirements listed below are not yet verified.",
  "ru": "Частично выполнено: {subject}. Сеанс остановился на `{reason}` после изменения, которое приняла независимая проверка; перечисленные ниже требования ещё не проверены.",
  "hi": "आंशिक रूप से पूरा: {subject}। सत्र `{reason}` पर एक ऐसे बदलाव के बाद रुका जिसे एक स्वतंत्र जाँच ने स्वीकार किया; नीचे दी गई आवश्यकताएँ अभी सत्यापित नहीं हैं।",
  "zh": "部分完成：{subject}。会话在一次被独立检查接受的更改之后停在 `{reason}`；下列需求尚未验证。",
  "es": "Completado en parte: {subject}. La sesión se detuvo en `{reason}` tras un cambio que una comprobación independiente aceptó; los requisitos de abajo aún no están verificados."},
  "a repository work-item session ends after a change that an executed independent check accepted while other requirements stay unverified",
  "the report opens by naming the work item and the reason the session stopped, says an independent check accepted the change, and says the listed requirements are not yet verified",
  "Partly done: the request. The session stopped at `UsefulCandidateReadbackVerified` after a change that an independent check accepted; the requirements listed below are not yet verified."),
 ("work-item-report-partial-no-change-needed", {
  "en": "Partly done already: {subject}. The session stopped at `{reason}`: an independent check accepted the current files, so no change was made; the requirements listed below are not yet verified.",
  "ru": "Уже частично выполнено: {subject}. Сеанс остановился на `{reason}`: независимая проверка приняла текущие файлы, поэтому изменений не было; перечисленные ниже требования ещё не проверены.",
  "hi": "पहले से आंशिक रूप से पूरा: {subject}। सत्र `{reason}` पर रुका: एक स्वतंत्र जाँच ने मौजूदा फ़ाइलों को स्वीकार किया, इसलिए कोई बदलाव नहीं किया गया; नीचे दी गई आवश्यकताएँ अभी सत्यापित नहीं हैं।",
  "zh": "已部分完成：{subject}。会话停在 `{reason}`：独立检查接受了当前文件，因此未做更改；下列需求尚未验证。",
  "es": "Ya completado en parte: {subject}. La sesión se detuvo en `{reason}`: una comprobación independiente aceptó los archivos actuales, así que no se hizo ningún cambio; los requisitos de abajo aún no están verificados."},
  "a repository work-item session finds part of the requested state already present, an independent check accepts it, and other requirements stay unverified",
  "the report opens by naming the work item, says no change was made because the current files were accepted, and says the listed requirements are not yet verified",
  "Partly done already: the request. The session stopped at `VerifiedNoOp`: an independent check accepted the current files, so no change was made; the requirements listed below are not yet verified."),
]
def responses():
    out = []
    for intent, texts, *_ in entries:
        for lang in ["en", "ru", "hi", "zh", "es"]:
            out.append(f"  response response-{intent}-{lang}\n    intent {intent}\n    language {lang}\n    text \"{texts[lang]}\"\n")
    return "".join(out)
def meanings():
    out = []
    for intent, _, pre, eff, ex in entries:
        out.append(f"  {intent}\n    defined-by answer\n    role localized_response_template\n    precondition \"{pre}\"\n    effect \"{eff}\"\n    unit \"not applicable\"\n    example \"{ex}\"\n    grounding \"data/seed/multilingual-responses-agentic-workspace.lino\"\n")
    return "".join(out)
def insert_after(path, anchor, block):
    text = open(path, encoding="utf-8").read()
    if "work-item-report-partial" in text:
        print("already", path); return
    lines = text.split("\n")
    idx = next(i for i, l in enumerate(lines) if l == anchor)
    j = idx + 1
    while j < len(lines) and lines[j].startswith("    "):
        j += 1
    lines[j:j] = block.rstrip("\n").split("\n")
    open(path, "w", encoding="utf-8").write("\n".join(lines))
    print("updated", path)
for path in ["data/seed/multilingual-responses-agentic-workspace.lino", "rust/embedded/data/seed/multilingual-responses-agentic-workspace.lino"]:
    insert_after(path, "  response response-work-item-report-no-change-needed-es", responses())
for path in ["data/seed/meanings-engine-report-intents.lino", "rust/embedded/data/seed/meanings-engine-report-intents.lino"]:
    insert_after(path, "  work-item-report-no-change-needed", meanings())
