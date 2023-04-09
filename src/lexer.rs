use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Remainder,
    Power,
    Factorial,
    Negative,
}

#[derive(Debug, PartialEq)]
pub enum Group {
    Parenthesis,
    Bracket,
    Brace,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Number(f64),
    Unary(Operator),
    Binary(Operator),
    Left(Group),
    Right(Group),
}

fn lex_number<Iter: Iterator<Item = char>>(char: char, iterator: &mut Peekable<Iter>) -> f64 {
    let mut buffer = char.to_string();
    iterator.next();

    while let Some(char) = iterator.peek() {
        let new = char.to_string();
        match char {
            '0'..='9' | '.' => buffer += &new,
            _ => break,
        }
        iterator.next();
    }

    buffer.parse::<f64>().unwrap_or_default()
}

pub fn lex(expression: &String) -> anyhow::Result<Vec<Token>> {
    let mut result: Vec<Token> = vec![];

    let mut iterator = expression.chars().peekable();
    while let Some(&char) = iterator.peek() {
        match char {
            ' ' => {
                iterator.next();
            }
            '0'..='9' | '.' => {
                let number = lex_number(char, &mut iterator);
                result.push(Token::Number(number));
            }

            '+' => {
                result.push(Token::Binary(Operator::Plus));
                iterator.next();
            }
            '-' => {
                result.push(Token::Binary(Operator::Minus));
                iterator.next();
            }
            '*' => {
                result.push(Token::Binary(Operator::Multiply));
                iterator.next();
            }
            '/' => {
                result.push(Token::Binary(Operator::Divide));
                iterator.next();
            }
            '%' => {
                result.push(Token::Binary(Operator::Remainder));
                iterator.next();
            }
            '^' => {
                result.push(Token::Binary(Operator::Power));
                iterator.next();
            }
            '!' => {
                result.push(Token::Unary(Operator::Factorial));
                iterator.next();
            }

            '{' | '[' | '(' => {
                // i'm just too lazy to make all the match arms rn
                result.push(Token::Left(Group::Parenthesis));
                iterator.next();
            }
            '}' | ']' | ')' => {
                result.push(Token::Right(Group::Parenthesis));
                iterator.next();
            }

            _ => return Err(anyhow::Error::msg("unknown character in expression")),
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_vec(a: &[Token], b: &[Token]) -> bool {
        (a.len() == b.len()) && a.iter().zip(b).all(|(x, y)| x == y)
    }

    #[test]
    fn partialeq_works() {
        assert_eq!(Token::Number(5.3), Token::Number(5.3),);
    }
    #[test]
    fn compare_vec_works() {
        let a = vec![
            Token::Number(5.3),
            Token::Number(0.11),
            Token::Number(330.0),
        ];
        let b = vec![
            Token::Number(5.3),
            Token::Number(0.11),
            Token::Number(330.0),
        ];
        assert!(compare_vec(&a, &b),);
    }
    #[test]
    fn lex_works() {
        let expression = String::from("(53+110)");
        let equal_to = vec![
            Token::Left(Group::Parenthesis),
            Token::Number(53.0),
            Token::Binary(Operator::Plus),
            Token::Number(110.0),
            Token::Right(Group::Parenthesis),
        ];
        let result = lex(&expression).unwrap();
        assert!(compare_vec(&result, &equal_to));
    }
    #[test]
    fn lex_decimal() {
        let expression = String::from("5.3 .110 333.");
        let equal_to = vec![
            Token::Number(5.3),
            Token::Number(0.11),
            Token::Number(333.0),
        ];
        let result = lex(&expression).unwrap();
        assert!(compare_vec(&result, &equal_to));
    }
}
