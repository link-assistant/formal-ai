//! Compile the same freely-phrased procedure in four languages (issue #674).
//!
//! Run with `cargo run --example issue_674_procedure_compiler`.

use formal_ai::skill_procedure::{compile_procedure, ProcedureHost, ProcedureStep};

struct DemoHost;

impl ProcedureHost for DemoHost {
    fn perform(&mut self, step: &ProcedureStep, input: &str) -> Result<String, String> {
        Ok(format!("{}({input})", step.kind))
    }
}

fn main() {
    let prompts = [
        ("en", "When I paste a link, fetch its title, translate it to Russian, save both, and reply with the translation."),
        ("ru", "Когда я вставляю ссылку, получи её заголовок, переведи его на русский, сохрани оба и ответь переводом."),
        ("hi", "जब मैं लिंक भेजूँ, उसका शीर्षक लाओ, उसे रूसी में अनुवाद करो, दोनों सहेजो और अनुवाद के साथ जवाब दो।"),
        ("zh", "当我粘贴链接，获取标题，翻译成俄语，保存两者，然后用译文回复。"),
    ];
    for (language, prompt) in prompts {
        match compile_procedure(prompt) {
            Ok(procedure) => {
                println!("--- {language} id={}", procedure.id);
                print!("{}", procedure.canonical_program);
                print!("{}", procedure.restate_steps());
                let run = procedure
                    .execute("https://example.org", &mut DemoHost)
                    .unwrap();
                println!("answer: {}", run.answer());
            }
            Err(error) => println!("--- {language} ERROR {error:?}"),
        }
    }
    let gap = compile_procedure(
        "When I paste a link, fetch its title, print it on my printer, and reply with the title.",
    );
    println!("--- gap {gap:?}");
}
