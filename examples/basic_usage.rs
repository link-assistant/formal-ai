use formal_ai::{create_chat_completion, ChatCompletionRequest, ChatMessage, MessageContent};

fn main() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage {
            role: String::from("user"),
            content: MessageContent::Text(String::from("Write me hello world program in Rust")),
        }],
        stream: false,
    };
    let completion = create_chat_completion(&request);

    println!("{}", completion.choices[0].message.content.plain_text());
}
