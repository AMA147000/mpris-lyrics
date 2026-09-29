use std::thread;
use std::time::{Duration, Instant};

use ::mpris::PlayerFinder;

use crate::mpris::PlayerState;
use crate::tui::render;

mod mpris;
mod tui;

const REFRESH_RATE: u64 = 30;
const METADATA_REFRESH_RATE: u64 = 5;

const REFRESH_DURATION: Duration = Duration::from_millis(1000 / REFRESH_RATE);
const METADATA_REFRESH_DURATION: Duration = Duration::from_millis(1000 / METADATA_REFRESH_RATE);

fn main() {
    let finder = PlayerFinder::new().expect("failed to connect to D-Bus");

    loop {
        let player = match finder.find_active() {
            Ok(p) => p,
            Err(_) => {
                thread::sleep(REFRESH_DURATION);
                continue;
            }
        };

        let name = player.identity();
        let mut state = PlayerState::default();

        loop {
            let frame_start = Instant::now();

            if !player.is_running() {
                break;
            }

            if state.update(&player).is_err() {
                break;
            }

            render(name, state.clone());

            if let Some(remaining) = REFRESH_DURATION.checked_sub(frame_start.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }
}
