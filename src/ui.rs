pub mod composer;
pub mod message_history;

use crate::app::App;
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;

const PADDING: u16 = 1;

pub fn render(frame: &mut ratatui::Frame, app: &mut App) {
    let block = Block::bordered()
        .title("RCODE")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(theme::BORDER)
        .title_style(Style::default().fg(theme::BORDER_LIGHT).bold().italic())
        .style(Style::default().bg(theme::BACKGROUND).fg(theme::TEXT));

    let inner = block.inner(frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Max(5),
        ])
        .split(inner);

    frame.render_widget(block, frame.area());
    frame.render_widget(
        Paragraph::new("Rust-powered CLI coding agent")
            .style(theme::MUTED)
            .block(Block::new().padding(Padding::left(1))),
        chunks[0],
    );

    app.message_history_area = Some(chunks[1]);
    app.update_cache(chunks[1].width);

    // Clamp scroll offset to max scroll, auto-scrolling to bottom if overshot
    app.scroll_offset = app.scroll_offset.min(app.get_max_scroll());
    let scroll_offset = app.scroll_offset;

    let message_history = message_history::render_message_history(
        app,
        chunks[1].width,
        chunks[1].height,
        scroll_offset,
    );
    frame.render_widget(message_history, chunks[1]);

    composer::render_status(frame, app, chunks[2], PADDING);
    composer::render_text_area(frame, app, chunks[3]);
}
