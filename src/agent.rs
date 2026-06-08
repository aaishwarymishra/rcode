use rig::client::{CompletionClient, ProviderClient};
use rig::completion::{Chat, Message};
use rig::providers::deepseek;
use rig::providers::deepseek::CompletionModel;
use std::sync::Arc;
use std::sync::mpsc::Sender;

pub mod file_tool;

pub enum AgentEvent {
    Response(String),
    Error(String),
}

pub type DeepSeekAgent = rig::agent::Agent<CompletionModel>;

pub fn create_deepseek_agent(model: &str) -> DeepSeekAgent {
    deepseek::Client::from_env()
        .agent(model)
        .preamble("You are a helpful coding assistant inside a terminal UI. Keep responses concise and useful.")
        .default_max_turns(10)
        .tool(file_tool::ListFiles)
        .tool(file_tool::SearchFile)
        .tool(file_tool::GetLines)
        .tool(file_tool::PatchFile)
        .tool(file_tool::ReadFile)
        .tool(file_tool::WriteFile)
        .build()
}

pub fn spawn_agent_request(
    prompt: String,
    tx: Sender<AgentEvent>,
    agent: Arc<DeepSeekAgent>,
    message_history: Vec<Message>,
) {
    tokio::spawn(async move {
        let result = agent
            .chat(Message::user(prompt), message_history)
            .await
            .map_err(|error| error.to_string());

        let _ = match result {
            Ok(response) => tx.send(AgentEvent::Response(response)),
            Err(error) => {
                println!("Error in agent request: {}", error);
                tx.send(AgentEvent::Error(error))
            }
        };
    });
}
