use formal_ai::{ChatCompletionRequest, ChatMessage, create_chat_completion};

fn main() {
    let request = ChatCompletionRequest {
        output_standard: None,
        lexical_density: None,
        model: None,
        messages: vec![ChatMessage::user("Write me hello world program in Rust")],
        response_format: None,
        temperature: None,
        stream: false,
        tools: Vec::new(),
        tool_choice: None,
        functions: Vec::new(),
        function_call: None,
        stream_options: None,
    };
    let completion = create_chat_completion(&request);

    println!("{}", completion.choices[0].message.content.plain_text());
}
