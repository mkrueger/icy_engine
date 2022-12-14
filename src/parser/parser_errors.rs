use std::error::Error;

#[derive(Debug, Clone)]
pub enum ParserError {
    InvalidChar(char),
    InvalidBuffer,
    UnsupportedEscapeSequence(String),
    UnsupportedCustomCommand(i32),
    Description(&'static str),
    UnsupportedControlCode(u32),
    UnsupportedFont(i32),
    UnexpectedSixelEnd(char),
    InvalidColorInSixelSequence,
    NumberMissingInSixelRepeat,
    InvalidSixelChar(char),
    UnsupportedSixelColorformat(i32),
    ErrorInSixelEngine(&'static str),
    InvalidPictureSize,

    InvalidRipAnsiQuery(i32),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::InvalidChar(ch) => write!(f, "invalid character {}", ch),
            ParserError::UnsupportedEscapeSequence(seq) => {
                write!(f, "unsupported escape sequence {}", seq)
            }
            ParserError::Description(str) => write!(f, "{}", str),
            ParserError::UnsupportedControlCode(code) => {
                write!(f, "unsupported control code {}", *code)
            }
            ParserError::UnsupportedCustomCommand(code) => {
                write!(f, "unsupported custom ansi command: {}", *code)
            }
            ParserError::UnsupportedFont(code) => write!(f, "font {} not supported", *code),
            ParserError::UnexpectedSixelEnd(ch) => {
                write!(f, "sixel sequence ended with <esc>{} expected '\\'", ch)
            }
            ParserError::InvalidBuffer => write!(f, "output buffer is invalid"),
            ParserError::InvalidColorInSixelSequence => {
                write!(f, "invalid color in sixel sequence")
            }
            ParserError::NumberMissingInSixelRepeat => {
                write!(f, "sixel repeat sequence is missing number")
            }
            ParserError::InvalidSixelChar(ch) => write!(f, "{} invalid in sixel data", ch),
            ParserError::UnsupportedSixelColorformat(i) => {
                write!(f, "{} invalid color format in sixel data", i)
            }
            ParserError::ErrorInSixelEngine(err) => write!(f, "sixel engine error: {}", err),
            ParserError::InvalidPictureSize => write!(f, "invalid sixel picture size description"),
            ParserError::InvalidRipAnsiQuery(i) => write!(f, "invalid rip ansi query <esc>[{}!", i),
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
