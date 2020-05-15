use std::{
    collections::VecDeque,
    time::{Duration, SystemTime},
};

use crossterm::event::{read, Event, KeyCode};
use fehler::throws;
use tui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Text},
    Terminal,
};

use crate::{
    keymap,
    metrics::Metric,
    word::{FinishedWord, Word},
    Error,
};

pub struct TestResults<'a>(Vec<FinishedWord<'a>>);

impl TestResults<'_> {
    pub fn word_cnt(&self) -> usize {
        self.0.len()
    }

    pub fn char_cnt(&self) -> usize {
        self.0
            .iter()
            .fold(0, |acc, word| acc + word.len_inc_delim())
    }

    pub fn duration_secs(&self) -> f64 {
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

        duration.as_secs_f64()
    }

    pub fn wpm_avg(&self) -> f64 {
        let word_cnt = self.char_cnt() as f64 / 5.0;

        if !self.0.is_empty() {
            word_cnt / (self.duration_secs() / 60.0)
        } else {
            0.0
        }
    }

    pub fn typo_cnt(&self) -> usize {
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
pub(crate) fn typing_test<'a, 'b, B: tui::backend::Backend>(
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
