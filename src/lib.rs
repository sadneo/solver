use crate::error::{Error, ErrorKind, Result};

pub mod error;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Binary(Binary),
    Unary(Unary),
    Number(f64),
    LeftParen,
    RightParen,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Binary {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    ImplicitMultiply,
    Exponent,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Unary {
    Negative,
    Factorial(u64),
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
            '+' => tokens.push(Token::Binary(Binary::Plus)),
            '-' => match tokens.last() {
                Some(Token::Number(_)) | Some(Token::RightParen) => {
                    tokens.push(Token::Binary(Binary::Minus))
                }
                _ => tokens.push(Token::Unary(Unary::Negative)),
            },
            '*' => tokens.push(Token::Binary(Binary::Multiply)),
            '/' => tokens.push(Token::Binary(Binary::Divide)),
            '%' => tokens.push(Token::Binary(Binary::Modulo)),
            '!' => {
                let mut n = 1;
                iterator.next();

                while let Some((_, '!')) = iterator.peek() {
                    n += 1;
                    iterator.next();
                }
                tokens.push(Token::Unary(Unary::Factorial(n)));
                continue;
            }
            '^' => tokens.push(Token::Binary(Binary::Exponent)),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            ' ' => {}
            _ => {
                return Err(Error::from_expression(
                    ErrorKind::InvalidToken,
                    expression.to_owned(),
                    *index,
                ))
            }
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
            return Err(Error::new(
                ErrorKind::TooManyRightParen,
                tokens.to_vec(),
                index,
            ));
        }
    }

    if balancer > 0 {
        Err(Error::new(
            ErrorKind::TooManyLeftParen,
            tokens.to_vec(),
            tokens.len() - 1,
        ))
    } else {
        Ok(())
    }
}

fn imply_multiplication(mut tokens: Vec<Token>) -> Vec<Token> {
    for index in 0..tokens.len() - 1 {
        let token = &tokens[index];
        let next_token = &tokens[index + 1];

        #[allow(clippy::nonminimal_bool)]
        if (*token == Token::RightParen && *next_token == Token::LeftParen)
            || (matches!(token, Token::Number(_)) && *next_token == Token::LeftParen)
            || (matches!(next_token, Token::Number(_)) && *token == Token::RightParen)
        {
            tokens.insert(index + 1, Token::Binary(Binary::ImplicitMultiply));
        }
    }

    tokens
}

fn parse_expr(tokens: &[Token], pos: &mut usize) -> f64 {
    parse_term(tokens, pos)
}

fn parse_term(tokens: &[Token], pos: &mut usize) -> f64 {
    let mut sum = parse_factor(tokens, pos);

    while *pos < tokens.len()
        && (tokens[*pos] == Token::Binary(Binary::Plus)
            || tokens[*pos] == Token::Binary(Binary::Minus))
    {
        let operator = &tokens[*pos];
        *pos += 1;
        let factor = parse_factor(tokens, pos);

        match operator {
            Token::Binary(Binary::Plus) => sum += factor,
            Token::Binary(Binary::Minus) => sum -= factor,
            _ => unreachable!(),
        }
    }

    sum
}

fn parse_factor(tokens: &[Token], pos: &mut usize) -> f64 {
    let mut product = parse_factorial(tokens, pos);

    while *pos < tokens.len()
        && (tokens[*pos] == Token::Binary(Binary::Multiply)
            || tokens[*pos] == Token::Binary(Binary::Divide)
            || tokens[*pos] == Token::Binary(Binary::Modulo))
    {
        let operator = &tokens[*pos];
        *pos += 1;
        let factorial = parse_factorial(tokens, pos);

        match operator {
            Token::Binary(Binary::Multiply) => product *= factorial,
            Token::Binary(Binary::Divide) => product /= factorial,
            Token::Binary(Binary::Modulo) => product %= factorial,
            _ => unreachable!(),
        }
    }

    product
}

fn parse_factorial(tokens: &[Token], pos: &mut usize) -> f64 {
    let implicit_product = parse_implicit_product(tokens, pos);

    if *pos < tokens.len() {
        let Token::Unary(Unary::Factorial(n)) = tokens[*pos] else {
            return implicit_product;
        };
        let mut next_factor = implicit_product as u64 - n;
        let mut factorial = implicit_product as u64;

        while next_factor > 1 {
            factorial *= next_factor;
            next_factor -= n;
        }
        *pos += 1;
        factorial as f64
    } else {
        implicit_product
    }
}

fn parse_implicit_product(tokens: &[Token], pos: &mut usize) -> f64 {
    let mut product = parse_exponent(tokens, pos);

    while *pos < tokens.len() && tokens[*pos] == Token::Binary(Binary::ImplicitMultiply) {
        let operator = &tokens[*pos];
        *pos += 1;
        let power = parse_exponent(tokens, pos);

        match operator {
            Token::Binary(Binary::ImplicitMultiply) => product *= power,
            _ => unreachable!(),
        }
    }

    product
}

fn parse_exponent(tokens: &[Token], pos: &mut usize) -> f64 {
    let mut power = parse_negative(tokens, pos);

    while *pos < tokens.len() && tokens[*pos] == Token::Binary(Binary::Exponent) {
        *pos += 1;
        let negative = parse_negative(tokens, pos);

        power = f64::powf(power, negative);
    }

    power
}

fn parse_negative(tokens: &[Token], pos: &mut usize) -> f64 {
    if let Token::Unary(Unary::Negative) = tokens[*pos] {
        *pos += 1;
        -parse_primary(tokens, pos)
    } else {
        parse_primary(tokens, pos)
    }
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
        let equal_to = vec![
            Token::Number(53.0),
            Token::Binary(Binary::Plus),
            Token::Number(110.0),
        ];
        let result = tokenize(&expression).unwrap();
        assert!(compare_vec(&result, &equal_to));

        let expression = String::from("6 * ((20 - 4) / 2 + 8) / 6");
        let equal_to = vec![
            Token::Number(6.0),
            Token::Binary(Binary::Multiply),
            Token::LeftParen,
            Token::LeftParen,
            Token::Number(20.0),
            Token::Binary(Binary::Minus),
            Token::Number(4.0),
            Token::RightParen,
            Token::Binary(Binary::Divide),
            Token::Number(2.0),
            Token::Binary(Binary::Plus),
            Token::Number(8.0),
            Token::RightParen,
            Token::Binary(Binary::Divide),
            Token::Number(6.0),
        ];
        let result = tokenize(&expression).unwrap();
        assert!(compare_vec(&result, &equal_to));
    }

    #[test]
    fn tokenize_negative() {
        let expression = String::from("-53-110");
        let equal_to = vec![
            Token::Unary(Unary::Negative),
            Token::Number(53.0),
            Token::Binary(Binary::Minus),
            Token::Number(110.0),
        ];
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
            Token::Binary(Binary::ImplicitMultiply),
            Token::LeftParen,
            Token::RightParen,
        ];
        assert_eq!(imply_multiplication(tokens), expected_result);

        let tokens = vec![Token::LeftParen, Token::RightParen, Token::Number(3.0)];
        let expected_result = vec![
            Token::LeftParen,
            Token::RightParen,
            Token::Binary(Binary::ImplicitMultiply),
            Token::Number(3.0),
        ];
        assert_eq!(imply_multiplication(tokens), expected_result);

        let tokens = vec![Token::Number(3.0), Token::LeftParen, Token::RightParen];
        let expected_result = vec![
            Token::Number(3.0),
            Token::Binary(Binary::ImplicitMultiply),
            Token::LeftParen,
            Token::RightParen,
        ];
        assert_eq!(imply_multiplication(tokens), expected_result);
    }

    #[test]
    fn parsing_works() {
        assert_eq!(evaluate("6 * ((20 - 4) / 2 + 8) / 6").unwrap(), 16.0);
        assert_eq!(evaluate("11 % 4").unwrap(), 3.0);
        assert_eq!(evaluate("11 % (9)(5)").unwrap(), 11.0);
        assert_eq!(evaluate("30 + 13 * 3").unwrap(), 69.0);
        assert_eq!(evaluate("30 + (8 + 5) * 3").unwrap(), 69.0);
        assert_eq!(evaluate("10 / 2").unwrap(), 5.0);
        assert_eq!(evaluate("(4 + 6) * 2 - 5").unwrap(), 15.0);
        assert_eq!(evaluate("2 * (3 + 4) / 2").unwrap(), 7.0);
        assert_eq!(evaluate("1 + 2 + 3 + 4 + 5").unwrap(), 15.0);
        assert_eq!(evaluate("(5 - 2) * (12 / 2)").unwrap(), 18.0);
        assert_eq!(evaluate("9!").unwrap(), 362880.0);
        assert_eq!(evaluate("(3!)!").unwrap(), 720.0);
        assert_eq!(evaluate("(3!)^2").unwrap(), 36.0);
    }
}
