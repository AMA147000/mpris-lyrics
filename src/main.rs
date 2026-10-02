use std::thread;
use std::time::{Duration, Instant};

use ::mpris::PlayerFinder;

use crate::mpris::PlayerState;
use crate::tui::App;

mod mpris;
mod tui;

const REFRESH_RATE: u64 = 30; // Hz (how often to refresh the screen and position)
const METADATA_REFRESH_RATE: u64 = 5; // Hz (how often to refresh metadata)
const SCROLL_TIMEOUT: Duration = Duration::from_millis(1000); // ms (how much time to wait since last scroll before auto-scrolling takes over again)
const LRC_START_THRESHOLD: u32 = 5000; // ms (the threshold without lyrics acceptable before inserting a "♪ ♪ ♪")

const REFRESH_DURATION: Duration = Duration::from_millis(1000 / REFRESH_RATE);
const METADATA_REFRESH_DURATION: Duration = Duration::from_millis(1000 / METADATA_REFRESH_RATE);

fn main() {
    let mut terminal = ratatui::init();
    let finder = PlayerFinder::new().expect("failed to connect to D-Bus"); // TODO: return error instead of panicking

    loop {
        let player = match finder.find_active() {
            Ok(p) => p,
            Err(_) => {
                thread::sleep(REFRESH_DURATION);
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
