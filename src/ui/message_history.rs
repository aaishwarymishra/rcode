use crate::app::{App, MessageUtils, Role};
use crate::theme;
use ratatui::prelude::*;
use ratatui::widgets::*;
use tui_widgets::scrollview::{self, ScrollView};

pub fn render_message_history(
    width: u16,
    app: &mut App,
    has_border: bool,
    padding: u16,
) -> ScrollView {
    let total_height = app.get_height(width, has_border);
    let scroll_area = Size::new(width, total_height);

    let mut scroll_view =
        ScrollView::new(scroll_area).scrollbars_visibility(scrollview::ScrollbarVisibility::Never);

    let mut y = 0;

    for message in &mut app.messages {
        let height = message.get_height(width, has_border);

        let mut block = Block::default().style(Style::default().bg(theme::SURFACE).fg(theme::TEXT));

        if has_border {
            let (label, accent) = match message.role {
                Role::User => ("User", theme::USER),
                Role::Agent => ("Agent", theme::AGENT),
                Role::System => ("System", theme::MUTED),
            };

            block = block
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(accent)
                .title(label.fg(accent).bold())
                .title_alignment(Alignment::Left);
        }

        let content = message.get_content();
        let paragraph = Paragraph::new(content.as_str())
            .block(block.padding(Padding::new(padding, padding, 0, 0)));

        scroll_view.render_widget(paragraph, Rect::new(0, y, width, height));
        y += height;
    }

    scroll_view
}
