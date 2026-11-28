use super::*;

pub(crate) fn language_aware_answer_for(
    rule: &SelectedRule,
    language: Language,
    prompt: &str,
    prior_code_response: bool,
) -> String {
    match (rule, language) {
        (SelectedRule::Greeting, Language::Russian) => String::from(russian_greeting_answer()),
        (SelectedRule::Greeting, Language::Hindi) => String::from(hindi_greeting_answer()),
        (SelectedRule::Greeting, Language::Chinese) => String::from(chinese_greeting_answer()),
        (SelectedRule::Wellbeing, Language::Russian) => String::from(russian_wellbeing_answer()),
        (SelectedRule::Wellbeing, Language::Hindi) => String::from(hindi_wellbeing_answer()),
        (SelectedRule::Wellbeing, Language::Chinese) => String::from(chinese_wellbeing_answer()),
        (SelectedRule::Farewell, Language::Russian) => String::from(russian_farewell_answer()),
        (SelectedRule::Farewell, Language::Hindi) => String::from(hindi_farewell_answer()),
        (SelectedRule::Farewell, Language::Chinese) => String::from(chinese_farewell_answer()),
        (SelectedRule::TestStatus, Language::Russian) => String::from(russian_test_status_answer()),
        (SelectedRule::TestStatus, Language::Hindi) => String::from(hindi_test_status_answer()),
        (SelectedRule::TestStatus, Language::Chinese) => String::from(chinese_test_status_answer()),
        (SelectedRule::CourtesyResponse, Language::Russian) => {
            String::from(russian_courtesy_response_answer())
        }
        (SelectedRule::CourtesyResponse, Language::Hindi) => {
            String::from(hindi_courtesy_response_answer())
        }
        (SelectedRule::CourtesyResponse, Language::Chinese) => {
            String::from(chinese_courtesy_response_answer())
        }
        (SelectedRule::AssistantFreeTime, Language::Russian) => {
            String::from(russian_assistant_free_time_answer())
        }
        (SelectedRule::AssistantFreeTime, Language::Hindi) => {
            String::from(hindi_assistant_free_time_answer())
        }
        (SelectedRule::AssistantFreeTime, Language::Chinese) => {
            String::from(chinese_assistant_free_time_answer())
        }
        (SelectedRule::Identity, Language::Russian) => String::from(russian_identity_answer()),
        (SelectedRule::Identity, Language::Hindi) => String::from(hindi_identity_answer()),
        (SelectedRule::Identity, Language::Chinese) => String::from(chinese_identity_answer()),
        (SelectedRule::AssistantName, Language::Russian) => {
            String::from(russian_assistant_name_answer())
        }
        (SelectedRule::AssistantName, Language::Hindi) => {
            String::from(hindi_assistant_name_answer())
        }
        (SelectedRule::AssistantName, Language::Chinese) => {
            String::from(chinese_assistant_name_answer())
        }
        (SelectedRule::WriteProgram(spec), language) => {
            let answer = write_program_answer(*spec, language, prior_code_response);
            crate::code_editing::apply_inline_hello_world_output_replacement(prompt, &answer, *spec)
                .unwrap_or(answer)
        }
        (
            SelectedRule::UnsupportedWriteProgram {
                task,
                language: program_language,
            },
            language,
        ) => {
            unsupported_write_program_answer(task.as_deref(), program_language.as_deref(), language)
        }
        (SelectedRule::Unknown, _) => {
            crate::unknown_opener::language_aware_unknown_answer(prompt, language)
        }
        _ => rule.answer(),
    }
}
