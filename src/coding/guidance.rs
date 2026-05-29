//! Issue #330: novice-first guidance that accompanies every generated program.
//!
//! A code answer must *teach* a novice, so each `write_program` response is
//! followed by a plain-language "How it works" explanation and step-by-step
//! "How to test it yourself" instructions. Both are localized for every
//! supported response language. These builders live alongside the coding
//! [`catalog`](crate::coding::catalog) and are kept out of `engine.rs` so that
//! module stays under the repository's per-file line-count limit.

use crate::coding::catalog::ProgramSpec;
use crate::language::Language;
use crate::solver::{ConversationRole, ConversationTurn};

/// Issue #330: did an earlier assistant turn already present a fenced code
/// block? When it did, follow-up code edits omit the verbose setup steps and
/// show a concise "test it the same way" note instead. Detected from the dialog
/// rather than hard-coded to a turn number.
pub fn history_has_prior_code(history: &[ConversationTurn]) -> bool {
    history.iter().any(|turn| {
        matches!(turn.role, ConversationRole::Assistant) && turn.content.contains("```")
    })
}

/// Issue #330: a "How it works" paragraph so a novice understands the program
/// instead of receiving an unexplained snippet. The explanation is localized
/// for every supported response language.
pub fn program_explanation_section(spec: ProgramSpec, language: Language) -> String {
    let heading = match language {
        Language::Russian => "Как это работает:",
        Language::Hindi => "यह कैसे काम करता है:",
        Language::Chinese => "工作原理：",
        _ => "How it works:",
    };
    format!(
        "{heading}\n{}",
        program_explanation(spec.task.slug, language)
    )
}

/// Plain-language description of the algorithm for each supported task. Kept
/// language-agnostic in the *programming* sense (every template implements the
/// same algorithm) and localized in the *response* sense (issue #330).
fn program_explanation(task_slug: &str, language: Language) -> &'static str {
    match (task_slug, language) {
        ("hello_world", Language::Russian) => {
            "Программа выводит текст `Hello, world!` в стандартный вывод и завершается."
        }
        ("hello_world", Language::Hindi) => {
            "प्रोग्राम मानक आउटपुट पर `Hello, world!` टेक्स्ट छापता है और फिर समाप्त हो जाता है।"
        }
        ("hello_world", Language::Chinese) => {
            "程序将文本 `Hello, world!` 打印到标准输出，然后退出。"
        }
        ("hello_world", _) => {
            "The program prints the text `Hello, world!` to standard output and then exits."
        }
        ("count_to_three", Language::Russian) => {
            "Программа выводит числа 1, 2 и 3 — каждое на отдельной строке — и завершается."
        }
        ("count_to_three", Language::Hindi) => {
            "प्रोग्राम संख्याएँ 1, 2 और 3 — हर एक अलग पंक्ति में — छापता है और फिर समाप्त हो जाता है।"
        }
        ("count_to_three", Language::Chinese) => {
            "程序打印数字 1、2 和 3 —— 每个数字单独一行 —— 然后退出。"
        }
        ("count_to_three", _) => {
            "The program prints the numbers 1, 2, and 3 — each on its own line — and then exits."
        }
        ("list_files_arg", Language::Russian) => {
            "Программа берёт путь к каталогу из первого аргумента командной строки (если \
             аргумент не задан, используется текущий каталог), читает содержимое этого \
             каталога, оставляет только обычные файлы, сортирует их имена по алфавиту и \
             печатает каждое имя на отдельной строке."
        }
        ("list_files_arg", Language::Hindi) => {
            "प्रोग्राम पहले कमांड-लाइन तर्क से निर्देशिका पथ लेता है (कोई तर्क न होने पर वर्तमान \
             निर्देशिका का उपयोग करता है), उस निर्देशिका की प्रविष्टियाँ पढ़ता है, केवल सामान्य \
             फ़ाइलें रखता है, उनके नामों को वर्णानुक्रम में क्रमबद्ध करता है, और हर नाम को अलग पंक्ति \
             में छापता है।"
        }
        ("list_files_arg", Language::Chinese) => {
            "程序从第一个命令行参数获取目录路径（未提供参数时使用当前目录），读取该目录的条目，\
             只保留普通文件，按字母顺序排序它们的名称，然后将每个名称打印在单独一行。"
        }
        ("list_files_arg", _) => {
            "The program takes the directory path from the first command-line argument \
             (falling back to the current directory when none is given), reads that \
             directory's entries, keeps only the regular files, sorts their names \
             alphabetically, and prints each name on its own line."
        }
        // `list_files` and any future directory task share the current-directory wording.
        (_, Language::Russian) => {
            "Программа читает содержимое текущего каталога, оставляет только обычные файлы, \
             собирает их имена в список, сортирует список по алфавиту и печатает каждое имя \
             на отдельной строке."
        }
        (_, Language::Hindi) => {
            "प्रोग्राम वर्तमान निर्देशिका की प्रविष्टियाँ पढ़ता है, केवल सामान्य फ़ाइलें रखता है, उनके \
             नाम एक सूची में एकत्र करता है, सूची को वर्णानुक्रम में क्रमबद्ध करता है, और हर नाम को \
             अलग पंक्ति में छापता है।"
        }
        (_, Language::Chinese) => {
            "程序读取当前目录的条目，只保留普通文件，将它们的名称收集到一个列表中，\
             按字母顺序排序，然后将每个名称打印在单独一行。"
        }
        (_, _) => {
            "The program reads the entries of the current directory, keeps only the regular \
             files, collects their names into a list, sorts the list alphabetically, and \
             prints each name on its own line."
        }
    }
}

/// Issue #330: step-by-step, novice-friendly instructions for testing the
/// program. When the dialog already walked the user through running code
/// (`prior_code_response`), the verbose setup steps are replaced by a short
/// "test it the same way" note so follow-up edits stay concise.
pub fn program_test_instructions(
    spec: ProgramSpec,
    language: Language,
    prior_code_response: bool,
) -> String {
    let execution = &spec.language.execution;
    let save_as = spec.language.save_as;
    let run_command = execution.run_command;

    if prior_code_response {
        return match language {
            Language::Russian => format!(
                "Проверьте обновлённую программу так же, как и раньше: сохраните код в файл \
                 `{save_as}` и снова выполните `{run_command}`."
            ),
            Language::Hindi => format!(
                "अपडेट किए गए प्रोग्राम को पहले की तरह ही जाँचें: कोड को `{save_as}` फ़ाइल में सहेजें \
                 और फिर से `{run_command}` चलाएँ।"
            ),
            Language::Chinese => format!(
                "像之前一样测试更新后的程序：将代码保存到文件 `{save_as}`，然后再次运行 \
                 `{run_command}`。"
            ),
            _ => format!(
                "Test the updated program the same way as before: save the code to `{save_as}` \
                 and run `{run_command}` again."
            ),
        };
    }

    let heading = match language {
        Language::Russian => "Как проверить это самостоятельно:",
        Language::Hindi => "इसे स्वयं कैसे जाँचें:",
        Language::Chinese => "如何自行测试：",
        _ => "How to test it yourself:",
    };

    let mut steps: Vec<String> = Vec::new();
    let setup_hint = spec.language.setup_hint;
    steps.push(match language {
        Language::Russian => format!("Установите инструментарий: {setup_hint}."),
        Language::Hindi => format!("टूलचेन इंस्टॉल करें: {setup_hint}।"),
        Language::Chinese => format!("安装工具链：{setup_hint}。"),
        _ => format!("Install {setup_hint}."),
    });
    steps.push(match language {
        Language::Russian => format!("Сохраните приведённый выше код в файл `{save_as}`."),
        Language::Hindi => format!("ऊपर दिए गए कोड को `{save_as}` फ़ाइल में सहेजें।"),
        Language::Chinese => format!("将上面的代码保存到文件 `{save_as}`。"),
        _ => format!("Save the code above to a file named `{save_as}`."),
    });
    if let Some(check_command) = execution.check_command {
        steps.push(match language {
            Language::Russian => format!("Проверьте, что код компилируется: `{check_command}`."),
            Language::Hindi => format!("जाँचें कि कोड संकलित होता है: `{check_command}`।"),
            Language::Chinese => format!("检查代码能否编译：`{check_command}`。"),
            _ => format!("Check that it compiles: `{check_command}`."),
        });
    }
    steps.push(match language {
        Language::Russian => format!("Запустите программу: `{run_command}`."),
        Language::Hindi => format!("प्रोग्राम चलाएँ: `{run_command}`।"),
        Language::Chinese => format!("运行程序：`{run_command}`。"),
        _ => format!("Run it: `{run_command}`."),
    });
    steps.push(match language {
        Language::Russian => "Сравните вывод с разделом ожидаемого вывода выше.".to_owned(),
        Language::Hindi => "आउटपुट की तुलना ऊपर दिए गए अपेक्षित आउटपुट से करें।".to_owned(),
        Language::Chinese => "将输出与上面的预期输出部分进行比较。".to_owned(),
        _ => "Compare the output with the expected output shown above.".to_owned(),
    });

    let numbered = steps
        .iter()
        .enumerate()
        .map(|(index, step)| format!("{}. {step}", index + 1))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{heading}\n{numbered}")
}
