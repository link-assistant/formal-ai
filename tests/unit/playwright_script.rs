use formal_ai::{FormalAiEngine, SolverConfig, UniversalSolver};

const PLAYWRIGHT_EN_ANSWER: &str = r#"I will use a starter TypeScript example based on the Playwright docs.

Source: https://playwright.dev/docs/writing-tests

```typescript
import { test, expect } from '@playwright/test';

test('opens the Playwright docs', async ({ page }) => {
  await page.goto('https://playwright.dev/');
  await expect(page).toHaveTitle(/Playwright/);

  await page.getByRole('link', { name: 'Docs' }).click();
  await expect(page.getByRole('heading', { name: /Playwright/ })).toBeVisible();
});
```

Check it with:
1. `npm init playwright@latest`
2. `npx playwright test`

Provide the URL, actions, and expected result if you want a site-specific script."#;

const PLAYWRIGHT_RU_TYPO_ANSWER: &str = r#"Я трактую `Playright` как `Playwright` и даю стартовый TypeScript-пример по документации Playwright.

Source: https://playwright.dev/docs/writing-tests

```typescript
import { test, expect } from '@playwright/test';

test('opens the Playwright docs', async ({ page }) => {
  await page.goto('https://playwright.dev/');
  await expect(page).toHaveTitle(/Playwright/);

  await page.getByRole('link', { name: 'Docs' }).click();
  await expect(page.getByRole('heading', { name: /Playwright/ })).toBeVisible();
});
```

Проверка:
1. `npm init playwright@latest`
2. `npx playwright test`

Уточните URL, действия и ожидаемый результат, если нужен сценарий под конкретный сайт."#;

#[test]
fn russian_playwright_script_prompt_returns_starter_example() {
    // Regression test for issue #135: the reported Russian prompt with the
    // common "Playright" typo was routed to the unknown fallback.
    let response = FormalAiEngine.answer("Можешь написать мне Playright скрипт?");

    assert_eq!(response.answer, PLAYWRIGHT_RU_TYPO_ANSWER);
    assert_eq!(
        response.intent, "playwright_script",
        "answer was: {}",
        response.answer
    );
    assert!(response.answer.contains("Playwright"));
    assert!(response.answer.contains("```typescript"));
    assert!(response.answer.contains("@playwright/test"));
    assert!(
        response
            .answer
            .contains("https://playwright.dev/docs/writing-tests")
    );
    assert_ne!(response.intent, "unknown");
}

#[test]
fn correctly_spelled_playwright_prompt_does_not_claim_typo_correction() {
    let response = FormalAiEngine.answer("Can you write a Playwright script?");

    assert_eq!(response.answer, PLAYWRIGHT_EN_ANSWER);
    assert_eq!(
        response.intent, "playwright_script",
        "answer was: {}",
        response.answer
    );
    assert!(response.answer.contains("@playwright/test"));
    assert!(!response.answer.contains("`Playright`"));
}

#[test]
fn playwright_script_prompts_route_across_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            prompt: "Can you write a Playwright script?",
        },
        Case {
            language: "ru",
            prompt: "Можешь написать мне Playright скрипт?",
        },
        Case {
            language: "hi",
            prompt: "क्या तुम Playwright script लिख सकते हो?",
        },
        Case {
            language: "zh",
            prompt: "可以写一个 Playwright script 吗？",
        },
    ];

    for case in cases {
        let response = FormalAiEngine.answer(case.prompt);

        assert_eq!(
            response.answer,
            if case.language == "ru" {
                PLAYWRIGHT_RU_TYPO_ANSWER
            } else {
                PLAYWRIGHT_EN_ANSWER
            }
        );
        assert_eq!(
            response.intent, "playwright_script",
            "language: {}, answer was: {}",
            case.language, response.answer
        );
        assert!(response.answer.contains("@playwright/test"));
        assert_ne!(response.intent, "unknown");
    }
}

#[test]
fn low_guess_playwright_script_prompt_asks_for_scope() {
    let solver = UniversalSolver::new(SolverConfig {
        guess_probability: 0.1,
        ..SolverConfig::default()
    });
    let response = solver.solve("Можешь написать мне Playright скрипт?");

    assert_eq!(
        response.answer,
        "Я могу написать Playwright-скрипт. Уточните URL страницы, действия и ожидаемую проверку. Если нужен пример по умолчанию, я могу взять стартовый сценарий из документации Playwright."
    );
    assert_eq!(
        response.intent, "playwright_script_clarification",
        "answer was: {}",
        response.answer
    );
    assert!(response.answer.contains("URL"));
    assert!(response.answer.contains("Playwright"));
    assert!(!response.answer.contains("```typescript"));
    assert_ne!(response.intent, "unknown");
}
