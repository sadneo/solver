use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self.kind {
            ErrorKind::InvalidToken => "Invalid token", // token, where
            ErrorKind::TooManyLeftParen => "Too many left parentheses", // where
            ErrorKind::TooManyRightParen => "Too many right parentheses", // where
            ErrorKind::UnexpectedToken => "Unexpected token", // token
            ErrorKind::DivisionByZero => "Division by zero", // where
        };
        write!(f, "{}", message)
    }
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorKind {
    InvalidToken,
    TooManyLeftParen,
    TooManyRightParen,
    UnexpectedToken,
    DivisionByZero,
}
