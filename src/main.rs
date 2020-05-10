use std::{
    collections::{HashSet, VecDeque},
    io::{stdout, Write},
    time::{Duration, SystemTime},
};

use crossterm::{
    cursor::{self, MoveTo, RestorePosition, SavePosition},
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{style, Attribute, Color, Print, PrintStyledContent, StyledContent},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use fehler::throws;
use rand::seq::SliceRandom;
use resource::resource_str;

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

#[derive(Debug, Clone)]
#[non_exhaustive]
enum Metric {
    Delimiter {
        value: char,
        duration: Duration,
    },
    Match {
        value: char,
        duration: Duration,
    },
    Typo {
        expected: char,
        typed: char,
        duration: Duration,
    },
}

struct Word<'a> {
    word: &'a str,
    typed: String,
    metrics: Vec<Metric>,
}

struct FinishedWord<'a> {
    word: &'a str,
    metrics: Vec<Metric>,
}

impl<'a> std::convert::From<Word<'a>> for FinishedWord<'a> {
    fn from(word: Word<'a>) -> Self {
        Self {
            word: word.word,
            metrics: word.metrics,
        }
    }
}

impl<'a> Word<'a> {
    fn from(word: &'a str) -> Self {
        let typed = String::new();
        let metrics = vec![];
        Self {
            word,
            typed,
            metrics,
        }
    }

    fn add_char(&mut self, typed: char, duration: Duration) {
        let expected = self.word.chars().nth(self.typed.len());
        if let Some(expected) = expected {
            if typed != expected {
                self.metrics.push(Metric::Typo {
                    expected,
                    typed,
                    duration,
                });
            } else {
                self.metrics.push(Metric::Match {
                    value: typed,
                    duration,
                });
            }
        }
        self.typed.push(typed);
    }

    fn remove_char(&mut self) {
        self.typed.pop();
    }

    fn is_complete(&self) -> bool {
        self.word == self.typed
    }

    fn finalise(mut self, typed: char, duration: Duration) -> FinishedWord<'a> {
        self.metrics.push(Metric::Delimiter {
            value: typed,
            duration,
        });
        self.into()
    }

    fn styled(&self) -> Vec<StyledContent<char>> {
        let mut styled = vec![];

        for (idx, tc) in self.typed.chars().enumerate() {
            let wc = self.word.chars().nth(idx);

            let color = if wc.is_none() || wc.unwrap() != tc {
                Color::Red
            } else {
                Color::Blue
            };

            let c = if tc == ' ' { '␣' } else { tc };

            styled.push(style(c).with(color));
        }

        styled
    }

    #[throws(crossterm::ErrorKind)]
    fn print_typed(&self) {
        for sc in self.styled() {
            queue!(stdout(), PrintStyledContent(sc))?;
        }
    }
}

impl<'a> std::ops::Deref for Word<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.word
    }
}

fn get_test_words<'a>(
    word_list: &[&'a str],
    allowed: &HashSet<char>,
    amount: usize,
) -> VecDeque<&'a str> {
    let mut rng = rand::thread_rng();
    let mut words = VecDeque::new();

    let mut word;
    let mut chars;
    for _ in 0..amount {
        'search: loop {
            word = word_list.choose(&mut rng).unwrap();
            chars = word.chars().collect::<HashSet<char>>();
            if chars.is_subset(allowed) {
                words.push_back(*word);
                break 'search;
            }
        }
    }

    words
}

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
enum Error {
    CrosstermError(#[from] crossterm::ErrorKind),
    IoError(#[from] std::io::Error),
    SystemTimeError(#[from] std::time::SystemTimeError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[throws]
fn typing_test<'a>(mut test_words: VecDeque<&'a str>) -> Vec<FinishedWord> {
    let mut test_word = Word::from(test_words.pop_front().unwrap());
    let mut typed = String::new();
    let mut finished_words = vec![];

    let mut start_char = SystemTime::now();

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
                            typed.push_str(&test_word);
                            typed.push(' ');

                            let finished_word = test_word.finalise(c, start_char.elapsed()?);
                            finished_words.push(finished_word);

                            test_word = match test_words.pop_front() {
                                Some(word) => {
                                    start_char = SystemTime::now();
                                    Word::from(word)
                                }
                                None => break,
                            };
                        } else {
                            test_word.add_char(c, start_char.elapsed()?);
                            start_char = SystemTime::now();
                        }
                    }
                    _ => {}
                }
            }
        }

        let remaining_words = test_words
            .iter()
            .map(|s| format!(" {}", s))
            .collect::<String>();

        execute!(
            stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            PrintStyledContent(style(&typed).with(Color::DarkGrey)),
            SavePosition,
            Print(test_word.chars().next().unwrap()),
        )?;

        let (c, r) = cursor::position()?;
        if c == 1 {
            queue!(stdout(), MoveTo(0, r))?;
        } else {
            queue!(stdout(), RestorePosition)?;
        }

        queue!(
            stdout(),
            SavePosition,
            Print(&*test_word),
            PrintStyledContent(style(remaining_words).with(Color::DarkGrey)),
            RestorePosition,
        )?;

        test_word.print_typed()?;

        stdout().flush()?;
    }

    finished_words
}

#[throws]
fn main() {
    enable_raw_mode()?;
    execute!(stdout(), cursor::Hide)?;

    let words = resource_str!("assets/words_alpha.txt");
    let word_list = words.split_whitespace().collect::<Vec<&str>>();

    // Lesson 1 - Home row, 8 keys (starting positions)
    // Lesson 2 - Home row, 10 keys
    // Lesson 3 - Home row + C, F, K, L, M, P, R, V
    // Lesson 4 - Home row + B, G, J, Q, W, X, Y, Z
    // Lesson 5 - The entire roman alphabet
    'lessons: for lesson_alphabet in &[
        "aoeuhtns",
        "aoeuidhtns",
        "aoeuidhtnscfklmprv",
        "aoeuidhtnsbgjqwxyz",
        "abcdefghijklmnopqrstuvwxyz",
    ] {
        let allowed = lesson_alphabet.chars().collect::<HashSet<char>>();

        let test_words = get_test_words(&word_list, &allowed, 100);
        let finished_words = typing_test(test_words)?;

        let word_cnt: f64 = (finished_words
            .iter()
            .fold(0, |acc, word| acc + word.word.chars().count())
            as f64
            + finished_words.len() as f64)
            / 5.0;

        let duration = finished_words
            .iter()
            .fold(Duration::default(), |acc, word| {
                acc + word
                    .metrics
                    .iter()
                    .fold(Duration::default(), |acc, metric| match metric {
                        Metric::Delimiter { duration, .. }
                        | Metric::Match { duration, .. }
                        | Metric::Typo { duration, .. } => acc + *duration,
                        _ => acc,
                    })
            });

        let wpm_avg = if !finished_words.is_empty() {
            word_cnt / (duration.as_secs_f64() / 60.0)
        } else {
            0.0
        };

        let typos = finished_words.iter().fold(0, |acc, word| {
            acc + word.metrics.iter().fold(0, |acc, metric| {
                if let Metric::Typo { .. } = metric {
                    acc + 1
                } else {
                    acc
                }
            })
        });

        queue!(
            stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            PrintStyledContent(style(format!("{:.0}", wpm_avg)).attribute(Attribute::Underlined)),
            Print(format!(" {}\r\n", typos)),
        )?;

        stdout().flush()?;

        'hold: loop {
            if poll(Duration::from_millis(100))? {
                if let Event::Key(event) = read()? {
                    if event.code == KeyCode::Esc {
                        break 'lessons;
                    } else if event.code == KeyCode::Enter {
                        break 'hold;
                    }
                }
            }
        }
    }

    execute!(stdout(), MoveTo(0, 0), Clear(ClearType::All), cursor::Show)?;
    disable_raw_mode()?;
}
