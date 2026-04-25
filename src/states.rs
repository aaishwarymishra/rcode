use ratatui::{prelude::*, widgets::*};
use ratatui_textarea::TextArea;
use textwrap::wrap;
use tui_scrollview::ScrollViewState;

#[derive(Debug, PartialEq, Eq)]
pub enum Role {
    User,
    Agent,
}
pub struct CliMessage {
    pub role: Role,
    pub content: String,
}

pub struct App {
    pub exit: bool,
    pub text_area: TextArea<'static>,
    pub messages: Vec<CliMessage>,
    pub scroll_view_state: ScrollViewState,
}

pub trait MessageUtils {
    fn get_height(&self, width: u16, has_border: bool) -> u16;
}

impl MessageUtils for App {
    fn get_height(&self, width: u16, has_border: bool) -> u16 {
        let mut height = 0;
        for m in &self.messages {
            height += m.get_height(width, has_border);
        }
        height
    }
}

impl MessageUtils for CliMessage {
    fn get_height(&self, width: u16, has_border: bool) -> u16 {
        let lines = wrap(&self.content, width as usize);
        let mut height = lines.len() as u16;
        if has_border {
            height += 2;
        }
        height
    }
}

impl App {
    pub fn dummy() -> Self {
        let dummy_messages = vec![
            CliMessage {
                role: Role::User,
                content: "Hello".to_string(),
            },
            CliMessage {
                role: Role::Agent,
                content: "How are you?".to_string(),
            },
            CliMessage {
                role: Role::User,
                content: "What is your name?".to_string(),
            },
        ];
        Self {
            exit: false,
            text_area: Self::create_text_area(),
            messages: dummy_messages,
            scroll_view_state: ScrollViewState::default(),
        }
    }

    pub fn new() -> Self {
        Self {
            exit: false,
            text_area: Self::create_text_area(),
            messages: Vec::new(),
            scroll_view_state: ScrollViewState::default(),
        }
    }

    pub fn create_text_area() -> TextArea<'static> {
        let mut text_area = TextArea::default();
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .bg(Color::Black);
        text_area.set_block(block);
        text_area
    }

    pub fn add_message(&mut self, role: Role, content: String) {
        self.messages.push(CliMessage { role, content });
    }

    pub fn take_input(&mut self) -> String {
        let input = self.text_area.lines().join("\n");
        self.text_area.clear();
        input
    }
}
