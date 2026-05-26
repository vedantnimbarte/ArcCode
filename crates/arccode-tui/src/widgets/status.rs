use arccode_core::Usage;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default, Clone)]
pub struct StatusLine {
    pub model: String,
    pub provider: String,
    pub mode: String,
    pub usage: Usage,
}

impl StatusLine {
    pub fn merge_usage(&mut self, u: &Usage) {
        self.usage.add(u);
    }
}

pub struct StatusView<'a> {
    pub status: &'a StatusLine,
}

impl<'a> Widget for StatusView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = self.status;
        let cache_hit_pct = (s.usage.cache_hit_ratio() * 100.0).round() as u32;
        let line = Line::from(vec![
            Span::styled(
                format!(" {} ", s.provider),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(s.model.clone(), Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled(
                format!("mode={}", s.mode),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "tok in:{} out:{} cache:{}%",
                    s.usage.input_tokens + s.usage.cache_creation_input_tokens,
                    s.usage.output_tokens,
                    cache_hit_pct
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        Paragraph::new(line)
            .style(Style::default().bg(Color::Reset))
            .render(area, buf);
    }
}
