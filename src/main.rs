use std::thread;
use std::time::{Duration, Instant};

use ::mpris::PlayerFinder;

use crate::mpris::PlayerState;
use crate::tui::App;

mod mpris;
mod tui;

/// How often to refresh the screen and position
const REFRESH_INTERVAL: Duration = Duration::from_millis(1000 / 30);
/// How often to refresh metadata
const METADATA_REFRESH_INTERVAL: Duration = Duration::from_millis(1000 / 5);
/// How much time to wait since last scroll before auto-scrolling takes over again
const SCROLL_TIMEOUT_DURATION: Duration = Duration::from_millis(1000);
/// The threshold without lyrics acceptable before inserting a "♪ ♪ ♪"
const LRC_START_THRESHOLD: Duration = Duration::from_millis(5000);

fn main() {
    let mut terminal = ratatui::init();
    let finder = PlayerFinder::new().expect("failed to connect to D-Bus"); // TODO: return error instead of panicking

    loop {
        let player = match finder.find_active() {
            Ok(p) => p,
            Err(_) => {
                thread::sleep(REFRESH_INTERVAL);
                continue;
            }
        };

        let name = player.identity().to_string();
        let mut state = PlayerState::default();

        let mut app = App::new(name);

        loop {
            let frame_start = Instant::now();

            if !player.is_running() {
                break;
            }

            if state.update(&player).is_err() {
                break;
            }

            app.frame(&mut terminal, frame_start, &state)
                .expect("failed to draw frame to terminal") // TODO: return error instead of panicking
        }
    }
}
