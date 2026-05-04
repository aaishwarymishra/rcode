use crate::app::App;
use ratatui::widgets::*;

pub fn render_message_history<'a>(
    app: &'a App,
    _width: u16,
    viewport_height: u16,
    scroll_offset: u16,
) -> Paragraph<'a> {
    let start = scroll_offset as usize;
    let end = scroll_offset.saturating_add(viewport_height) as usize;
    let end = end.min(app.cached_lines.len());

    let visible_lines = if start < app.cached_lines.len() {
        &app.cached_lines[start..end]
    } else if app.cached_lines.len() != 0 {
        &app.cached_lines[start.saturating_sub(viewport_height as usize).max(0)..]
    } else {
        &[]
    };

    let text = ratatui::text::Text::from_iter(visible_lines.iter().cloned());

    Paragraph::new(text)
}
