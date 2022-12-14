use crate::state::{GlobalState, State};
use log::{debug, info};
use std::io::{stdin, stdout};

mod api;
mod state;
mod tui_impl;

fn main() {
    env_logger::init();

    let url = "http://localhost:8000";
    let endpoint = api::Endpoint::new(url);
    debug!("Created API endpoint");
    let game = endpoint.load_game();
    info!("Loaded game from server");

    let stdin = stdin();
    let mut stdout = stdout();
    let mut tui = tui_impl::Tui::new(&game, &stdin, &mut stdout);
    let mut s = GlobalState::new();
    debug!("Created game state");

    loop {
        debug!("Current state: {:?}", s);
        let input = tui.render(&s);
        debug!("Received input: {:?}", input);

        s = s.next(&game, &endpoint, input);
        if s.exit() {
            break;
        }
    }

    info!("Exiting tui");
}
