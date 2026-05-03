use rig::client::{CompletionClient, ProviderClient};
use rig::completion::{Chat, Message};
use rig::providers::openai;
use rig::providers::openai::responses_api::ResponsesCompletionModel;
use std::sync::Arc;
use std::sync::mpsc::Sender;

pub enum AgentEvent {
    Response(String),
    Error(String),
}

pub type OpenAiAgent = rig::agent::Agent<ResponsesCompletionModel>;

pub fn create_openai_agent(model: &str) -> OpenAiAgent {
    openai::Client::from_env()
        .agent(model)
        .preamble("You are a helpful coding assistant inside a terminal UI. Keep responses concise and useful.")
        .build()
}

pub fn spawn_agent_request(
    prompt: String,
    tx: Sender<AgentEvent>,
    agent: Arc<OpenAiAgent>,
    message_history: Vec<Message>,
) {
    tokio::spawn(async move {
        let result = agent
            .chat(Message::user(prompt), message_history)
            .await
            .map_err(|error| error.to_string());

        let _ = match result {
            Ok(response) => tx.send(AgentEvent::Response(response)),
            Err(error) => tx.send(AgentEvent::Error(error)),
        };
    });
}
