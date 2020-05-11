use crate::Error;

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
fn typing_test<'a>(mut test_words: VecDeque<&'a str>) -> TestResults {
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

        let remaining_words = test_words
            .iter()
            .map(|s| format!(" {}", s))
            .collect::<String>();

        execute!(
            stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(style(&typed).with(Color::DarkGrey)),
            SavePosition,
            Print(test_word.char_at(0).unwrap()),
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
            Print(test_word.as_str()),
            Print(style(remaining_words).with(Color::DarkGrey)),
            RestorePosition,
            Print(&test_word),
        )?;

        stdout().flush()?;
    }

    TestResults(finished_words)
}

