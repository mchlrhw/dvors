use std::{
    io::{stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ErrorKind, ExecutableCommand,
};
use fehler::throws;

fn map_qwerty_to_dvorak(code: KeyCode) -> KeyCode {
    if let KeyCode::Char(c) = code {
        let mapped = match c {
            'q' => '\'',
            'Q' => '"',
            'w' => ',',
            'W' => '<',
            'e' => '.',
            'E' => '>',
            'r' => 'p',
            'R' => 'P',
            't' => 'y',
            'T' => 'Y',
            'y' => 'f',
            'Y' => 'F',
            'u' => 'g',
            'U' => 'G',
            'i' => 'c',
            'I' => 'C',
            'o' => 'r',
            'O' => 'R',
            'p' => 'l',
            'P' => 'L',
            '[' => '/',
            '{' => '?',
            ']' => '=',
            '}' => '+',
            // TODO: The rest...
            _ => c,
        };

        KeyCode::Char(mapped)
    } else {
        code
    }
}

#[throws(ErrorKind)]
fn print_keyboard() {
    let keyboard = "' , . p y f g c r l / = \\\r\n a o e u i d h t n s -\r\n  ; q j k x b m w v z";
    execute!(
        stdout(),
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print(format!("{}\n", keyboard)),
        MoveTo(0, 0),
    )?;
}

#[throws(ErrorKind)]
fn main() {
    enable_raw_mode()?;

    loop {
        if poll(Duration::from_millis(500))? {
            match read()? {
                Event::Key(event) => {
                    if event.code == KeyCode::Esc {
                        break;
                    }
                    let c = map_qwerty_to_dvorak(event.code);
                    execute!(stdout(), Print(format!("{:?}", c)),)?;
                }
                _ => {}
            }
        } else {
            print_keyboard()?;
        }
    }

    disable_raw_mode()?;
}
