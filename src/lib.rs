use std::fs;

use crate::{errors::ParseError, parser::nodes::Definition, tokenizer::Token};

pub mod errors;
pub mod parser;
pub mod tokenizer;

pub fn parse_file<T: CustomParser>(
    path: &str,
    ctx: &ParseContext<T>,
) -> Result<Definition<T>, ParseError> {
    let contents = fs::read_to_string(path).expect("read file");
    parse_source(&contents, ctx)
}

pub fn parse_source<T: CustomParser>(
    src: &str,
    ctx: &ParseContext<T>,
) -> Result<Definition<T>, ParseError> {
    let tokens = tokenizer::tokenize(src, &ctx)?;
    let ast = parser::parser::parse(tokens, &ctx)?;
    Ok(ast.first().unwrap().clone())
}

pub struct ParseContext<T: CustomParser> {
    pub custom_keywords: Vec<String>,
    pub custom_parser: T,
}

impl<T: CustomParser> ParseContext<T> {
    pub fn new(parser: T) -> Self {
        ParseContext {
            custom_keywords: parser.get_custom_keywords(),
            custom_parser: parser,
        }
    }
}

pub trait CustomParser: Clone {
    type Token: Clone;
    type Value: Clone;
    type Unit: Clone;

    fn get_custom_keywords(&self) -> Vec<String>;

    fn parse_token(&self, stream: &mut Stream<char>) -> Result<Option<Self::Token>, ParseError>;
    fn parse_value(
        &self,
        stream: &mut Stream<Token<Self>>,
    ) -> Result<Option<Self::Value>, ParseError>;
    fn parse_unit(
        &self,
        stream: &mut Stream<Token<Self>>,
    ) -> Result<Option<Self::Unit>, ParseError>;
}

pub struct Stream<T> {
    data: Vec<T>,
    pub index: usize,
}

impl<T: Clone> Stream<T> {
    pub fn new(data: Vec<T>) -> Self {
        Stream {
            data: data,
            index: 0,
        }
    }
    pub fn read(&mut self) -> Result<T, ParseError> {
        self.index += 1;
        let val = self.data.get(self.index - 1);
        match val {
            Some(d) => Ok(d.clone()),
            None => Err(ParseError::new("ran out of data")),
        }
    }
    pub fn peek(&self) -> Result<T, ParseError> {
        let val = self.data.get(self.index);
        match val {
            Some(d) => Ok(d.clone()),
            None => Err(ParseError::new("no data")),
        }
    }
    pub fn peek_ahead(&self, delta: usize) -> Result<T, ParseError> {
        let val = self.data.get(self.index + delta);
        match val {
            Some(d) => Ok(d.clone()),
            None => Err(ParseError::new("no data")),
        }
    }
    pub fn back(&mut self) {
        self.index -= 1;
    }
    pub fn has_more(&self) -> bool {
        self.index < self.data.len()
    }
}
