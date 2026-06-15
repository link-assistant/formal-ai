use formal_ai::{create_chat_completion, ChatCompletionRequest, ChatMessage};

fn main() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage::user("Write me hello world program in Rust")],
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
    };
    let completion = create_chat_completion(&request);

    println!("{}", completion.choices[0].message.content.plain_text());
}
