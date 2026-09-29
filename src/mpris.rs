use lrc::Lyrics;
use mpris::Player;

#[derive(Clone)]
pub(crate) struct PlayerState {
    pub(crate) title: Option<String>,
    pub(crate) artists: Option<Vec<String>>,
    pub(crate) position_ms: u64,
    pub(crate) lyrics: Option<Lyrics>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            title: None,
            artists: None,
            position_ms: 0,
            lyrics: None,
        }
    }
}

impl PlayerState {
    pub(crate) fn update(&mut self, player: &Player) -> Result<(), mpris::DBusError> {
        let metadata = player.get_metadata()?;

        self.title = metadata.title().map(|s| s.to_string());
        self.artists = metadata
            .artists()
            .map(|artists| artists.into_iter().map(|s| s.to_string()).collect());
        self.position_ms = player.get_position_in_microseconds().unwrap_or(0) / 1000;

        let lyrics_str = metadata.get("xesam:asText").and_then(|v| v.as_str());
        self.lyrics = match lyrics_str {
            Some(text) => {
                if text.is_empty() {
                    None
                } else {
                    Some(text.parse().expect("failed to parse lyrics"))
                }
            }
            None => None,
        };

        Ok(())
    }
}
