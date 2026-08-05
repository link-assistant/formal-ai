//! Probe for the issue #916 language matrix: which registered language does the
//! failure report come back in, and does the adverbial-qualifier fix (#905 §3)
//! read the same way in every registered language?
//!
//! Copy to `examples/` or run with
//! `cargo run --example issue_916_language_probe` after symlinking.

use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::language;
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::seed;

const MISSING_FILE_ENVELOPE: &str = "Command: cat hello.txt\n\
     Directory: (root)\n\
     Output: (empty)\n\
     Error: cat: hello.txt: No such file or directory\n\
     Exit Code: 1\n\
     Signal: 0\n\
     Process Group PGID: 685377";

fn main() {
    println!("registered: {:?}", language::registered_languages());

    for intent in ["tool_result_failed", "tool_result_failed_exit_code"] {
        for slug in language::registered_languages()
            .into_iter()
            .map(formal_ai::Language::slug)
        {
            println!(
                "{intent}/{slug} => {:?}",
                seed::localized_response(intent, slug)
            );
        }
    }

    let prompts = [
        "Run cat hello.txt",
        "Запусти cat hello.txt",
        "cat hello.txt चलाओ",
        "hello.txt फ़ाइल को टर्मिनल में दिखाएँ",
        "प्रोग्राम चलाओ और परिणाम बताओ",
        "运行 cat hello.txt",
        "Ejecuta cat hello.txt",
        "Ejecuta el comando y dime qué pasó, por favor",
        "Show me the contents of hello.txt",
        "Покажи содержимое hello.txt",
        "hello.txt की सामग्री दिखाएँ",
        "显示 hello.txt 的内容",
    ];
    for prompt in prompts {
        println!("detect({prompt:?}) = {:?}", language::detect(prompt).slug());
        let messages = vec![
            ChatMessage::user(prompt),
            ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                "call_probe",
                "bash",
                r#"{"command":"cat hello.txt"}"#,
            )]),
            ChatMessage::tool_result("call_probe", "bash", MISSING_FILE_ENVELOPE),
        ];
        for tools in [&["bash"][..], &["bash", "read_file"][..]] {
            match plan_chat_step(&messages, tools) {
                Some(AgenticPlan::Final(answer)) => println!("  {tools:?} FINAL: {answer}"),
                other => println!("  {tools:?} OTHER: {other:?}"),
            }
        }
    }

    let write_requests = [
        "Create a file hello.txt containing exactly: Hello World",
        "Создай файл hello.txt, содержащий ровно: Hello World",
        "hello.txt फ़ाइल बनाओ जिसमें लिखा हो ठीक: Hello World",
        "hello.txt फ़ाइल बनाओ जिसमें लिखा हो: Hello World",
        "创建文件 hello.txt 内容为 恰好：Hello World",
        "创建文件 hello.txt 内容为：Hello World",
        "Crea un archivo hello.txt que contenga exactamente: Hello World",
    ];
    for task in write_requests {
        println!(
            "plan({task:?}) = {:?}",
            compose_general_change_plan(task).map(|plan| (
                plan.target,
                plan.content,
                plan.verification_command
            ))
        );
    }
}
