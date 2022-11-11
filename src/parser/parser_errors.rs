use std::error::Error;

#[derive(Debug, Clone)]
pub enum ParserError {
    InvalidChar(char),
    InvalidBuffer,
    UnsupportedEscapeSequence(String),
    UnsupportedCustomCommand(i32),
    Description(&'static str),
    UnsupportedControlCode(u32),
    UnsupportedFont(i32)
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::InvalidChar(ch) => write!(f, "invalid character {}", ch),
            ParserError::UnsupportedEscapeSequence(seq) => write!(f, "unsupported escape sequence {}", seq),
            ParserError::Description(str) => write!(f, "{}", str),
            ParserError::UnsupportedControlCode(code) => write!(f, "unsupported control code {}", *code),
            ParserError::UnsupportedCustomCommand(code) => write!(f, "unsupported custom ansi command: {}", *code),
            ParserError::UnsupportedFont(code) => write!(f, "font {} not supported", *code),
            ParserError::InvalidBuffer => write!(f, "output buffer is invalid"),
        }
    }
}

impl Error for ParserError {
    fn description(&self) -> &str {
        "use std::display"
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
