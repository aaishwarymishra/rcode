use crate::app::App;
use ratatui::widgets::*;

pub fn render_message_history<'a>(app: &'a App, scroll_offset: u16) -> Paragraph<'a> {
    let text = app.cached_text.clone();

    Paragraph::new(text).scroll((scroll_offset, 0))
}
