mod keymap;
mod metrics;
mod word;

use std::{
    collections::{HashSet, VecDeque},
    convert::From,
    fmt::{self, Display, Formatter},
    io::{stdout, Write},
    time::{Duration, SystemTime},
};

use crossterm::{
    event::{poll, read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fehler::throws;
use rand::seq::SliceRandom;
use resource::resource_str;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Text},
    Terminal,
};

use metrics::Metric;
use word::{FinishedWord, Word};

struct TestResults<'a>(Vec<FinishedWord<'a>>);

impl TestResults<'_> {
    fn wpm_avg(&self) -> f64 {
        let char_cnt = self
            .0
            .iter()
            .fold(0, |acc, word| acc + word.len_inc_delim());
        let word_cnt = char_cnt as f64 / 5.0;

        let duration = self.0.iter().fold(Duration::default(), |acc, word| {
            acc + word
                .metrics()
                .iter()
                .fold(Duration::default(), |acc, metric| match metric {
                    Metric::Delimiter { duration, .. }
                    | Metric::Match { duration, .. }
                    | Metric::Typo { duration, .. } => acc + *duration,
                })
        });

        if !self.0.is_empty() {
            word_cnt / (duration.as_secs_f64() / 60.0)
        } else {
            0.0
        }
    }

    fn typo_cnt(&self) -> usize {
        self.0.iter().fold(0, |acc, word| {
            acc + word.metrics().iter().fold(0, |acc, metric| {
                if let Metric::Typo { .. } = metric {
                    acc + 1
                } else {
                    acc
                }
            })
        })
    }
}

#[throws]
fn typing_test<'a, 'b, B: tui::backend::Backend>(
    terminal: &'b mut Terminal<B>,
    mut test_words: VecDeque<&'a str>,
) -> TestResults<'a> {
    let mut test_word = Word::from(test_words.pop_front().unwrap());
    let mut typed = String::new();
    let mut finished_words = vec![];

    let mut start_char = SystemTime::now();

    loop {
        terminal.draw(|mut frame| {
            let size = frame.size();

            let remaining_words = test_words
                .iter()
                .map(|s| format!(" {}", s))
                .collect::<String>()
                .chars()
                .skip(test_word.overflow())
                .collect::<String>();

            let mut text = vec![Text::styled(&typed, Style::default().fg(Color::DarkGray))];
            text.extend_from_slice(&test_word.styled_text());
            text.push(Text::styled(
                remaining_words,
                Style::default().fg(Color::DarkGray),
            ));

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            let paragraph = Paragraph::new(text.iter()).block(block).wrap(true);
            frame.render_widget(paragraph, size);
        })?;

        if let Event::Key(event) = read()? {
            if event.code == KeyCode::Esc {
                break;
            }
            let c = keymap::qwerty_to_dvorak(event.code);
            match c {
                KeyCode::Backspace => test_word.remove_char(),
                KeyCode::Char(c) => {
                    if c == ' ' && test_word.is_complete() {
                        typed.push_str(test_word.as_str());
                        typed.push(' ');

                        let finished_word = test_word.finalise(c, start_char.elapsed()?);
                        finished_words.push(finished_word);

                        test_word = match test_words.pop_front() {
                            Some(word) => {
                                start_char = SystemTime::now();
                                word.into()
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

    TestResults(finished_words)
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

impl Display for Error {
    #[throws(fmt::Error)]
    fn fmt(&self, f: &mut Formatter<'_>) {
        write!(f, "{:?}", self)?;
    }
}

#[throws]
fn main() {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.autoresize()?;
    terminal.hide_cursor()?;

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
        let test_results = typing_test(&mut terminal, test_words)?;

        terminal.draw(|mut frame| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(frame.size());

            let block = Block::default()
                .title("wpm")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(block, chunks[0]);
            let text = [Text::raw(format!("{:.0}", test_results.wpm_avg()))];
            let paragraph = Paragraph::new(text.iter()).block(block);
            frame.render_widget(paragraph, chunks[0]);

            let block = Block::default()
                .title("typos")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(block, chunks[1]);
            let text = [Text::raw(format!("{}", test_results.typo_cnt()))];
            let paragraph = Paragraph::new(text.iter()).block(block);
            frame.render_widget(paragraph, chunks[1]);
        })?;

        'hold: loop {
            if let Event::Key(event) = read()? {
                if event.code == KeyCode::Esc {
                    break 'lessons;
                } else if event.code == KeyCode::Enter {
                    break 'hold;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
}
