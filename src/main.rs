use std::{
    collections::HashSet,
    io::{stdout, Write},
    time::{Duration, SystemTime},
};

use crossterm::{
    cursor::{self, MoveTo},
    event::{poll, read, Event, KeyCode},
    execute,
    style::{style, Color, Print, PrintStyledContent, StyledContent},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use fehler::throws;
use rand::seq::SliceRandom;
use resource::resource_str;
use thiserror::Error;

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

struct Word {
    word: String,
    typed: String,
}

impl Word {
    fn from(word: &str) -> Self {
        let word = word.to_string();
        let typed = String::new();
        Self { word, typed }
    }

    fn add_char(&mut self, c: char) {
        self.typed.push(c);
    }

    fn remove_char(&mut self) {
        self.typed.pop();
    }

    fn is_complete(&self) -> bool {
        self.word == self.typed
    }

    fn styled(&self) -> Vec<StyledContent<char>> {
        let mut styled = vec![];

        for (c, tc) in self.word.chars().zip(self.typed.chars()) {
            let styled_c = if c == tc {
                style(c).with(Color::Blue)
            } else {
                style(c).with(Color::Red)
            };
            styled.push(styled_c);
        }

        styled
    }

    #[throws(crossterm::ErrorKind)]
    fn print(&self) {
        for sc in self.styled() {
            execute!(stdout(), PrintStyledContent(sc))?;
        }
    }
}

fn new_test_word(word_list: &[&str], allowed: &HashSet<char>) -> Word {
    let mut rng = rand::thread_rng();
    let mut word;
    let mut chars;
    loop {
        word = word_list.choose(&mut rng).unwrap();
        chars = word.chars().collect::<HashSet<char>>();
        if chars.is_subset(allowed) {
            return Word::from(word);
        }
    }
}

#[derive(Error, Debug)]
enum MainError {
    CrosstermError(#[from] crossterm::ErrorKind),
    SystemTimeError(#[from] std::time::SystemTimeError),
}

impl std::fmt::Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[throws(MainError)]
fn main() {
    enable_raw_mode()?;
    execute!(stdout(), cursor::Hide)?;

    let words = resource_str!("assets/words_alpha.txt");
    let word_list = words.split_whitespace().collect::<Vec<&str>>();
    let allowed = "aoeuhtns".chars().collect::<HashSet<char>>();
    let mut test_word = new_test_word(&word_list, &allowed);

    let start = SystemTime::now();
    let mut elapsed = 0;
    let mut total_chars = 0;

    loop {
        if poll(Duration::from_millis(100))? {
            if let Event::Key(event) = read()? {
                if event.code == KeyCode::Esc {
                    break;
                }
                let c = map_qwerty_to_dvorak(event.code);
                match c {
                    KeyCode::Backspace => test_word.remove_char(),
                    KeyCode::Char(c) => {
                        if c == ' ' && test_word.is_complete() {
                            elapsed = start.elapsed()?.as_secs();
                            // Add one for the space character.
                            total_chars += test_word.word.len() + 1;
                            test_word = new_test_word(&word_list, &allowed);
                        } else {
                            test_word.add_char(c);
                        }
                    }
                    _ => {}
                }
            }
        }

        execute!(
            stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(test_word.word.to_string()),
            MoveTo(0, 0),
        )?;

        test_word.print()?;

        let wpm = if elapsed == 0 {
            0.0
        } else {
            (total_chars as f64 / 5.0) / (elapsed as f64 / 60.0)
        };

        execute!(
            stdout(),
            MoveTo(0, 2),
            PrintStyledContent(style(format!("{:.0}", wpm)).with(Color::DarkGrey)),
        )?;
    }

    execute!(stdout(), MoveTo(0, 0), Clear(ClearType::All), cursor::Show)?;
    disable_raw_mode()?;
}
