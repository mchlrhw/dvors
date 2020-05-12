use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Metric {
    Delimiter {
        value: char,
        duration: Duration,
    },
    Match {
        value: char,
        duration: Duration,
    },
    Typo {
        value: char,
        expected: char,
        duration: Duration,
    },
}
