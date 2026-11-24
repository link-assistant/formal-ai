use super::*;

pub(super) fn the_document_plan_renders_the_seeded_plan_in_every_prompt_language() {
    for (prompt, expected) in [
        (
            "Make me a PDF document listing countries with food subsidies for low-income people.",
            PLAN_EN_PDF,
        ),
        (
            "Сделай мне пдф файл со списком стран, где есть пособия/скидки на еду для малоимущих, как в виде прямых денежных дотаций, так и в косвенной форме, например, талоны.",
            PLAN_RU_PDF,
        ),
        (
            "गरीब लोगों के लिए खाद्य सब्सिडी वाले देशों की सूची के साथ एक PDF दस्तावेज़ बनाओ।",
            PLAN_HI_PDF,
        ),
        (
            "给我做一个包含为低收入者提供食品补贴的国家列表的PDF文档。",
            PLAN_ZH_PDF,
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(response.intent, "document_generation_plan", "{prompt}");
        assert_eq!(response.answer, expected, "{prompt}");
    }
    assert!(PLAN_RU_DOCUMENT.starts_with("Это запрос на создание документа. "));
    assert_eq!(
        formal_ai::rule_interpreter::handler_table_row("document_format", "make me a pdf"),
        Some("PDF")
    );
    assert_eq!(
        formal_ai::rule_interpreter::handler_table_row("document_noun", "сделай отчёт"),
        Some("document")
    );
    assert_eq!(
        formal_ai::rule_interpreter::handler_table_value("document_output_fence", "TOML"),
        Some("")
    );
}

pub(super) fn a_software_project_follow_up_renders_the_seeded_sentences() {
    let solver = UniversalSolver::default();
    let scraper_plan = "Design a simple web scraper in Python that:\n1. Fetches a webpage\n2. Extracts all headings (h1, h2, h3)\n3. Counts word frequency\n4. Generates a markdown summary";
    let plan = solver.solve(scraper_plan);
    let history = [
        ConversationTurn::user(scraper_plan),
        ConversationTurn::assistant(plan.answer),
    ];
    for (prompt, head, block_lines, tail) in [
        (
            "test it by scraping wikipedia.org and show me the top 10 most frequent words.",
            "Recorded a verification follow-up for the scraper from the active plan.\n\nFormalized meaning:\n",
            [
                "  action \"test\"",
                "  target_site \"wikipedia.org\"",
                "  expected_output \"the top 10 most frequent words\"",
                "  approval_gate \"network_access\"",
            ],
            "\nReasoning steps:\n1. Recognize \"test\" as a verification request that exercises the scraper from the active plan, not a fact lookup.\n2. Bind the test target to wikipedia.org and keep live fetches behind the network_access gate.\n3. Record the expected output as \"the top 10 most frequent words\" so the test harness can assert it.\n4. Drive the artifact through a deterministic fixture before any host API or network call.\n5. Keep code execution behind approval gates because the sandbox cannot run untrusted code.\n\nVerification plan:\n1. Generate the scraper core plus a deterministic test harness with a captured wikipedia.org fixture.\n2. Assert each requirement (parsing, extraction, counting, summary) against the fixture.\n3. Surface the top 10 most frequent words from the fixture run.\n4. Run the python test command once the generated_code gate is approved.\n5. Promote the run to live wikipedia.org only after the test_execution and network_access gates pass.\n\nReply `approve plan` to generate the artifact plus this test harness. Running it live against the target needs the test_execution and network_access gates.",
        ),
        (
            "покажи результат",
            "Записано продолжение типа demonstration для scraper из активного плана.\n\nФормализованный смысл:\n",
            [
                "  action \"show\"",
                "  follow_up_kind demonstration",
                "  expected_output \"результат\"",
                "  approval_gate \"generated_code\"",
            ],
            "\nШаги рассуждения:\n1. Распознать «show» как запрос типа demonstration, который проверяет scraper из активного плана, а не поиск факта.\n2. Записать ожидаемый вывод как «результат», чтобы тестовый каркас мог его проверить.\n3. Прогнать артефакт через детерминированную фикстуру до любого вызова API хоста или сети.\n4. Держать выполнение кода за шлюзами одобрения, потому что песочница не может запускать недоверенный код.\n\nПлан проверки:\n1. Сгенерировать ядро scraper и детерминированный тестовый каркас с записанной фикстурой сайта (запрошенная цель).\n2. Проверить каждое требование (разбор, извлечение, подсчёт, сводку) на фикстуре.\n3. Показать результат из запуска на фикстуре.\n4. Запустить команду тестов python, как только шлюз generated_code будет одобрен.\n5. Переводить запуск на живой сайт (запрошенная цель) только после прохождения шлюзов test_execution и network_access.\n\nОтветьте `approve plan`, чтобы сгенерировать артефакт вместе с этим тестовым каркасом. Для живого запуска против цели нужны шлюзы test_execution и network_access.",
        ),
    ] {
        let response = solver.solve_with_history(prompt, &history);
        assert_eq!(response.intent, "software_project_followup", "{prompt}");
        let (actual_head, block, actual_tail) = follow_up_parts(&response.answer);
        assert_eq!(actual_head, head, "{prompt}");
        assert_eq!(actual_tail, tail, "{prompt}");
        for line in block_lines {
            assert!(
                block.lines().any(|candidate| candidate == line),
                "{prompt}: {block}"
            );
        }
    }

    let prices_plan = "Write a Python scraper that imports product prices and stores history";
    let plan = solver.solve(prices_plan);
    let history = [
        ConversationTurn::user(prices_plan),
        ConversationTurn::assistant(plan.answer),
    ];
    let implementation = solver.solve_with_history("approve plan", &history);
    let history = [
        ConversationTurn::user(prices_plan),
        ConversationTurn::assistant(implementation.answer),
    ];
    let approved = solver.solve_with_history("run it and verify the output", &history);
    assert_eq!(approved.intent, "software_project_followup");
    let (head, block, tail) = follow_up_parts(&approved.answer);
    assert_eq!(
        head,
        "Recorded a verification follow-up for the scraper from the active plan.\n\nFormalized meaning:\n"
    );
    assert!(block.contains("  approval_state approved\n"));
    assert_eq!(
        tail,
        "\nReasoning steps:\n1. Recognize \"test\" as a verification request that exercises the scraper from the active plan, not a fact lookup.\n2. Drive the artifact through a deterministic fixture before any host API or network call.\n3. Keep code execution behind approval gates because the sandbox cannot run untrusted code.\n\nVerification plan:\n1. Generate the scraper core plus a deterministic test harness with a captured the requested target fixture.\n2. Assert each requirement (parsing, extraction, counting, summary) against the fixture.\n3. Run the python test command once the generated_code gate is approved.\n4. Promote the run to live the requested target only after the test_execution and network_access gates pass.\n\nThe plan is approved, so the generated starter already includes this test harness. Running it live needs the test_execution and network_access gates."
    );
}
