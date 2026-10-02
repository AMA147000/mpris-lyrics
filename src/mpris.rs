use std::time::Instant;

use lrc::{Lyrics, TimeTag};
use mpris::Player;

use crate::{LRC_START_THRESHOLD, METADATA_REFRESH_DURATION};

#[derive(Clone)]
pub(crate) struct PlayerState {
    pub(crate) position_ms: u64,
    pub(crate) title: Option<String>,
    pub(crate) artists: Option<Vec<String>>,
    pub(crate) lyrics: Option<Lyrics>,
    metadata_last_updated: Instant,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            position_ms: 0,
            title: None,
            artists: None,
            lyrics: None,
            metadata_last_updated: {
                let instant = Instant::now();
                instant - METADATA_REFRESH_DURATION
            },
        }
    }
}

impl PlayerState {
    pub(crate) fn update(&mut self, player: &Player) -> Result<(), mpris::DBusError> {
        self.position_ms = player.get_position_in_microseconds().unwrap_or(0) / 1000;

        if self.metadata_last_updated.elapsed() > METADATA_REFRESH_DURATION {
            let metadata = player.get_metadata()?;

            self.title = metadata.title().map(|s| s.to_string());
            self.artists = metadata
                .artists()
                .map(|artists| artists.into_iter().map(|s| s.to_string()).collect());

            let lyrics_str = metadata.get("xesam:asText").and_then(|v| v.as_str());
            self.lyrics = match lyrics_str {
                Some(text) => {
                    if text.is_empty() {
                        None
                    } else {
                        let mut lyrics: Lyrics = text.parse().expect("failed to parse lyrics");
                        // Check whether the lyrics are LRC or plain text
                        if let Some(first_line) = lyrics.get_timed_lines().first() {
                            // Add starting line if first timestamp is after more than LRC_START_THRESHOLD
                            if first_line.0 > TimeTag::new(LRC_START_THRESHOLD) {
                                lyrics
                                    .add_timed_line(TimeTag::new(0), "")
                                    .expect("failed to insert starting line"); // TODO: return error instead of panicking
                            } else if first_line.0 > TimeTag::new(0) {
                                lyrics
                                    .add_timed_line(TimeTag::new(0), " ")
                                    .expect("failed to insert starting line"); // TODO: return error instead of panicking
                            }
                        }

                        Some(lyrics)
                    }
                }
                None => None,
            };

            self.metadata_last_updated = Instant::now();
        }

        Ok(())
    }
}
