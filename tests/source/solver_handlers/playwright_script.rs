use std::fmt::Write as _;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::{detect as detect_language, Language};
use crate::seed;
use crate::solver_handlers::finalize_simple;

const PLAYWRIGHT_DOCS_URL: &str = "https://playwright.dev/docs/writing-tests";

const PLAYWRIGHT_STARTER_TYPESCRIPT: &str = r"import { test, expect } from '@playwright/test';

test('opens the Playwright docs', async ({ page }) => {
  await page.goto('https://playwright.dev/');
  await expect(page).toHaveTitle(/Playwright/);

  await page.getByRole('link', { name: 'Docs' }).click();
  await expect(page.getByRole('heading', { name: /Playwright/ })).toBeVisible();
});";

pub fn try_playwright_script(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    guess_probability: f32,
) -> Option<SymbolicAnswer> {
    if !is_playwright_script_request(normalized) {
        return None;
    }

    log.append("script_framework", "playwright".to_owned());
    log.append("source", PLAYWRIGHT_DOCS_URL.to_owned());
    let corrected_spelling = mentions_playwright_misspelling(normalized);
    if corrected_spelling {
        log.append("spelling_correction", "Playright -> Playwright".to_owned());
    }
    log.append(
        "guess_probability",
        format!("{:.2}", guess_probability.clamp(0.0, 1.0)),
    );

    let language = detect_language(prompt);
    if guess_probability < 0.5 {
        let body = render_clarification(language);
        return Some(finalize_simple(
            prompt,
            log,
            "playwright_script_clarification",
            "response:playwright_script_clarification",
            &body,
            0.64,
        ));
    }

    let body = render_starter(language, corrected_spelling);
    Some(finalize_simple(
        prompt,
        log,
        "playwright_script",
        "response:playwright_script",
        &body,
        0.82,
    ))
}

/// True when the prompt both names the Playwright tool and carries a
/// script-authoring cue, each recognized as a raw substring across every
/// supported language via the `playwright_tool_name` and
/// `playwright_script_cue` roles in the seed lexicon.
fn is_playwright_script_request(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role_raw(seed::ROLE_PLAYWRIGHT_TOOL_NAME, normalized)
        && lexicon.mentions_role_raw(seed::ROLE_PLAYWRIGHT_SCRIPT_CUE, normalized)
}

/// True when the prompt contains a misspelled form of the Playwright name —
/// any `playwright_tool_name` word form whose `action` names the canonical
/// spelling. The misspelling and its correction live in the seed data, so the
/// handler reports the fix without naming either form in the code.
fn mentions_playwright_misspelling(normalized: &str) -> bool {
    seed::lexicon()
        .role_word_forms(seed::ROLE_PLAYWRIGHT_TOOL_NAME)
        .iter()
        .filter(|form| !form.action.is_empty())
        .any(|form| normalized.contains(form.text.as_str()))
}

fn render_clarification(language: Language) -> String {
    match language {
        Language::Russian => String::from(
            "Я могу написать Playwright-скрипт. Уточните URL страницы, действия и \
             ожидаемую проверку. Если нужен пример по умолчанию, я могу взять \
             стартовый сценарий из документации Playwright.",
        ),
        _ => String::from(
            "I can write a Playwright script. Please provide the page URL, the \
             actions to perform, and the expected assertion. If you want a \
             default example, I can use the starter scenario from the Playwright docs.",
        ),
    }
}

fn render_starter(language: Language, corrected_spelling: bool) -> String {
    let mut body = String::new();
    match (language, corrected_spelling) {
        (Language::Russian, true) => body.push_str(
            "Я трактую `Playright` как `Playwright` и даю стартовый TypeScript-пример \
             по документации Playwright.\n\n",
        ),
        (Language::Russian, false) => {
            body.push_str("Даю стартовый TypeScript-пример по документации Playwright.\n\n");
        }
        (_, true) => body.push_str(
            "I interpret `Playright` as `Playwright` and will use a starter \
             TypeScript example based on the Playwright docs.\n\n",
        ),
        (_, false) => body
            .push_str("I will use a starter TypeScript example based on the Playwright docs.\n\n"),
    }
    let _ = writeln!(body, "Source: {PLAYWRIGHT_DOCS_URL}\n");
    body.push_str("```typescript\n");
    body.push_str(PLAYWRIGHT_STARTER_TYPESCRIPT);
    body.push_str("\n```\n\n");
    if language == Language::Russian {
        body.push_str("Проверка:\n");
        body.push_str("1. `npm init playwright@latest`\n");
        body.push_str("2. `npx playwright test`\n");
        body.push_str(
            "\nУточните URL, действия и ожидаемый результат, если нужен сценарий под конкретный сайт.",
        );
    } else {
        body.push_str("Check it with:\n");
        body.push_str("1. `npm init playwright@latest`\n");
        body.push_str("2. `npx playwright test`\n");
        body.push_str(
            "\nProvide the URL, actions, and expected result if you want a site-specific script.",
        );
    }
    body
}
