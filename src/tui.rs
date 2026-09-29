use crate::mpris::PlayerState;

pub(crate) fn render(player_name: &str, state: PlayerState) {
    print!("\x1B[H\x1B[J"); // Jump home and clear
    println!("{}", player_name);
    println!(
        "{} - {}",
        state.title.unwrap_or("Unknown".to_string()),
        state
            .artists
            .unwrap_or(vec!["Unknown".to_string()])
            .join(", ")
    );
    println!("{}", state.position_ms);

    if let Some(lyrics) = state.lyrics {
        // check whether the lyrics are LRC or plain text
        if !lyrics.get_timed_lines().is_empty() {
            println!("{}", lyrics.line_at(state.position_ms as i64).unwrap_or(""));
        } else {
            println!("{}", lyrics.get_lines().join("\n"));
        }
    } else {
        println!("NO LYRICS");
    }
}
