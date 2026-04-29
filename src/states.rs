use crate::theme;
use ratatui::{prelude::*, widgets::*};
use ratatui_textarea::TextArea;
use rig::completion::message::{AssistantContent, Message, ToolResultContent, UserContent};
use textwrap::wrap;
use tui_scrollview::ScrollViewState;

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

pub struct App {
    pub exit: bool,
    pub text_area: TextArea<'static>,
    pub messages: Vec<CliMessage>,
    pub scroll_view_state: ScrollViewState,
    pub is_generating: bool,
    pub status: Option<String>,
    pub model: Option<String>,
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
        for m in &mut self.messages {
            height += m.get_height(width, has_border);
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

impl App {
    #[allow(dead_code)]
    pub fn new(model: String) -> Self {
        Self {
            exit: false,
            text_area: Self::create_text_area(),
            messages: Vec::new(),
            scroll_view_state: ScrollViewState::default(),
            is_generating: false,
            status: None,
            model: Some(model),
        }
    }

    pub fn create_text_area() -> TextArea<'static> {
        let mut text_area = TextArea::default();
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(theme::BORDER_LIGHT)
            .style(Style::default().bg(theme::BACKGROUND).fg(theme::TEXT));
        text_area.set_block(block);
        text_area.set_cursor_style(
            Style::default()
                .bg(theme::BORDER_LIGHT)
                .fg(theme::BACKGROUND),
        );
        text_area.set_cursor_line_style(Style::default().bg(theme::SURFACE_ALT));
        text_area.set_selection_style(Style::default().bg(theme::SELECTION));
        text_area.set_placeholder_text("Type a message...");
        text_area.set_placeholder_style(Style::default().fg(theme::MUTED));
        text_area
    }

    pub fn add_message(&mut self, role: Role, content: String) {
        self.messages.push(CliMessage::new(role, content));
    }

    pub fn message_history(&self) -> Vec<Message> {
        self.messages
            .iter()
            .map(|message| message.message.clone())
            .collect()
    }

    pub fn take_input(&mut self) -> String {
        let input = self.text_area.lines().join("\n");
        self.text_area.clear();
        input
    }
}
