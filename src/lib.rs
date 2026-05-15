pub(crate) mod arithmetic;
pub(crate) mod concepts;
pub mod engine;
pub mod event_log;
pub mod language;
pub mod protocol;
pub mod server;
pub mod solver;
pub(crate) mod solver_handlers;
pub(crate) mod solver_helpers;
pub mod telegram;
pub mod telegram_runtime;

pub use engine::{knowledge_links_notation, FormalAiEngine, SymbolicAnswer, DEFAULT_MODEL};
pub use event_log::{Event, EventLog};
pub use language::{detect as detect_language, Language};
pub use protocol::{
    create_chat_completion, create_response, ChatChoice, ChatCompletion, ChatCompletionRequest,
    ChatMessage, MessageContent, MessageContentPart, ResponseObject, ResponseOutputContent,
    ResponseOutputMessage, ResponseUsage, ResponsesRequest, TokenUsage,
};
pub use server::{handle_api_request, serve, ApiHttpResponse};
pub use solver::{
    solve, solve_with_history, ConversationRole, ConversationTurn, SolverConfig, UniversalSolver,
};
pub use telegram::{
    handle_telegram_webhook, parse_get_updates_response, telegram_html_from_markdown,
    ParsedUpdatesBatch, TelegramPollingConfig, TelegramPollingError, TelegramPollingReply,
    TelegramReplyParameters, TelegramWebhookError, TelegramWebhookReply,
};
pub use telegram_runtime::{
    run_telegram_polling, run_telegram_polling_with_transport, run_telegram_webhook_server,
    CurlTelegramTransport, TelegramPollingRuntimeError, TelegramTransport,
};
