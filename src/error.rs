use crate::{Token, Binary, Unary};
use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    expression: String,
    index: usize,
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self.kind {
            ErrorKind::InvalidToken => "Invalid token",
            ErrorKind::TooManyLeftParen => "Too many left parentheses",
            ErrorKind::TooManyRightParen => "Too many right parentheses",
            ErrorKind::UnexpectedToken => "Unexpected token",
            ErrorKind::DivisionByZero => "Division by zero",
        };

        let mut info: String;
        if message.len() <= self.index {
            let padding = self.index - message.len();
            info = " ".repeat(padding);
            info.push_str(message);
            info.push('^');
        } else {
            info = " ".repeat(self.index + 1);
            info.push('^');
            info.push_str(message);
        }

        write!(f, "{}\n{}", self.expression, info)
    }
}

impl Error {
    pub fn new(kind: ErrorKind, tokens: Vec<Token>, token_index: usize) -> Self {
        assert!((0..tokens.len()).contains(&token_index)); // TODO: safer way to do this

        let mut buffer = String::new();
        let mut string_index: Option<usize> = None;

        for (index, token) in tokens.iter().enumerate() {
            let string: String;
            if index == token_index {
                string_index = Some(buffer.len());
            }

            let token_str = match token {
                Token::Number(f64) => {
                    string = f64.to_string();
                    string.as_str()
                }
                Token::Binary(Binary::Plus) => "+",
                Token::Binary(Binary::Minus) => "-",
                Token::Binary(Binary::Multiply) => "*",
                Token::Binary(Binary::Divide) => "/",
                Token::Binary(Binary::Modulo) => "%",
                Token::Unary(Unary::Factorial(n)) => {
                    string = "!".repeat(*n as usize);
                    string.as_str()
                },
                Token::Binary(Binary::ImplicitMultiply) => "",
                Token::Binary(Binary::Exponent) => "^",
                Token::LeftParen => "(",
                Token::RightParen => ")",
                Token::Unary(Unary::Negative) => "-",
            };

            buffer.push_str(token_str);
        }

        if let Some(index) = string_index {
            Self {
                kind,
                expression: buffer,
                index,
            }
        } else {
            unreachable!()
        }
    }

    pub fn from_expression(kind: ErrorKind, expression: String, index: usize) -> Self {
        Self {
            kind,
            expression,
            index,
        }
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
