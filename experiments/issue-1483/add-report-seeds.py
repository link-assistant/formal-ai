# Appends the issue #1483 work-item report intents to both seed copies and the meanings.
import sys
entries = [
 ("work-item-report-accepted", {
  "en": "Done: {subject}. The session stopped at `{reason}` after a change that an independent check accepted.",
  "ru": "Выполнено: {subject}. Сеанс остановился на `{reason}` после изменения, которое приняла независимая проверка.",
  "hi": "पूरा हुआ: {subject}। सत्र `{reason}` पर एक ऐसे बदलाव के बाद रुका जिसे एक स्वतंत्र जाँच ने स्वीकार किया।",
  "zh": "已完成：{subject}。会话在一次被独立检查接受的更改之后停在 `{reason}`。",
  "es": "Completado: {subject}. La sesión se detuvo en `{reason}` tras un cambio que una comprobación independiente aceptó."},
  "a repository work-item session ends after a change that an executed independent check accepted",
  "the report opens by naming the work item and the reason the session stopped, and says an independent check accepted the change",
  "Done: the request. The session stopped at `UsefulChangeVerified` after a change that an independent check accepted."),
 ("work-item-report-no-change-needed", {
  "en": "Already done: {subject}. The session stopped at `{reason}`: an independent check accepted the current files, so no change was needed.",
  "ru": "Уже выполнено: {subject}. Сеанс остановился на `{reason}`: независимая проверка приняла текущие файлы, поэтому изменение не понадобилось.",
  "hi": "पहले से पूरा: {subject}। सत्र `{reason}` पर रुका: एक स्वतंत्र जाँच ने मौजूदा फ़ाइलों को स्वीकार किया, इसलिए किसी बदलाव की ज़रूरत नहीं थी।",
  "zh": "已经完成：{subject}。会话停在 `{reason}`：独立检查接受了当前文件，因此无需更改。",
  "es": "Ya completado: {subject}. La sesión se detuvo en `{reason}`: una comprobación independiente aceptó los archivos actuales, así que no hizo falta ningún cambio."},
  "a repository work-item session finds the requested state already present and an independent check accepts it",
  "the report opens by naming the work item and says no change was needed because the current files were accepted",
  "Already done: the request. The session stopped at `VerifiedNoOp`: an independent check accepted the current files, so no change was needed."),
 ("work-item-report-effects", {
  "en": "Files changed: {paths}.",
  "ru": "Изменённые файлы: {paths}.",
  "hi": "बदली गई फ़ाइलें: {paths}।",
  "zh": "已更改的文件：{paths}。",
  "es": "Archivos cambiados: {paths}."},
  "a work-item session changed one or more repository files",
  "the report lists the files the session changed",
  "Files changed: `src/ceiling.mjs`."),
 ("work-item-report-checks", {
  "en": "Checks: {checks}.",
  "ru": "Проверки: {checks}.",
  "hi": "जाँचें: {checks}।",
  "zh": "检查：{checks}。",
  "es": "Comprobaciones: {checks}."},
  "a work-item record carries one or more checks",
  "the report lists each check with its outcome",
  "Checks: `contract-derived-execution` passed."),
 ("work-item-report-check-passed", {
  "en": "`{kind}` passed", "ru": "`{kind}` пройдена", "hi": "`{kind}` सफल", "zh": "`{kind}` 通过", "es": "`{kind}` superada"},
  "a work-item check passed",
  "the report names the check and says it passed",
  "`contract-derived-execution` passed"),
 ("work-item-report-check-failed", {
  "en": "`{kind}` failed", "ru": "`{kind}` не пройдена", "hi": "`{kind}` विफल", "zh": "`{kind}` 未通过", "es": "`{kind}` fallida"},
  "a work-item check failed",
  "the report names the check and says it failed",
  "`contract-derived-execution` failed"),
 ("work-item-report-check-pending", {
  "en": "`{kind}` not run", "ru": "`{kind}` не выполнена", "hi": "`{kind}` नहीं चलाई गई", "zh": "`{kind}` 未运行", "es": "`{kind}` no ejecutada"},
  "a work-item check was required but not executed",
  "the report names the check and says it was not run",
  "`independent-task-acceptance` not run"),
 ("work-item-report-unresolved", {
  "en": "Evidence left unresolved ({count}):",
  "ru": "Неразрешённые свидетельства ({count}):",
  "hi": "अनसुलझे साक्ष्य ({count}):",
  "zh": "未解决的证据（{count}）：",
  "es": "Evidencia sin resolver ({count}):"},
  "a work-item record keeps evidence it could not resolve, such as an attachment or an unavailable read",
  "the report heads the list of unresolved evidence and the reason for each with their count",
  "Evidence left unresolved (1):"),
 ("work-item-report-requirements-verified", {
  "en": "All requirements were verified ({count}).",
  "ru": "Все требования проверены ({count}).",
  "hi": "सभी आवश्यकताएँ सत्यापित हुईं ({count})।",
  "zh": "全部需求均已验证（{count}）。",
  "es": "Todos los requisitos fueron verificados ({count})."},
  "every requirement of a work-item record was verified",
  "the report says all requirements were verified and how many there were",
  "All requirements were verified (2)."),
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
        ex = ex.replace('"', '\\"')
        out.append(f"  {intent}\n    defined-by answer\n    role localized_response_template\n    precondition \"{pre}\"\n    effect \"{eff}\"\n    unit \"not applicable\"\n    example \"{ex}\"\n    grounding \"data/seed/multilingual-responses-agentic-workspace.lino\"\n")
    return "".join(out)
def insert_after(path, anchor_prefix, block):
    text = open(path, encoding="utf-8").read()
    if "work-item-report-accepted" in text:
        print("already", path); return
    lines = text.split("\n")
    # find anchor line index, then skip its indented body
    idx = next(i for i, l in enumerate(lines) if l == anchor_prefix)
    j = idx + 1
    while j < len(lines) and lines[j].startswith("    "):
        j += 1
    lines[j:j] = block.rstrip("\n").split("\n")
    open(path, "w", encoding="utf-8").write("\n".join(lines))
    print("updated", path)
for path in ["data/seed/multilingual-responses-agentic-workspace.lino", "rust/embedded/data/seed/multilingual-responses-agentic-workspace.lino"]:
    insert_after(path, "  response response-work-item-report-trace-es", responses())
for path in ["data/seed/meanings-engine-report-intents.lino", "rust/embedded/data/seed/meanings-engine-report-intents.lino"]:
    insert_after(path, "  work-item-report-trace", meanings())
