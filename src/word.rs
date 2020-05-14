use std::time::Duration;

use tui::{
    style::{Color, Style},
    widgets::Text,
};

use crate::metrics::Metric;

#[derive(Debug, PartialEq)]
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

    pub fn len(&self) -> usize {
        self.value.chars().count()
    }

    pub fn typed_len(&self) -> usize {
        self.typed.chars().count()
    }

    pub fn overflow(&self) -> usize {
        self.typed_len().saturating_sub(self.len())
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

    pub fn styled_text(&self) -> Vec<Text> {
        let mut styled = vec![];

        // Display the typed characters.
        for (idx, tc) in self.typed.chars().enumerate() {
            let wc = self.char_at(idx);

            let color = if wc.is_none() || wc.unwrap() != tc {
                Color::Red
            } else {
                Color::Blue
            };

            let c = if tc == ' ' { '␣' } else { tc };

            styled.push(Text::styled(c.to_string(), Style::default().fg(color)));
        }

        // Fill in the untyped characters.
        for idx in self.typed_len()..self.len() {
            if let Some(c) = self.char_at(idx) {
                styled.push(Text::raw(c.to_string()));
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_match_metric() {
        let mut word: Word = "test".into();
        let expected = Word {
            value: "test",
            typed: "t".to_string(),
            metrics: vec![Metric::Match {
                value: 't',
                duration: Duration::from_secs(1),
            }],
        };

        word.add_char('t', Duration::from_secs(1));

        assert_eq!(word, expected);
    }
    #[test]
    fn test_generate_typo_metric() {
        let mut word: Word = "test".into();
        let expected = Word {
            value: "test",
            typed: "e".to_string(),
            metrics: vec![Metric::Typo {
                value: 'e',
                expected: 't',
                duration: Duration::from_secs(1),
            }],
        };

        word.add_char('e', Duration::from_secs(1));

        assert_eq!(word, expected);
    }
}
