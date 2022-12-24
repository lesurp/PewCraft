use crate::state::{Event, GlobalState, State};
use crate::tui_impl::input::{self, TuiEvent};
use crate::tui_impl::render::Renderer;
use clipboard::{ClipboardContext, ClipboardProvider};
use common::game::GameDefinition;
use crossterm::ExecutableCommand;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug};
use std::io::{Stdin, Stdout, StdoutLock, Write};
use tui::backend::CrosstermBackend;
use tui::Terminal;

pub struct Tui<'a> {
    game_definition: &'a GameDefinition,
    //stdin: Bytes<StdinLock<'a>>,
    stdout: Terminal<CrosstermBackend<StdoutLock<'a>>>,
    clipboard: ClipboardContext,
}

impl<'a> Tui<'a> {
    pub fn new(game_definition: &'a GameDefinition, _: &'a Stdin, stdout: &'a mut Stdout) -> Self {
        debug!("Enabling raw mode");
        enable_raw_mode().unwrap();
        debug!("Raw mode enabled");
        stdout.execute(EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout.lock());
        //let stdin = stdin.lock().bytes();
        let stdout = Terminal::new(backend).unwrap();
        let clipboard = ClipboardProvider::new().unwrap();
        Tui {
            game_definition,
            //stdin,
            stdout,
            clipboard,
        }
    }

    pub fn render(&mut self, s: &GlobalState) -> Event {
        debug!("tui.rs:render");

        self.stdout.hide_cursor().unwrap();

        let g = self.game_definition;
        self.stdout
            .draw(|mut f| Renderer::render(&mut f, s, g))
            .unwrap();

        debug!("Current state: {:?}", s);

        match input::get(s.expected_event()) {
            TuiEvent::StateEvent(e) => e,
            // TODO
            TuiEvent::CopyClipboard => {
                if let Some(string) = s.get_game_id() {
                    self.clipboard.set_contents(string).unwrap();
                }
                Event::Other
            }
            // TODO
            TuiEvent::PasteClipboard => {
                Event::PrintableString(self.clipboard.get_contents().unwrap())
            }
        }
    }
}

impl<'a> Drop for Tui<'a> {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(self.stdout.backend_mut(), LeaveAlternateScreen).unwrap();
    }
}
