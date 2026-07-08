use glp::CustomParser;
use glp::ParseContext;
use glp::Stream;
use glp::allow_type_data;
use glp::allow_type_data_read;
use glp::errors::ParseError;
use glp::expect_type_get_data;
use glp::tokenizer::Keyword;
use glp::tokenizer::Token;
use glp::tokenizer::TokenType;

fn main() {
    let custom = MyCustomParser {};
    let ctx = ParseContext::new(custom);

    let res = glp::parse_file("examples/data/example.lang", &ctx);

    match res {
        Ok(_) => {}
        Err(err) => err.print(),
    }
}

#[derive(Clone)]
enum CustomToken {
    Hex(u32),
    PercentSign,
}

#[derive(Clone)]
enum CustomValue {
    Hex(u32),
}

#[derive(Clone)]
enum CustomUnit {
    Px,
    Percent,
}

#[derive(Clone)]
struct MyCustomParser;

impl CustomParser for MyCustomParser {
    type Token = CustomToken;
    type Value = CustomValue;
    type Unit = CustomUnit;

    fn get_custom_keywords(&self) -> Vec<String> {
        vec!["px"].iter().map(|s| s.to_string()).collect()
    }

    fn parse_token(&self, stream: &mut Stream<char>) -> Result<Option<Self::Token>, ParseError> {
        let c = stream.read()?;

        match c {
            '#' => {
                let str = Self::read_hex_str(stream)?;

                let str = match str.len() {
                    6 => str + "FF",
                    8 => str,
                    _ => return Err(ParseError::new("parse hex")),
                };

                let n = i64::from_str_radix(&str, 16);
                let n = match n {
                    Ok(n) => n,
                    Err(_) => return Err(ParseError::new("parse hex")),
                };

                Ok(Some(CustomToken::Hex(n as u32)))
            }
            '%' => Ok(Some(CustomToken::PercentSign)),
            _ => Ok(None),
        }
    }

    fn parse_value(
        &self,
        stream: &mut Stream<Token<Self>>,
    ) -> Result<Option<CustomValue>, ParseError> {
        if !allow_type_data!(TokenType::Custom, stream) {
            return Ok(None);
        }

        let keyword = expect_type_get_data!(TokenType::Custom, stream);
        let val = match keyword {
            CustomToken::Hex(v) => CustomValue::Hex(v),
            _ => return Ok(None),
        };

        Ok(Some(val))
    }

    fn parse_unit(
        &self,
        stream: &mut Stream<Token<Self>>,
    ) -> Result<Option<CustomUnit>, ParseError> {
        if let Some(keyword) = allow_type_data_read!(TokenType::Keyword, stream) {
            let keyword = match keyword {
                Keyword::Custom(k) => k,
                _ => return Err(ParseError::new("invalid unit")),
            };

            return match keyword.as_str() {
                "px" => Ok(Some(CustomUnit::Px)),
                _ => Err(ParseError::new("invalid unit")),
            };
        }

        if let Some(token) = allow_type_data_read!(TokenType::Custom, stream) {
            return match token {
                CustomToken::PercentSign => Ok(Some(CustomUnit::Percent)),
                _ => Err(ParseError::new("invalid unit")),
            };
        }
        return Ok(None);
    }
}

impl MyCustomParser {
    fn read_hex_str(stream: &mut Stream<char>) -> Result<String, ParseError> {
        let mut str = String::new();

        loop {
            let c = stream.read()?;
            if "abcdef1234567890".contains(c) {
                str.push(c.clone());
            } else {
                break;
            }
        }

        stream.back();

        Ok(str)
    }
}
