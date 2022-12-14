use crate::state::{GlobalState, State};
use log::{debug, info};
use std::io::{stdin, stdout};

mod api;
mod state;
mod tui;

fn main() {
    env_logger::init();

    let url = "http://localhost:8000";
    let endpoint = api::Endpoint::new(url);
    let game = endpoint.load_game();

    let stdin = stdin();
    let mut stdout = stdout();
    let mut tui = tui::Tui::new(&game, &stdin, &mut stdout);
    let mut s = GlobalState::new(&game, &endpoint);

    loop {
        debug!("Current state: {:?}", s);
        let input = tui.render(&s);
        debug!("Received input: {:?}", input);

        s = s.next(input);
        if s.exit() {
            break;
        }
    }

    info!("Exiting tui");
}
