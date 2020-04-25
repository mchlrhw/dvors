#[derive(Copy, Clone, Debug, PartialEq)]
enum Key {
    BackTick,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
    OpenBracket,
    CloseBracket,
    Quote,
    Comma,
    Period,
    P,
    Y,
    F,
    G,
    C,
    R,
    L,
    ForwardSlash,
    Equal,
    BackSlash,
    A,
    O,
    E,
    U,
    I,
    D,
    H,
    T,
    N,
    S,
    Dash,
    Semicolon,
    Q,
    J,
    K,
    X,
    B,
    M,
    W,
    V,
    Z,
}

fn key_code_to_key(code: KeyCode) -> Option<Key> {
    if let KeyCode::Char(c) = code {
        let key = match c {
            '`' | '~' => Key::BackTick,
            '1' | '!' => Key::One,
            '2' | '@' => Key::Two,
            '3' | '#' => Key::Three,
            '4' | '$' => Key::Four,
            '5' | '%' => Key::Five,
            '6' | '^' => Key::Six,
            '7' | '&' => Key::Seven,
            '8' | '*' => Key::Eight,
            '9' | '(' => Key::Nine,
            '0' | ')' => Key::Zero,
            '[' | '{' => Key::OpenBracket,
            ']' | '}' => Key::CloseBracket,
            '\'' | '"' => Key::Quote,
            ',' | '<' => Key::Comma,
            '.' | '>' => Key::Period,
            'p' | 'P' => Key::P,
            'y' | 'Y' => Key::Y,
            'f' | 'F' => Key::F,
            'g' | 'G' => Key::G,
            'c' | 'C' => Key::C,
            'r' | 'R' => Key::R,
            'l' | 'L' => Key::L,
            '/' | '?' => Key::ForwardSlash,
            '=' | '+' => Key::Equal,
            '\\' | '|' => Key::BackSlash,
            'a' | 'A' => Key::A,
            'o' | 'O' => Key::O,
            'e' | 'E' => Key::E,
            'u' | 'U' => Key::U,
            'i' | 'I' => Key::I,
            'd' | 'D' => Key::D,
            'h' | 'H' => Key::H,
            't' | 'T' => Key::T,
            'n' | 'N' => Key::N,
            's' | 'S' => Key::S,
            '-' | '_' => Key::Dash,
            ';' | ':' => Key::Semicolon,
            'q' | 'Q' => Key::Q,
            'j' | 'J' => Key::J,
            'k' | 'K' => Key::K,
            'x' | 'X' => Key::X,
            'b' | 'B' => Key::B,
            'm' | 'M' => Key::M,
            'w' | 'W' => Key::W,
            'v' | 'V' => Key::V,
            'z' | 'Z' => Key::Z,
            _ => unreachable!(),
        };

        Some(key)
    } else {
        None
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Self::BackTick => "`~",
            Self::One => "1!",
            Self::Two => "2@",
            Self::Three => "3#",
            Self::Four => "4$",
            Self::Five => "5%",
            Self::Six => "6^",
            Self::Seven => "7&",
            Self::Eight => "8*",
            Self::Nine => "9(",
            Self::Zero => "0)",
            Self::OpenBracket => "[{",
            Self::CloseBracket => "]}",
            Self::Quote => "'\"",
            Self::Comma => ",<",
            Self::Period => ".>",
            Self::P => "p",
            Self::Y => "y",
            Self::F => "f",
            Self::G => "g",
            Self::C => "c",
            Self::R => "r",
            Self::L => "l",
            Self::ForwardSlash => "\\|",
            Self::Equal => "=+",
            Self::BackSlash => "/?",
            Self::A => "a",
            Self::O => "o",
            Self::E => "e",
            Self::U => "u",
            Self::I => "i",
            Self::D => "d",
            Self::H => "h",
            Self::T => "t",
            Self::N => "n",
            Self::S => "s",
            Self::Dash => "-_",
            Self::Semicolon => ";:",
            Self::Q => "q",
            Self::J => "j",
            Self::K => "k",
            Self::X => "x",
            Self::B => "b",
            Self::M => "m",
            Self::W => "w",
            Self::V => "v",
            Self::Z => "z",
        };

        write!(f, "{:^4}", c)
    }
}

struct Keyboard {
    keys: Vec<Key>,
    numberrow_cnt: usize,
    toprow_cnt: usize,
    homerow_cnt: usize,
    _bottomrow_cnt: usize,
    pressed: Option<Key>,
}

impl Default for Keyboard {
    fn default() -> Self {
        use Key::*;

        Self {
            keys: vec![
                BackTick,
                One,
                Two,
                Three,
                Four,
                Five,
                Six,
                Seven,
                Eight,
                Nine,
                Zero,
                OpenBracket,
                CloseBracket,
                Quote,
                Comma,
                Period,
                P,
                Y,
                F,
                G,
                C,
                R,
                L,
                ForwardSlash,
                Equal,
                BackSlash,
                A,
                O,
                E,
                U,
                I,
                D,
                H,
                T,
                N,
                S,
                Dash,
                Semicolon,
                Q,
                J,
                K,
                X,
                B,
                M,
                W,
                V,
                Z,
            ],
            numberrow_cnt: 13,
            toprow_cnt: 13,
            homerow_cnt: 11,
            _bottomrow_cnt: 10,
            pressed: None,
        }
    }
}

impl Keyboard {
    fn numberrow_idx(&self) -> usize {
        0
    }

    fn toprow_idx(&self) -> usize {
        self.numberrow_idx() + self.numberrow_cnt
    }

    fn homerow_idx(&self) -> usize {
        self.toprow_idx() + self.toprow_cnt
    }

    fn bottomrow_idx(&self) -> usize {
        self.homerow_idx() + self.homerow_cnt
    }

    fn key_pressed(&mut self, key: Option<Key>) {
        self.pressed = key;
    }
}

impl fmt::Display for Keyboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self
            .to_styled()
            .iter()
            .map(|s| s.content().to_string())
            .collect::<Vec<String>>()
            .join("");
        s.fmt(f)
    }
}

impl Keyboard {
    fn to_styled(&self) -> Vec<StyledContent<String>> {
        let mut styled = vec![];

        for key in &self.keys {
            let mut styled_key = style(key.to_string());
            if let Some(pressed) = self.pressed {
                if *key == pressed {
                    styled_key = styled_key.red();
                }
            };
            styled.push(styled_key);
        }

        // NOTE: These are in reverse order so the indices don't interact in weird ways
        styled.insert(self.bottomrow_idx(), style("\r\n          ".to_string()));
        styled.insert(self.homerow_idx(), style("\r\n        ".to_string()));
        styled.insert(self.toprow_idx(), style("\r\n      ".to_string()));

        styled
    }
}

#[throws(ErrorKind)]
fn print_keyboard(keyboard: &Keyboard) {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0),)?;
    for sc in keyboard.to_styled() {
        stdout().execute(PrintStyledContent(sc))?;
    }
}

