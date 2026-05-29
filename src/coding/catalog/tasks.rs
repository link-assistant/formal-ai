//! The catalog of supported coding tasks. Each task is realized in every
//! language by a matching template in [`super::templates_core`] /
//! [`super::templates_extended`]; adding a task is a matter of extending
//! [`PROGRAM_TASKS`] and supplying those templates.

use super::types::ProgramTask;

pub const PROGRAM_TASKS: &[ProgramTask] = &[
    ProgramTask {
        slug: "hello_world",
        label: "hello world",
        aliases: &["hello world", "хелло ворлд"],
        output: "Hello, world!",
    },
    ProgramTask {
        slug: "count_to_three",
        label: "count to three",
        aliases: &[
            "count to three",
            "count to 3",
            "counts to three",
            "counts to 3",
        ],
        output: "1\n2\n3",
    },
    ProgramTask {
        slug: "list_files",
        label: "list files in the current directory",
        // English, Russian, Hindi and Chinese phrasings of "list the files in
        // the current directory" (issue #312). The Russian reporter wrote
        // "выдаёт список файлов в текущей директории"; competitors answered with
        // full code. Every supported prompt language (en, ru, hi, zh) is covered
        // so the whole class of list-files requests resolves, not just Russian.
        aliases: &[
            "list files in the current directory",
            "list files in current directory",
            "list files in the directory",
            "list the files in the current directory",
            "lists files in the current directory",
            "lists the files in the current directory",
            "list files in a directory",
            "list directory files",
            "list files",
            "lists files",
            "files in the current directory",
            "files in current directory",
            "список файлов в текущей директории",
            "список файлов в текущем каталоге",
            "список файлов в директории",
            "список файлов в каталоге",
            "выдаёт список файлов",
            "выдает список файлов",
            "выводит список файлов",
            "вывести список файлов",
            "вывод списка файлов",
            "список файлов",
            "файлы в текущей директории",
            "файлы в текущем каталоге",
            // Hindi: "list of files (in the current directory)".
            "फ़ाइलों की सूची",
            "फाइलों की सूची",
            "वर्तमान निर्देशिका की फ़ाइलें",
            "वर्तमान निर्देशिका की फाइलें",
            "निर्देशिका की फ़ाइलें",
            // Chinese: "list the files in the current directory".
            "列出当前目录中的文件",
            "列出当前目录中文件",
            "列出当前目录的文件",
            "列出当前目录文件",
            "列出目录中的文件",
            "列出文件",
        ],
        // Verified output for the documented sample directory containing exactly
        // `Cargo.toml`, `README.md`, and `main.rs`. Every template sorts names in
        // byte order, so the output is identical across languages.
        output: "Cargo.toml\nREADME.md\nmain.rs",
    },
    ProgramTask {
        slug: "list_files_arg",
        label: "list files in the directory given as a path argument",
        // Issue #324 follow-up: "Сделай так, чтобы программа принимала путь как
        // аргумент" (make the program accept a path as an argument). This task is
        // the path-argument variant of `list_files`; conversation context maps a
        // bare "accept a path argument" modification onto it (see
        // `program_path_argument_modifier`). Aliases let an explicit, single-turn
        // request resolve here directly too. Every supported prompt language
        // (en, ru, hi, zh) is covered.
        aliases: &[
            "list files in the directory given as a path argument",
            "list files in a directory given as an argument",
            "list files in the directory passed as an argument",
            "list files in a path argument",
            "list files with a path argument",
            "list files accepting a path argument",
            "список файлов в каталоге переданном как аргумент",
            "список файлов в директории переданной как аргумент",
            "список файлов по пути из аргумента",
            // Hindi: "list of files in the directory given as a path argument".
            "पथ तर्क के रूप में दी गई निर्देशिका की फ़ाइलों की सूची",
            // Chinese: "list the files in the directory given as a path argument".
            "列出作为路径参数给出的目录中的文件",
            "列出路径参数指定目录中的文件",
        ],
        // When no argument is supplied the templates fall back to "." so the
        // documented sample directory still produces the verified listing.
        output: "Cargo.toml\nREADME.md\nmain.rs",
    },
    // Issue #330: the catalog supports general coding tasks, not only
    // hello-world. The tasks below are classic, deterministic exercises that
    // exercise control flow (fizzbuzz), arithmetic (factorial, sum), and string
    // handling (reverse). Each has a fixed, self-describing scenario so the
    // verified output is unambiguous, and every supported prompt language
    // (en, ru, hi, zh) is covered.
    ProgramTask {
        slug: "fizzbuzz",
        label: "FizzBuzz",
        aliases: &[
            "fizzbuzz",
            "fizz buzz",
            // Russian transliterations of "FizzBuzz".
            "физзбазз",
            "физз базз",
            "физбаз",
            // Hindi transliteration of "FizzBuzz".
            "फ़िज़बज़",
            "फिज़बज़",
            // Chinese transliteration of "FizzBuzz".
            "菲茨巴兹",
        ],
        output: "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz",
    },
    ProgramTask {
        slug: "factorial",
        label: "factorial of 5",
        // Tied to the concrete value 5 (5! = 120) so the verified output is
        // unambiguous; the aliases require the number to avoid answering a
        // different factorial with the 5! program.
        aliases: &[
            "factorial of 5",
            "factorial of five",
            "5 factorial",
            "five factorial",
            // Russian: "factorial of 5" / "of five".
            "факториал 5",
            "факториал пяти",
            "факториал числа 5",
            // Hindi: "factorial of 5".
            "5 का फैक्टोरियल",
            "पाँच का फैक्टोरियल",
            // Chinese: "factorial of 5" (阶乘 = factorial).
            "5的阶乘",
            "五的阶乘",
        ],
        output: "120",
    },
    ProgramTask {
        slug: "reverse_string",
        label: "string reversal",
        // Reverses the literal string "hello" -> "olleh"; the scenario is fixed
        // so the output is verifiable, mirroring the hello-world philosophy.
        aliases: &[
            "reverse a string",
            "reverse the string hello",
            "reverse hello",
            "reverse string hello",
            "reverse the word hello",
            // Russian: "reverse the string" / "reverse hello".
            "перевернуть строку",
            "перевернуть строку hello",
            "развернуть строку",
            // Hindi: "reverse the string" / "reverse hello".
            "स्ट्रिंग को उलटें",
            "स्ट्रिंग पलटें",
            // Chinese: "reverse the string" (反转字符串 / 翻转字符串).
            "反转字符串",
            "翻转字符串",
            "反转hello",
        ],
        output: "olleh",
    },
    ProgramTask {
        slug: "sum_to_ten",
        label: "sum from 1 to 10",
        // Sums 1..=10 -> 55; the range is fixed so the output is verifiable.
        aliases: &[
            "sum of 1 to 10",
            "sum from 1 to 10",
            "sum the numbers 1 to 10",
            "sum of numbers from 1 to 10",
            "sum 1 to 10",
            "sum to ten",
            // Russian: "sum from 1 to 10" / "sum of the numbers from 1 to 10".
            "сумма от 1 до 10",
            "сумма чисел от 1 до 10",
            "сумма чисел от одного до десяти",
            // Hindi: "sum from 1 to 10".
            "1 से 10 तक का योग",
            "1 से 10 तक योग",
            // Chinese: "sum from 1 to 10" (求和 = sum).
            "1到10的和",
            "1到10求和",
            "求1到10的和",
        ],
        output: "55",
    },
];
