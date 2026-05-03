use super::App;
use rig::completion::message::{AssistantContent, Message, ToolResultContent, UserContent};
use textwrap::wrap;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    User,
    Agent,
    System,
}

#[derive(Debug)]
pub struct CliMessage {
    pub role: Role,
    pub message: Message,
    cached_height: Option<HeightCache>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HeightCache {
    width: u16,
    has_border: bool,
    height: u16,
}

impl CliMessage {
    pub fn new(role: Role, content: String) -> Self {
        let message = match role {
            Role::User => Message::user(content),
            Role::Agent => Message::assistant(content),
            Role::System => Message::system(content),
        };

        Self {
            role,
            message,
            cached_height: None,
        }
    }

    #[allow(dead_code)]
    pub fn from_message(message: Message) -> Self {
        let role = match &message {
            Message::User { .. } => Role::User,
            Message::Assistant { .. } => Role::Agent,
            Message::System { .. } => Role::System,
        };

        Self {
            role,
            message,
            cached_height: None,
        }
    }

    pub fn get_content(&self) -> String {
        match &self.message {
            Message::System { content } => content.clone(),
            Message::User { content } => content
                .iter()
                .filter_map(|item| match item {
                    UserContent::Text(text) => Some(text.text.clone()),
                    UserContent::ToolResult(result) => {
                        let text = result
                            .content
                            .iter()
                            .filter_map(|result_item| match result_item {
                                ToolResultContent::Text(text) => Some(text.text.as_str()),
                                ToolResultContent::Image(_) => None,
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

                        if text.is_empty() { None } else { Some(text) }
                    }
                    UserContent::Image(_)
                    | UserContent::Audio(_)
                    | UserContent::Video(_)
                    | UserContent::Document(_) => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Message::Assistant { content, .. } => content
                .iter()
                .filter_map(|item| match item {
                    AssistantContent::Text(text) => Some(text.text.clone()),
                    AssistantContent::Reasoning(reasoning) => {
                        let text = reasoning.display_text();
                        if text.is_empty() { None } else { Some(text) }
                    }
                    AssistantContent::ToolCall(_) | AssistantContent::Image(_) => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

pub trait MessageUtils {
    fn get_height(&mut self, width: u16, has_border: bool) -> u16;
}

impl MessageUtils for App {
    fn get_height(&mut self, width: u16, has_border: bool) -> u16 {
        let mut height = 0;
        for message in &mut self.messages {
            height += message.get_height(width, has_border);
        }
        height
    }
}

impl MessageUtils for CliMessage {
    fn get_height(&mut self, width: u16, has_border: bool) -> u16 {
        if let Some(cache) = self.cached_height {
            if cache.width == width && cache.has_border == has_border {
                return cache.height;
            }
        }

        let content = self.get_content();
        let lines = wrap(&content, width as usize);
        let mut height = lines.len() as u16;
        if has_border {
            height += 2;
        }

        self.cached_height = Some(HeightCache {
            width,
            has_border,
            height,
        });

        height
    }
}
