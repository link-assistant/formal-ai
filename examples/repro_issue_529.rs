use formal_ai::{solve_with_history, ConversationTurn};

fn main() {
    let history = [
        ConversationTurn::user("Прошлое сообщение"),
        ConversationTurn::assistant("Я ещё не научился отвечать на это."),
    ];
    let prompts = [
        "что было написано в прошлом сообщении?",
        "what was written in the previous message?",
        "what was my previous question?",
        "repeat my last message",
    ];
    for p in prompts {
        let r = solve_with_history(p, &history);
        println!(
            "PROMPT: {p}\n  intent={}\n  answer={}\n",
            r.intent,
            r.answer.replace('\n', " | ")
        );
    }
}
