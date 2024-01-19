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
}
