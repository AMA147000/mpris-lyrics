use std::path::PathBuf;
use std::process::exit;
use std::time::{Duration, Instant, SystemTime};
use std::{env, fs, thread};

use ::mpris::PlayerFinder;

use crate::mpris::{PlayerState, parse_lyrics};
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

const HELP_MESSAGE: &str = "MPRIS Lyrics
A simple Linux CLI that displays synced lyrics from the MPRIS `xesam:asText` property

usage: mpris-lyrics [options]
    options:
        -l, --lrc <path>   specifies a single LRC file to override the lyrics of all the songs with
        -v, --version      display the current version
        -h, --help         display this help message

    note: when using the `-l, --lrc` options, changes in the file reflect automatically (useful as \
                            live feedback when creating an LRC file)
";

fn main() {
    let args: Vec<String> = env::args().collect();
    let override_file = parse_args(args);

    let mut terminal = ratatui::init();
    let finder = PlayerFinder::new().expect("failed to connect to D-Bus"); // TODO: return error instead of panicking

    let mut modification_timestamp = SystemTime::UNIX_EPOCH;
    let mut override_lyrics = None;

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

            if let Some(path) = &override_file {
                if fs::exists(path).unwrap_or_default() {
                    let new_modification_timestamp = fs::metadata(path)
                        .expect("failed to poll metadata for the override lrc file")
                        .modified()
                        .expect(
                            "failed to acess the modification timestamp of the override lrc file",
                        );

                    if new_modification_timestamp > modification_timestamp {
                        let string = fs::read_to_string(path).ok();
                        modification_timestamp = new_modification_timestamp;
                        override_lyrics = parse_lyrics(string.as_deref());
                    }
                } else {
                    modification_timestamp = SystemTime::UNIX_EPOCH;
                    override_lyrics = None;
                }
            }

            app.frame(&mut terminal, frame_start, &state, &override_lyrics)
                .expect("failed to draw frame to terminal") // TODO: return error instead of panicking
        }
    }
}

fn parse_args(args: Vec<String>) -> Option<PathBuf> {
    if args.len() == 1 {
        None
    } else if args.len() == 2 && (args[1] == "-h" || args[1] == "--help") {
        println!("{}", HELP_MESSAGE);
        exit(0);
    } else if args.len() == 2 && (args[1] == "-v" || args[1] == "--version") {
        let version = env!("CARGO_PKG_VERSION");
        println!("v{}", version);
        exit(0);
    } else if args.len() == 3 && (args[1] == "-l" || args[1] == "--lrc") {
        Some(args[2].clone().into())
    } else {
        println! {"invalid arguments"}
        println!("{}", HELP_MESSAGE);
        exit(1);
    }
}
