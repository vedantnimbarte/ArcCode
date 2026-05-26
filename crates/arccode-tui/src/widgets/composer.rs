use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct Composer {
    pub input: String,
    pub busy: bool,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,
}

impl Composer {
    pub fn clear(&mut self) {
        self.input.clear();
        self.history_idx = None;
    }

    pub fn take_input(&mut self) -> String {
        let s = std::mem::take(&mut self.input);
        if !s.trim().is_empty() {
            self.history.push(s.clone());
        }
        self.history_idx = None;
        s
    }

    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let next = match self.history_idx {
            None => self.history.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.history_idx = Some(next);
        self.input = self.history[next].clone();
    }

    pub fn history_next(&mut self) {
        match self.history_idx {
            None => {}
            Some(i) if i + 1 >= self.history.len() => {
                self.history_idx = None;
                self.input.clear();
            }
            Some(i) => {
                self.history_idx = Some(i + 1);
                self.input = self.history[i + 1].clone();
            }
        }
    }
}

pub struct ComposerView<'a> {
    pub composer: &'a Composer,
}

impl<'a> Widget for ComposerView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = if self.composer.busy {
            " ⏳ working "
        } else {
            " › "
        };
        let title_style = if self.composer.busy {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Cyan)
        };
        let block = Block::default()
            .borders(Borders::TOP)
            .title(Span::styled(title, title_style));
        let mut text = self.composer.input.clone();
        if !self.composer.busy {
            text.push('▏'); // cursor
        }
        Paragraph::new(Line::from(text))
            .block(block)
            .render(area, buf);
    }
}
