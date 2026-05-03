pub mod message;

pub use message::{CliMessage, MessageUtils, Role};

use crate::theme;
use ratatui::{prelude::*, widgets::*};
use ratatui_textarea::TextArea;
use tui_widgets::scrollview::ScrollViewState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SelectedWidget {
    #[default]
    MessageHistory,
    TextArea,
}

pub struct App {
    pub exit: bool,
    pub text_area: TextArea<'static>,
    pub messages: Vec<CliMessage>,
    pub scroll_view_state: ScrollViewState,
    pub is_generating: bool,
    pub status: Option<String>,
    pub model: Option<String>,
    pub selected_widget: SelectedWidget,
    pub message_history_area: Option<Rect>,
    pub text_area_area: Option<Rect>,
}

impl App {
    pub fn new(model: String) -> Self {
        Self {
            exit: false,
            text_area: Self::create_text_area(),
            messages: Vec::new(),
            scroll_view_state: ScrollViewState::default(),
            is_generating: false,
            status: None,
            model: Some(model),
            selected_widget: SelectedWidget::TextArea,
            message_history_area: None,
            text_area_area: None,
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

    pub fn set_selected_widget_from_position(&mut self, x: u16, y: u16) {
        if let Some(area) = self.message_history_area {
            if area.contains(Position::new(x, y)) {
                self.selected_widget = SelectedWidget::MessageHistory;
                return;
            }
        }

        if let Some(area) = self.text_area_area {
            if area.contains(Position::new(x, y)) {
                self.selected_widget = SelectedWidget::TextArea;
            }
        }
    }

    pub fn message_history(&self) -> Vec<rig::completion::Message> {
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
