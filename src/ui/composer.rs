use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui_textarea::WrapMode;

pub fn render_status(frame: &mut ratatui::Frame, app: &App, area: Rect, padding: u16) {
    let status =
        app.status
            .as_deref()
            .unwrap_or(if app.is_generating { "Thinking..." } else { "" });

    frame.render_widget(
        Paragraph::new(status)
            .style(theme::MUTED)
            .block(Block::new().padding(Padding::horizontal(padding))),
        area,
    );
}

pub fn render_text_area(frame: &mut ratatui::Frame, app: &mut App, area: Rect) {
    app.text_area.set_wrap_mode(WrapMode::WordOrGlyph);
    app.text_area_area = Some(area);
    frame.render_widget(&app.text_area, area);
}
