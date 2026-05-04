pub mod message;

pub use message::{CliMessage, MessageUtils, Role};

use crate::theme;
use ratatui::{prelude::*, widgets::*};
use ratatui_textarea::TextArea;

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
    pub scroll_offset: u16,
    pub is_generating: bool,
    pub status: Option<String>,
    pub model: Option<String>,
    pub selected_widget: SelectedWidget,
    pub message_history_area: Option<Rect>,
    pub text_area_area: Option<Rect>,
    pub cached_lines: Vec<ratatui::text::Line<'static>>,
    pub cached_width: u16,
    pub cached_msg_count: usize,
}

impl App {
    pub fn new(model: String) -> Self {
        Self {
            exit: false,
            text_area: Self::create_text_area(),
            messages: Vec::new(),
            scroll_offset: 0,
            is_generating: false,
            status: None,
            model: Some(model),
            selected_widget: SelectedWidget::TextArea,
            message_history_area: None,
            text_area_area: None,
            cached_lines: Vec::new(),
            cached_width: 0,
            cached_msg_count: 0,
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

    pub fn get_max_scroll(&self) -> u16 {
        let max_h = self.get_height();
        let viewport = self.message_history_area.map(|r| r.height).unwrap_or(20);
        max_h.saturating_sub(viewport)
    }

    pub fn get_height(&self) -> u16 {
        self.cached_lines.len() as u16
    }

    pub fn update_cache(&mut self, width: u16) {
        if self.cached_width == width && self.cached_msg_count == self.messages.len() {
            return;
        }

        let mut text_lines = Vec::new();
        for message in &self.messages {
            let (label, accent) = match message.role {
                Role::User => ("User", theme::USER),
                Role::Agent => ("Agent", theme::AGENT),
                Role::System => ("System", theme::MUTED),
            };

            let label_line = format!("── {} {}", label, "─".repeat(20));

            text_lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(label_line, Style::default().fg(accent).bold()),
            ]));

            let content = message.get_content();
            let message_lines = textwrap::wrap(&content, width as usize);
            for line in message_lines {
                text_lines.push(ratatui::text::Line::from(line.into_owned()));
            }
            text_lines.push(ratatui::text::Line::from(""));
        }

        self.cached_lines = text_lines;
        self.cached_width = width;
        self.cached_msg_count = self.messages.len();
    }

    pub fn take_input(&mut self) -> String {
        let input = self.text_area.lines().join("\n");
        self.text_area.clear();
        input
    }
}
