mod keymap;
mod metrics;
mod typingtest;
mod word;

use std::{
    collections::{HashSet, VecDeque},
    fmt::{self, Display, Formatter},
    io::{stdout, Write},
};

use crossterm::{
    event::{read, Event, KeyCode},
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
    widgets::{Block, Borders, Paragraph, Sparkline, Text},
    Terminal,
};

use typingtest::typing_test;

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
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(30),
                        Constraint::Percentage(30),
                        Constraint::Percentage(40),
                    ]
                    .as_ref(),
                )
                .split(frame.size());

            let row_0_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(rows[0]);

            let row_1_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ]
                    .as_ref(),
                )
                .split(rows[1]);

            let block = Block::default()
                .title("wpm")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(block, row_0_chunks[0]);
            let text = [Text::raw(format!("{:.0}", test_results.wpm_avg()))];
            let paragraph = Paragraph::new(text.iter()).block(block);
            frame.render_widget(paragraph, row_0_chunks[0]);

            let block = Block::default()
                .title("typos")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(block, row_0_chunks[1]);
            let text = [Text::raw(format!("{}", test_results.typo_cnt()))];
            let paragraph = Paragraph::new(text.iter()).block(block);
            frame.render_widget(paragraph, row_0_chunks[1]);

            let block = Block::default()
                .title("words typed")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(block, row_1_chunks[0]);
            let text = [Text::raw(format!("{}", test_results.word_cnt()))];
            let paragraph = Paragraph::new(text.iter()).block(block);
            frame.render_widget(paragraph, row_1_chunks[0]);

            let block = Block::default()
                .title("characters typed")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(block, row_1_chunks[1]);
            let text = [Text::raw(format!("{}", test_results.char_cnt()))];
            let paragraph = Paragraph::new(text.iter()).block(block);
            frame.render_widget(paragraph, row_1_chunks[1]);

            let block = Block::default()
                .title("total seconds")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(block, row_1_chunks[2]);
            let text = [Text::raw(format!("{:.1}", test_results.duration_secs()))];
            let paragraph = Paragraph::new(text.iter()).block(block);
            frame.render_widget(paragraph, row_1_chunks[2]);

            let word_durations = test_results.normalised_word_durations();
            let block = Block::default()
                .title("word times (normalised)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(block, rows[2]);
            let sparkline = Sparkline::default().data(&word_durations).block(block);
            frame.render_widget(sparkline, rows[2]);
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
