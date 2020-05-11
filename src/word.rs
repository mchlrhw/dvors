use std::fmt::{self, Display, Formatter};
use std::time::Duration;

use crossterm::style::{style, Color, StyledContent};
use fehler::throws;

use crate::metrics::Metric;

pub(crate) struct Word<'a> {
    value: &'a str,
    typed: String,
    metrics: Vec<Metric>,
}

pub(crate) struct FinishedWord<'a> {
    value: &'a str,
    metrics: Vec<Metric>,
}

impl<'a> FinishedWord<'a> {
    fn len(&self) -> usize {
        self.value.chars().count()
    }

    pub fn len_inc_delim(&self) -> usize {
        self.len() + 1
    }

    pub fn metrics(&self) -> &[Metric] {
        &self.metrics
    }
}

impl<'a> From<Word<'a>> for FinishedWord<'a> {
    fn from(word: Word<'a>) -> Self {
        Self {
            value: word.value,
            metrics: word.metrics,
        }
    }
}

impl<'a> Word<'a> {
    pub fn as_str(&self) -> &'a str {
        self.value
    }

    pub fn char_at(&self, idx: usize) -> Option<char> {
        self.value.chars().nth(idx)
    }

    pub fn add_char(&mut self, typed: char, duration: Duration) {
        let expected = self.char_at(self.typed.len());
        if let Some(expected) = expected {
            if typed != expected {
                self.metrics.push(Metric::Typo {
                    value: typed,
                    expected,
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

    pub fn remove_char(&mut self) {
        self.typed.pop();
    }

    pub fn is_complete(&self) -> bool {
        self.value == self.typed
    }

    pub fn finalise(mut self, typed: char, duration: Duration) -> FinishedWord<'a> {
        self.metrics.push(Metric::Delimiter {
            value: typed,
            duration,
        });
        self.into()
    }

    fn styled(&self) -> Vec<StyledContent<char>> {
        let mut styled = vec![];

        for (idx, tc) in self.typed.chars().enumerate() {
            let wc = self.char_at(idx);

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
}

impl<'a> From<&'a str> for Word<'a> {
    fn from(string: &'a str) -> Self {
        let typed = String::new();
        let metrics = vec![];
        Self {
            value: string,
            typed,
            metrics,
        }
    }
}

impl Display for Word<'_> {
    #[throws(fmt::Error)]
    fn fmt(&self, f: &mut Formatter<'_>) {
        for sc in self.styled() {
            write!(f, "{}", sc)?;
        }
    }
}
