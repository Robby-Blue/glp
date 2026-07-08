use std::char;

use crate::{CustomParser, ParseContext, Stream, errors::ParseError};

#[derive(Clone)]
pub struct Token<T: CustomParser> {
    pub token_type: TokenType<T>,
    start: usize,
    end: usize,
}

impl<T: CustomParser> Token<T> {
    pub fn new(token_type: TokenType<T>, start: usize, end: usize) -> Self {
        Token {
            token_type,
            start,
            end,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenType<T: CustomParser> {
    Keyword(Keyword),
    Identifier(String),
    OpeningParenthesis,
    ClosingParenthesis,
    OpeningCurly,
    ClosingCurly,
    Operator(Operator),
    Equals,
    NumberLiteral(f64),
    StringLiteral(String),
    Comma,
    Custom(T::Token),
}

#[derive(Debug, Clone)]
pub enum Keyword {
    Def,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

pub(crate) fn tokenize<T: CustomParser>(
    src: &str,
    ctx: &ParseContext<T>,
) -> Result<Vec<Token<T>>, ParseError> {
    let mut stream = Stream::new(src.chars().collect());
    let mut tokens = vec![];

    while stream.has_more() {
        let c = stream.peek()?;

        if c.is_whitespace() {
            stream.read()?;
            continue;
        }

        let start = stream.index;
        let token_type = read_token(&mut stream, ctx)?;
        let end = stream.index;

        let token = Token::new(token_type, start, end);
        tokens.push(token);
    }

    Ok(tokens)
}

fn read_token<T: CustomParser>(
    stream: &mut Stream<char>,
    ctx: &ParseContext<T>,
) -> Result<TokenType<T>, ParseError> {
    let c = stream.read()?;
    if is_identifier_first_char(&c) {
        Ok(read_identifier(stream, ctx)?)
    } else if c.is_numeric() {
        Ok(read_number(stream)?)
    } else {
        let token = match c {
            '(' => TokenType::OpeningParenthesis,
            ')' => TokenType::ClosingParenthesis,
            '{' => TokenType::OpeningCurly,
            '}' => TokenType::ClosingCurly,
            '\"' => read_string_literal(stream)?,
            '=' => TokenType::Equals,
            ',' => TokenType::Comma,
            '+' => TokenType::Operator(Operator::Addition),
            '-' => TokenType::Operator(Operator::Subtraction),
            '*' => TokenType::Operator(Operator::Multiplication),
            '/' => TokenType::Operator(Operator::Division),
            _ => read_custom_token(stream, ctx)?,
        };
        Ok(token)
    }
}

fn read_custom_token<T: CustomParser>(
    stream: &mut Stream<char>,
    ctx: &ParseContext<T>,
) -> Result<TokenType<T>, ParseError> {
    stream.back();
    let token = ctx.custom_parser.parse_token(stream)?;
    match token {
        Some(token) => Ok(TokenType::Custom(token)),
        None => todo!("read token err"),
    }
}

fn read_identifier<T: CustomParser>(
    stream: &mut Stream<char>,
    ctx: &ParseContext<T>,
) -> Result<TokenType<T>, ParseError> {
    stream.back();

    let mut str = String::new();
    loop {
        let c = stream.read()?;
        if is_identifier_char(&c) {
            str.push(c.clone());
        } else {
            break;
        }
    }

    stream.back();

    if str == "def" {
        Ok(TokenType::Keyword(Keyword::Def))
    } else if ctx.custom_keywords.contains(&str) {
        Ok(TokenType::Keyword(Keyword::Custom(str.to_string())))
    } else {
        Ok(TokenType::Identifier(str))
    }
}

fn read_number<T: CustomParser>(stream: &mut Stream<char>) -> Result<TokenType<T>, ParseError> {
    stream.back();

    let mut str = String::new();
    loop {
        let c = stream.read()?;
        if c.is_numeric() {
            str.push(c.clone());
        } else {
            break;
        }
    }

    stream.back();

    let num = str.parse();
    match num {
        Ok(n) => Ok(TokenType::NumberLiteral(n)),
        Err(_) => Err(ParseError::new("num parsing")),
    }
}

fn read_string_literal<T: CustomParser>(
    stream: &mut Stream<char>,
) -> Result<TokenType<T>, ParseError> {
    let mut str = String::new();
    loop {
        let c = stream.read()?;
        if c == '\"' {
            break;
        } else {
            str.push(c.clone());
        }
    }

    Ok(TokenType::StringLiteral(str))
}

fn is_identifier_first_char(c: &char) -> bool {
    return is_identifier_char(c) && !c.is_numeric();
}

fn is_identifier_char(c: &char) -> bool {
    return c.is_alphabetic() || c.is_numeric() || "_".contains(c.clone());
}
