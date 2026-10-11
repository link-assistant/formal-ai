//! R1188-U20: requirement extraction lists the obligations an issue states,
//! in every seeded language, from seed vocabulary alone. Mirrors
//! `rust/tests/web/requirement-extraction.test.mjs`.

use formal_ai::agentic_coding::requirement_extraction::{extract_requirements, requirement_units};

const ISSUE: &str = "## Context
The cache was added last year. It works well.

The page must load in under a second. Add a progress bar to the upload form.

```js
must(notBeRead);
```
> We should ignore quoted text.

## Acceptance criteria
- The upload resumes after a reload
- [ ] Errors are shown in the user's language

## Notes
- The page MUST load in under a second!
- Something nice to know";

#[test]
fn obligations_directives_checklists_and_sections_are_listed_once_in_order() {
    assert_eq!(
        extract_requirements(ISSUE),
        [
            "The page must load in under a second.",
            "Add a progress bar to the upload form.",
            "The upload resumes after a reload",
            "Errors are shown in the user's language",
        ]
    );
}

#[test]
fn code_quotes_and_headings_are_never_units() {
    let texts: Vec<String> = requirement_units(ISSUE)
        .into_iter()
        .map(|unit| unit.text)
        .collect();
    assert!(!texts.iter().any(|text| text.contains("notBeRead")));
    assert!(!texts.iter().any(|text| text.contains("quoted")));
    assert!(!texts.iter().any(|text| text.contains("Acceptance")));
}

#[test]
fn a_definition_of_done_states_a_requirement_in_every_seeded_language() {
    assert_eq!(
        extract_requirements("Fixed means: every node carries a record. The loop is old."),
        ["Fixed means: every node carries a record."]
    );
    assert_eq!(
        extract_requirements("Исправлено значит: каждый узел хранит запись. Цикл старый."),
        ["Исправлено значит: каждый узел хранит запись."]
    );
    assert_eq!(
        extract_requirements("完成标准：每个节点都有记录。循环很旧。"),
        ["完成标准：每个节点都有记录。"]
    );
    assert_eq!(
        extract_requirements(
            "Arreglado significa: cada nodo guarda un registro. El bucle es viejo."
        ),
        ["Arreglado significa: cada nodo guarda un registro."]
    );
    assert_eq!(
        extract_requirements("पूरा तब माना जाएगा जब हर नोड रिकॉर्ड रखे। लूप पुराना है।"),
        ["पूरा तब माना जाएगा जब हर नोड रिकॉर्ड रखे।"]
    );
}

#[test]
fn every_seeded_language_states_requirements() {
    assert_eq!(
        extract_requirements("Страница должна загружаться быстро. Это просто заметка."),
        ["Страница должна загружаться быстро."]
    );
    assert_eq!(
        extract_requirements("页面必须在一秒内加载。这是背景。"),
        ["页面必须在一秒内加载。"]
    );
    assert_eq!(
        extract_requirements("La página debe cargar rápido. Es una nota."),
        ["La página debe cargar rápido."]
    );
    assert_eq!(
        extract_requirements("पेज को जल्दी लोड होना चाहिए। यह एक नोट है।"),
        ["पेज को जल्दी लोड होना चाहिए।"]
    );
}
