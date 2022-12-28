use crate::state::Event;
use crossterm::event::{read, KeyCode};
use futures::executor::block_on;
use futures::future::FutureExt;
use futures::pin_mut;
use futures::select;

type RawEvent = crossterm::event::Event;

pub enum TuiEvent {
    StateEvent(Event),
    CopyClipboard,
    PasteClipboard,
}

pub fn get() -> TuiEvent {
    block_on(get_async())
}

async fn get_async() -> TuiEvent {
    let input = user_input().fuse();
    let timeout = timeout().fuse();

    pin_mut!(input, timeout);

    select! {
        event = input => event,
        () = timeout  => TuiEvent::StateEvent(Event::Timeout),
    }
}

fn char_to_event(c: char) -> TuiEvent {
    match c {
        '[' => TuiEvent::CopyClipboard,
        ']' => TuiEvent::PasteClipboard,
        c => TuiEvent::StateEvent(Event::PrintableString(c.to_string())),
    }
}

async fn user_input() -> TuiEvent {
    TuiEvent::StateEvent(match read().unwrap() {
        RawEvent::Key(key) => match key.code {
            KeyCode::Left => Event::Left,
            KeyCode::Right => Event::Right,
            KeyCode::Up => Event::Up,
            KeyCode::Down => Event::Down,
            KeyCode::Char(c) => return char_to_event(c),
            KeyCode::Enter => Event::Confirm,
            KeyCode::Backspace => Event::Backspace,
            KeyCode::Esc => Event::Exit,
            _ => Event::Other,
        },
        _ => Event::Other,
    })
}

async fn timeout() {
    std::thread::sleep(std::time::Duration::from_millis(500));
}
