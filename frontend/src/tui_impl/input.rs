use crate::state::{Event, ExpectedEvent};
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

pub fn get(ev: ExpectedEvent) -> TuiEvent {
    block_on(get_async(ev))
}

async fn get_async(ev: ExpectedEvent) -> TuiEvent {
    let input = user_input(ev).fuse();
    let timeout = timeout().fuse();

    pin_mut!(input, timeout);

    select! {
        event = input => event,
        () = timeout  => TuiEvent::StateEvent(Event::Timeout),
    }
}

fn char_to_event(c: char, ev: ExpectedEvent) -> TuiEvent {
    TuiEvent::StateEvent(if matches!(ev, ExpectedEvent::Char) {
        Event::PrintableString(c.to_string())
    } else {
        match c {
            'q' => Event::Exit,
            'h' => Event::Left,
            'l' => Event::Right,
            'j' => Event::Down,
            'k' => Event::Up,
            'y' => return TuiEvent::CopyClipboard,
            '[' => return TuiEvent::PasteClipboard,
            c => Event::PrintableString(c.to_string()),
        }
    })
}

async fn user_input(ev: ExpectedEvent) -> TuiEvent {
    TuiEvent::StateEvent(match read().unwrap() {
        RawEvent::Key(key) => match key.code {
            KeyCode::Left => Event::Left,
            KeyCode::Right => Event::Right,
            KeyCode::Up => Event::Up,
            KeyCode::Down => Event::Down,
            KeyCode::Char(c) => return char_to_event(c, ev),
            KeyCode::Enter => Event::Confirm,
            KeyCode::Backspace => Event::Backspace,
            _ => Event::Other,
        },
        _ => Event::Other,
    })
}

async fn timeout() {
    std::thread::sleep(std::time::Duration::from_millis(500));
}
