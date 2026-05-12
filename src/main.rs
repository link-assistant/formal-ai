use std::error::Error;

use clap::{Subcommand, ValueEnum};
use lino_arguments::Parser;

use formal_ai::{
    create_chat_completion, create_response, knowledge_links_notation, serve,
    ChatCompletionRequest, ChatMessage, FormalAiEngine, MessageContent, ResponsesRequest,
    DEFAULT_MODEL,
};

#[derive(Parser, Debug)]
#[command(name = "formal-ai", about = "Formal symbolic AI proof of concept")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Chat {
        #[arg(long, env = "FORMAL_AI_PROMPT")]
        prompt: String,

        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    Dataset,
    Serve {
        #[arg(long, env = "FORMAL_AI_HOST", default_value = "127.0.0.1")]
        host: String,

        #[arg(long, env = "FORMAL_AI_PORT", default_value_t = 8080)]
        port: u16,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Chat,
    Responses,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => formatter.write_str("text"),
            Self::Chat => formatter.write_str("chat"),
            Self::Responses => formatter.write_str("responses"),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let command = args.command.unwrap_or_else(|| Command::Chat {
        prompt: String::from("Hi"),
        format: OutputFormat::Text,
    });

    match command {
        Command::Chat { prompt, format } => run_chat(&prompt, format)?,
        Command::Dataset => println!("{}", knowledge_links_notation()),
        Command::Serve { host, port } => serve(&format!("{host}:{port}"))?,
    }

    Ok(())
}

fn run_chat(prompt: &str, format: OutputFormat) -> Result<(), Box<dyn Error>> {
    match format {
        OutputFormat::Text => {
            let response = FormalAiEngine.answer(prompt);
            println!("{}", response.answer);
        }
        OutputFormat::Chat => {
            let request = ChatCompletionRequest {
                model: Some(String::from(DEFAULT_MODEL)),
                messages: vec![ChatMessage {
                    role: String::from("user"),
                    content: MessageContent::Text(String::from(prompt)),
                }],
                stream: false,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&create_chat_completion(&request))?
            );
        }
        OutputFormat::Responses => {
            let request = ResponsesRequest {
                model: Some(String::from(DEFAULT_MODEL)),
                input: serde_json::Value::String(String::from(prompt)),
                instructions: None,
                stream: false,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&create_response(&request))?
            );
        }
    }

    Ok(())
}
