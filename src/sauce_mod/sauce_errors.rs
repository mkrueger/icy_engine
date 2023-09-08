use std::error::Error;

use super::SAUCE_LEN;

#[derive(Debug, Clone)]
pub enum SauceError {
    FileTooShort,
    NoSauce,
    UnsupportedSauceVersion(String),
    InvalidCommentBlock,
    InvalidCommentId(String),
    UnsupportedSauceDate(String),
    CommentLimitExceeded(usize),
    BinFileWidthLimitExceeded(i32),
}

impl std::fmt::Display for SauceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SauceError::FileTooShort => write!(f, "file needs to be at least {SAUCE_LEN} bytes"),
            SauceError::NoSauce => write!(f, "no sauce found."),
            SauceError::UnsupportedSauceVersion(ver) => write!(f, "unsupported version {ver}"),
            SauceError::InvalidCommentBlock => write!(f, "invalid sauce comment block"),
            SauceError::InvalidCommentId(id) => write!(f, "invalid sauce comment id {id}"),
            SauceError::UnsupportedSauceDate(err) => {
                write!(f, "unsupported sauce date format: {err}")
            }
            SauceError::CommentLimitExceeded(limit) => {
                write!(f, "comment limit exceeded (maximum of 255): {limit}")
            }
            SauceError::BinFileWidthLimitExceeded(limit) => {
                write!(f, "bin file width limit exceeded (maximum of 512): {limit}")
            }
        }
    }
}

impl Error for SauceError {
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
