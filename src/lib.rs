#[derive(Debug, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
}

fn tokenize(expression: &str) -> anyhow::Result<Vec<Token>> {
    let mut tokens: Vec<Token> = vec![];

    let mut iterator = expression.chars().peekable();

    while let Some(char) = iterator.peek() {
        match char {
            '0'..='9' | '.' => {
                let mut buffer = char.to_string();
                iterator.next();

                while let Some(new) = iterator.peek() {
                    if !matches!(new, '0'..='9' | '.') {
                        break;
                    }
                    buffer.push(*new);
                    iterator.next();
                }

                let number = buffer.parse::<f64>().unwrap();
                tokens.push(Token::Number(number));
            },
            '+' => {
                tokens.push(Token::Plus);
                iterator.next();
            },
            '-' => {
                tokens.push(Token::Minus);
                iterator.next();
            },
            '*' => {
                tokens.push(Token::Multiply);
                iterator.next();
            },
            '/' => {
                tokens.push(Token::Divide);
                iterator.next();
            },
            '(' => {
                tokens.push(Token::LeftParen);
                iterator.next();
            },
            ')' => {
                tokens.push(Token::RightParen);
                iterator.next();
            },
            ' ' => {
                iterator.next();
            },
            _ => return Err(anyhow::Error::msg("unknown character in expression")),
        }
    }

    Ok(tokens)
}

fn match_parenthesis(tokens: &[Token]) -> Option<bool> {
    let mut balancer = 0;
    for token in tokens {
        if let Token::LeftParen = token {
            balancer += 1;
        } else if let Token::RightParen = token {
            balancer -= 1;
        }
        if balancer < 0 {
            return Some(false);
        }
    }

    if balancer > 0 {
        Some(true)
    } else {
        None
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
    let mut product = parse_primary(tokens, pos);

    while *pos < tokens.len() && (tokens[*pos] == Token::Multiply || tokens[*pos] == Token::Divide) {
        let operator = &tokens[*pos];
        *pos += 1;
        let primary = parse_factor(tokens, pos);

        match operator {
            Token::Multiply => product *= primary,
            Token::Divide => product /= primary,
            _ => unreachable!(),
        }
    }

    product
}

fn parse_primary(tokens: &[Token], pos: &mut usize) -> f64 {
    if let Token::Number(number) = tokens[*pos] {
        *pos += 1;

        number
    } else if let Token::LeftParen = tokens[*pos] {
        *pos += 1;
        let primary = parse_expr(tokens, pos);
        assert!(tokens[*pos] == Token::RightParen, "Expected right paren at {}, found {:?}", pos, tokens[*pos]);
        *pos += 1;

        primary
    } else {
        panic!("Expected number or '(' at {}, found {:?}", pos, tokens[*pos])
    }
}

pub fn evaluate(expression: &str) -> anyhow::Result<f64> {
    let tokens = tokenize(expression)?;
    if let Some(too_many_left) = match_parenthesis(&tokens) {
        let message = if too_many_left {
            "Too many left parenthesis"
        } else {
            "Too many right parenthesis"
        };
        return Err(anyhow::Error::msg(message));
    }

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
            Token::Plus,
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
    fn match_parenthesis_works() {
        let tokens = vec![Token::LeftParen, Token::RightParen];
        assert_eq!(match_parenthesis(&tokens), None);

        let tokens = vec![Token::LeftParen, Token::LeftParen, Token::RightParen];
        assert_eq!(match_parenthesis(&tokens), Some(true));

        let tokens = vec![Token::LeftParen, Token::RightParen, Token::RightParen];
        assert_eq!(match_parenthesis(&tokens), Some(false));
    }

    #[test]
    fn imply_multiplication_works() {
        let tokens = vec![Token::LeftParen, Token::RightParen, Token::LeftParen, Token::RightParen];
        let expected_result = vec![Token::LeftParen, Token::RightParen, Token::Multiply, Token::LeftParen, Token::RightParen];
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
