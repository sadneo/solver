use crate::error::{Error, ErrorKind, Result};

pub mod error;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    LeftParen,
    RightParen,
}

fn tokenize(expression: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = vec![];

    let mut iterator = expression.chars().enumerate().peekable();

    while let Some((index, char)) = iterator.peek() {
        match char {
            '0'..='9' | '.' => {
                let mut buffer = char.to_string();
                iterator.next();

                while let Some((_, new)) = iterator.peek() {
                    if !matches!(new, '0'..='9' | '.') {
                        break;
                    }
                    buffer.push(*new);
                    iterator.next();
                }

                let number = buffer.parse::<f64>().unwrap();
                tokens.push(Token::Number(number));
                continue;
            }
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Multiply),
            '/' => tokens.push(Token::Divide),
            '%' => tokens.push(Token::Modulo),
            '^' => tokens.push(Token::Exponent),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            ' ' => {},
            _ => return Err(Error::from_expression(ErrorKind::InvalidToken, expression.to_owned(), *index)),
        }
        iterator.next();
    }

    Ok(tokens)
}

fn match_parentheses(tokens: &[Token]) -> Result<()> {
    let mut balancer = 0;
    for (index, token) in tokens.iter().enumerate() {
        if let Token::LeftParen = token {
            balancer += 1;
        } else if let Token::RightParen = token {
            balancer -= 1;
        }
        if balancer < 0 {
            return Err(Error::new(ErrorKind::TooManyRightParen, tokens.to_vec(), index));
        }
    }

    if balancer > 0 {
        Err(Error::new(ErrorKind::TooManyLeftParen, tokens.to_vec(), tokens.len() - 1))
    } else {
        Ok(())
    }
}

fn imply_multiplication(mut tokens: Vec<Token>) -> Vec<Token> {
    for index in 0..tokens.len() - 1 {
        let token = &tokens[index];
        let next_token = &tokens[index + 1];
        if *token == Token::RightParen && *next_token == Token::LeftParen {
            tokens.insert(index + 1, Token::Multiply);
        }
    }

    tokens
}

fn parse_expr(tokens: &[Token], pos: &mut usize) -> f64 {
    parse_term(tokens, pos)
}

fn parse_term(tokens: &[Token], pos: &mut usize) -> f64 {
    let mut sum = parse_factor(tokens, pos);

    while *pos < tokens.len() && (tokens[*pos] == Token::Plus || tokens[*pos] == Token::Minus) {
        let operator = &tokens[*pos];
        *pos += 1;
        let factor = parse_factor(tokens, pos);

        match operator {
            Token::Plus => sum += factor,
            Token::Minus => sum -= factor,
            _ => unreachable!(),
        }
    }

    sum
}

fn parse_factor(tokens: &[Token], pos: &mut usize) -> f64 {
    let mut product = parse_exponent(tokens, pos);

    while *pos < tokens.len() && (tokens[*pos] == Token::Multiply || tokens[*pos] == Token::Divide || tokens[*pos] == Token::Modulo)
    {
        let operator = &tokens[*pos];
        *pos += 1;
        let power = parse_exponent(tokens, pos);

        match operator {
            Token::Multiply => product *= power,
            Token::Divide => product /= power,
            Token::Modulo => product %= power,
            _ => unreachable!(),
        }
    }

    product
}

fn parse_exponent(tokens: &[Token], pos: &mut usize) -> f64 {
    let mut power = parse_primary(tokens, pos);

    while *pos < tokens.len() && tokens[*pos] == Token::Exponent
    {
        let operator = &tokens[*pos];
        *pos += 1;
        let primary = parse_primary(tokens, pos);

        match operator {
            Token::Exponent => power = f64::powf(power, primary),
            _ => unreachable!(),
        }
    }

    power
}

fn parse_primary(tokens: &[Token], pos: &mut usize) -> f64 {
    if let Token::Number(number) = tokens[*pos] {
        *pos += 1;

        number
    } else if let Token::LeftParen = tokens[*pos] {
        *pos += 1;
        let primary = parse_expr(tokens, pos);
        assert!(
            tokens[*pos] == Token::RightParen,
            "Expected right paren at {}, found {:?}",
            pos,
            tokens[*pos]
        );
        *pos += 1;

        primary
    } else {
        panic!(
            "Expected number or '(' at {}, found {:?}",
            pos, tokens[*pos]
        )
    }
}

pub fn evaluate(expression: &str) -> Result<f64> {
    let tokens = tokenize(expression)?;
    match_parentheses(&tokens)?;

    let tokens = imply_multiplication(tokens);
    Ok(parse_expr(&tokens, &mut 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_vec(a: &[Token], b: &[Token]) -> bool {
        (a.len() == b.len()) && a.iter().zip(b).all(|(x, y)| x == y)
    }

    #[test]
    fn tokenize_works() {
        let expression = String::from("53+110");
        let equal_to = vec![Token::Number(53.0), Token::Plus, Token::Number(110.0)];
        let result = tokenize(&expression).unwrap();
        assert!(compare_vec(&result, &equal_to));
    }

    #[test]
    fn tokenize_decimal() {
        let expression = String::from("5.3 .110 333.");
        let equal_to = vec![
            Token::Number(5.3),
            Token::Number(0.11),
            Token::Number(333.0),
        ];
        let result = tokenize(&expression).unwrap();
        assert!(compare_vec(&result, &equal_to));
    }

    #[test]
    fn match_parentheses_works() {
        let tokens = vec![Token::LeftParen, Token::RightParen];
        assert!(match_parentheses(&tokens).is_ok());

        let tokens = vec![Token::LeftParen, Token::LeftParen, Token::RightParen];
        let Err(error) = match_parentheses(&tokens) else {
            panic!();
        };
        assert_eq!(error.kind(), ErrorKind::TooManyLeftParen);

        let tokens = vec![Token::LeftParen, Token::RightParen, Token::RightParen];
        let Err(error) = match_parentheses(&tokens) else {
            panic!();
        };
        assert_eq!(error.kind(), ErrorKind::TooManyRightParen);
    }

    #[test]
    fn imply_multiplication_works() {
        let tokens = vec![
            Token::LeftParen,
            Token::RightParen,
            Token::LeftParen,
            Token::RightParen,
        ];
        let expected_result = vec![
            Token::LeftParen,
            Token::RightParen,
            Token::Multiply,
            Token::LeftParen,
            Token::RightParen,
        ];
        assert_eq!(imply_multiplication(tokens), expected_result);
    }

    #[test]
    fn parsing_works() {
        assert_eq!(evaluate("(30)(2)").unwrap(), 60.0);
        assert_eq!(evaluate("30 + 13 * 3").unwrap(), 69.0);
        assert_eq!(evaluate("30 + (8 + 5) * 3").unwrap(), 69.0);
        assert_eq!(evaluate("10 / 2").unwrap(), 5.0);
        assert_eq!(evaluate("(4 + 6) * 2 - 5").unwrap(), 15.0);
        assert_eq!(evaluate("2 * (3 + 4) / 2").unwrap(), 7.0);
        assert_eq!(evaluate("1 + 2 + 3 + 4 + 5").unwrap(), 15.0);
        assert_eq!(evaluate("(5 - 2) * (12 / 2)").unwrap(), 18.0);
    }
}
