use std::io;
use std::process::exit;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};

use crate::mpris::PlayerState;
use crate::{REFRESH_INTERVAL, SCROLL_TIMEOUT_DURATION};

pub(crate) struct App {
    player_name: String,
    last_title: Option<String>,
    last_artist: Option<Vec<String>>,
    lyrics_vertical_scroll: u16,
    last_scroll: Instant,
    exit: bool,
}

impl App {
    pub(crate) fn new(name: String) -> Self {
        Self {
            player_name: name,
            last_title: None,
            last_artist: None,
            lyrics_vertical_scroll: 0,
            last_scroll: {
                let instant = Instant::now();
                instant - SCROLL_TIMEOUT_DURATION
            },
            exit: false,
        }
    }

    pub(crate) fn frame(
        &mut self,
        terminal: &mut DefaultTerminal,
        frame_start: Instant,
        state: &PlayerState,
    ) -> io::Result<()> {
        if self.exit {
            ratatui::restore();
            exit(0);
        }

        terminal.draw(|frame| self.draw(frame, state))?;

        // Handle user inputs in the time the app was gonna wait anyways
        if let Some(remaining) = REFRESH_INTERVAL.checked_sub(frame_start.elapsed()) {
            self.handle_events(remaining)?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, state: &PlayerState) {
        let title = Line::from(format! {" {} ", self.player_name}.bold());
        let instructions = Line::from(vec![
            " Scroll up ".into(),
            "<Up>".blue().bold(),
            " Scroll down ".into(),
            "<Down>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let metadata = Text::from(vec![Line::from(vec![
            state
                .title
                .clone()
                .unwrap_or("Unknown".to_string())
                .yellow(),
            " - ".into(),
            state
                .artists
                .as_ref()
                .map(|artists| artists.join(", "))
                .unwrap_or("Unknown".to_string())
                .yellow(),
        ])]);

        let inner_area = block.inner(frame.area());
        let [_, lyrics_area, _] = Layout::vertical([
            Constraint::Length(2), // 1 line for metadata + 1 empty spacer row
            Constraint::Min(0),
            Constraint::Length(1), // 1 empty spacer row
        ])
        .areas(inner_area);

        let paragraph = Paragraph::new(metadata).centered().block(block);
        frame.render_widget(paragraph, frame.area());

        // Reset scroll when new song starts
        if self.last_title != state.title || self.last_artist != state.artists {
            self.lyrics_vertical_scroll = 0;
            self.last_scroll = Instant::now() - SCROLL_TIMEOUT_DURATION; // ensure we auto-scroll immediately
            self.last_title = state.title.clone();
            self.last_artist = state.artists.clone();
        }

        if let Some(lyrics) = &state.lyrics {
            // Check whether the lyrics are LRC or plain text
            let lyrics = if !lyrics.get_timed_lines().is_empty() {
                let offset = lyrics
                    .find_timed_line_index(state.position_ms as i64)
                    .unwrap_or(0);

                // Allow scrolling outside of the auto-scroll window
                if self.last_scroll.elapsed() > SCROLL_TIMEOUT_DURATION {
                    let middle_row = lyrics_area.height / 2;
                    self.lyrics_vertical_scroll = (offset as u16).saturating_sub(middle_row);
                }

                let lines: Vec<_> = lyrics
                    .get_timed_lines()
                    .iter()
                    .enumerate()
                    .map(|(idx, (_time, arc_line))| {
                        let mut line_str = arc_line.as_ref();
                        if line_str.is_empty() {
                            line_str = "♪ ♪ ♪";
                        }
                        let line = Line::from(line_str);

                        if idx < offset {
                            line.dim()
                        } else if idx == offset {
                            line.bold().green()
                        } else {
                            line
                        }
                    })
                    .collect();

                Text::from(lines)
            } else {
                Text::from(lyrics.get_lines())
            };

            // Prevent scrolling past the end
            let max_scroll = (lyrics.lines.len() as u16).saturating_sub(1);
            self.lyrics_vertical_scroll = self.lyrics_vertical_scroll.min(max_scroll);

            let paragraph = Paragraph::new(lyrics)
                .scroll((self.lyrics_vertical_scroll, 0))
                .centered();
            frame.render_widget(paragraph, lyrics_area);
        } else {
            let lyrics = Text::from("NO LYRICS").bold().dim();

            let paragraph = Paragraph::new(lyrics).centered();
            frame.render_widget(
                paragraph,
                lyrics_area.centered_vertically(Constraint::Length(1)),
            );
        };
    }

    fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        if event::poll(timeout)? {
            match event::read()? {
                // It's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit = true;
            }
            KeyCode::Down => {
                self.lyrics_vertical_scroll = self.lyrics_vertical_scroll.saturating_add(1); // I mean, why not ;)
                self.last_scroll = Instant::now();
            }
            KeyCode::Up => {
                self.lyrics_vertical_scroll = self.lyrics_vertical_scroll.saturating_sub(1);
                self.last_scroll = Instant::now();
            }
            _ => {}
        }
    }
}
