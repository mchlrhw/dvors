use std::{
    fmt,
    io::{stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor::{self, MoveTo},
    event::{poll, read, Event, KeyCode},
    execute,
    style::{style, Colorize, PrintStyledContent, StyledContent},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ErrorKind, ExecutableCommand,
};
use fehler::throws;

fn map_qwerty_to_dvorak(code: KeyCode) -> KeyCode {
    if let KeyCode::Char(c) = code {
        let mapped = match c {
            '-' => '[',
            '_' => '{',
            '=' => ']',
            '+' => '}',
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
            's' => 'o',
            'S' => 'O',
            'd' => 'e',
            'D' => 'E',
            'f' => 'u',
            'F' => 'U',
            'g' => 'i',
            'G' => 'I',
            'h' => 'd',
            'H' => 'D',
            'j' => 'h',
            'J' => 'H',
            'k' => 't',
            'K' => 'T',
            'l' => 'n',
            'L' => 'N',
            ';' => 's',
            ':' => 'S',
            '\'' => '-',
            '"' => '_',
            'z' => ';',
            'Z' => ':',
            'x' => 'q',
            'X' => 'Q',
            'c' => 'j',
            'C' => 'J',
            'v' => 'k',
            'V' => 'K',
            'b' => 'x',
            'B' => 'X',
            'n' => 'b',
            'N' => 'B',
            ',' => 'w',
            '<' => 'W',
            '.' => 'v',
            '>' => 'V',
            '/' => 'z',
            '?' => 'Z',
            _ => c,
        };

        KeyCode::Char(mapped)
    } else {
        code
    }
}

#[throws(ErrorKind)]
fn main() {
    enable_raw_mode()?;
    execute!(stdout(), cursor::Hide)?;

    loop {
        if poll(Duration::from_millis(500))? {
            if let Event::Key(event) = read()? {
                if event.code == KeyCode::Esc {
                    break;
                }
                let c = map_qwerty_to_dvorak(event.code);
            }
        }
    }

    execute!(stdout(), MoveTo(0, 0), Clear(ClearType::All), cursor::Show)?;
    disable_raw_mode()?;
}
