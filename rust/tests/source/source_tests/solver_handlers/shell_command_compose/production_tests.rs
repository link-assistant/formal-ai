use super::handle_shell_command_compose;
use crate::engine::normalize_prompt;
use crate::event_log::EventLog;

#[test]
fn shell_composer_yields_to_a_more_specific_code_intent() {
    let prompt = "Show the last 20 lines of app.log";
    let composed =
        handle_shell_command_compose(prompt, &normalize_prompt(prompt), &mut EventLog::new())
            .expect("the file request has a shell composition");
    assert!(composed.answer.contains("tail -n 20 app.log"));

    let prompt = format!("{prompt} and write a regex for each record");
    assert!(
        handle_shell_command_compose(&prompt, &normalize_prompt(&prompt), &mut EventLog::new(),)
            .is_none(),
        "the registered regex intent takes precedence"
    );
}
