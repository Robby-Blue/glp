use std::collections::HashMap;

use crate::{
    CustomParser, ParseContext, Stream,
    errors::ParseError,
    parser::nodes::{Definition, Expression, FuncCall, Identifier, Value},
    tokenizer::{
        Keyword, Operator, Token,
        TokenType::{self},
    },
};

#[macro_export]
macro_rules! expect_specific_custom_keyword {
    ($expected:expr, $stream:expr) => {{
        match $crate::expect_custom_keyword!($stream).as_str() {
            $expected => Ok(()),
            _ => Err(ParseError::new("expected keyword not found"))?,
        }
    }};
}

#[macro_export]
macro_rules! expect_custom_keyword {
    ($stream:expr) => {{
        let token = $stream.read()?;
        match $crate::token_is_type_data!(token.token_type, TokenType::Keyword) {
            Some(data) => match data {
                $crate::tokenizer::Keyword::Custom(c) => c,
                _ => return Err(ParseError::new("expected custom keyword not found")),
            },
            None => Err(ParseError::new("expected type not found "))?,
        }
    }};
}

#[macro_export]
macro_rules! expect_type_get_data {
    ($variant:path, $stream:expr) => {{
        let token = $stream.read()?;
        match $crate::token_is_type_data!(token.token_type, $variant) {
            Some(data) => data,
            None => Err(ParseError::new(
                format!("expected type not found {}", line!()).as_str(),
            ))?,
        }
    }};
}

#[macro_export]
macro_rules! allow_type_data {
    ($variant:path, $stream:expr) => {{
        let token = $stream.peek()?;
        match $crate::token_is_type_data!(token.token_type, $variant) {
            Some(_) => true,
            None => false,
        }
    }};
}

#[macro_export]
macro_rules! allow_type_data_read {
    ($variant:path, $stream:expr) => {{
        let token = $stream.peek()?;
        match $crate::token_is_type_data!(token.token_type, $variant) {
            Some(data) => {
                $stream.read()?;
                Some(data)
            }
            None => None,
        }
    }};
}

#[macro_export]
macro_rules! token_is_type_data {
    ($t:expr, $variant:path) => {
        match $t {
            $variant(data) => Some(data),
            _ => None,
        }
    };
}

#[macro_export]
macro_rules! expect_type {
    ($variant:path, $stream:expr) => {{
        let token = $stream.read()?;
        match $crate::token_is_type_no_data!(token.token_type, $variant) {
            true => (),
            false => Err(ParseError::new(
                format!("expected type not found; {} {}", $stream.index, line!()).as_str(),
            ))?,
        }
    }};
}

#[macro_export]
macro_rules! allow_type {
    ($variant:path, $stream:expr) => {{
        let token = $stream.peek()?;
        $crate::token_is_type_no_data!(token.token_type, $variant)
    }};
}

#[macro_export]
macro_rules! token_is_type_no_data {
    ($t:expr, $variant:path) => {
        match $t {
            $variant => true,
            _ => false,
        }
    };
}

pub(crate) fn parse<T: CustomParser>(
    tokens: Vec<Token<T>>,
    ctx: &ParseContext<T>,
) -> Result<Vec<Definition<T>>, ParseError> {
    let mut stream = Stream::new(tokens);
    let mut nodes = vec![];

    while stream.has_more() {
        let token = stream.read()?;

        let def = match token.token_type {
            TokenType::Keyword(Keyword::Def) => parse_def(&mut stream, ctx)?,
            _ => unreachable!("unknown token type"),
        };

        nodes.push(def);
    }

    Ok(nodes)
}

fn parse_def<T: CustomParser>(
    stream: &mut Stream<Token<T>>,
    ctx: &ParseContext<T>,
) -> Result<Definition<T>, ParseError> {
    let name = expect_type_get_data!(TokenType::Identifier, stream);

    let mut paramters = vec![];

    expect_type!(TokenType::OpeningParenthesis, stream);
    while let Some(parameter) = allow_type_data_read!(TokenType::Identifier, stream) {
        paramters.push(Identifier::new(parameter));
        if !allow_type!(TokenType::Comma, stream) {
            break;
        }
    }
    expect_type!(TokenType::ClosingParenthesis, stream);

    expect_type!(TokenType::OpeningCurly, stream);

    let expr = parse_expression(stream, ctx, false, false)?;

    expect_type!(TokenType::ClosingCurly, stream);

    Ok(Definition::new(
        Identifier::new(name),
        paramters,
        Box::new(expr),
    ))
}

fn parse_func_call<T: CustomParser>(
    stream: &mut Stream<Token<T>>,
    ctx: &ParseContext<T>,
) -> Result<Value<T>, ParseError> {
    let name = expect_type_get_data!(TokenType::Identifier, stream);

    let mut args = vec![];
    let mut kwargs = HashMap::new();
    if allow_type!(TokenType::OpeningParenthesis, stream) {
        expect_type!(TokenType::OpeningParenthesis, stream);

        loop {
            let kwarg = try_parse_kwarg(stream, ctx)?;
            match kwarg {
                Some((kw, arg)) => {
                    kwargs.insert(kw, Box::new(arg));
                }
                None => {
                    let arg = allow_expression(stream, ctx, true, false);
                    match arg {
                        Some(arg) => args.push(Box::new(arg)),
                        None => break,
                    }
                }
            }
        }

        expect_type!(TokenType::ClosingParenthesis, stream);
    } else {
        if !allow_type!(TokenType::OpeningCurly, stream) {
            let expr = allow_expression(stream, ctx, true, true);
            match expr {
                Some(expr) => {
                    args.push(Box::new(expr));
                }
                None => {
                    return Ok(Value::Variable(Identifier::new(name)));
                }
            }
        }
    }
    let mut funcs = vec![];

    if allow_type!(TokenType::OpeningCurly, stream) {
        stream.read()?;

        while allow_type_data!(TokenType::Identifier, stream) {
            let func = parse_func_call(stream, ctx)?;
            match func {
                Value::FuncCall(func) => {
                    funcs.push(func);
                }
                _ => {
                    return Err(ParseError::new("found expr or so instead of func"));
                }
            }
        }

        expect_type!(TokenType::ClosingCurly, stream);
    }

    Ok(Value::FuncCall(Box::new(FuncCall::new(
        Identifier::new(name),
        args,
        kwargs,
        funcs,
    ))))
}

fn try_parse_kwarg<T: CustomParser>(
    stream: &mut Stream<Token<T>>,
    ctx: &ParseContext<T>,
) -> Result<Option<(String, Expression<T>)>, ParseError> {
    if !allow_type_data!(TokenType::Identifier, stream) {
        return Ok(None);
    }
    let name = expect_type_get_data!(TokenType::Identifier, stream);

    if !allow_type!(TokenType::Equals, stream) {
        stream.back();
        return Ok(None);
    }
    stream.read()?;

    let val = parse_expression(stream, ctx, true, false)?;

    if !allow_type!(TokenType::ClosingParenthesis, stream) {
        expect_type!(TokenType::Comma, stream);
    }

    Ok(Some((name, val)))
}

fn allow_expression<T: CustomParser>(
    stream: &mut Stream<Token<T>>,
    ctx: &ParseContext<T>,
    is_arg: bool,
    is_no_parentheses_arg: bool,
) -> Option<Expression<T>> {
    let i = stream.index;

    match parse_expression(stream, ctx, is_arg, is_no_parentheses_arg) {
        Ok(expr) => Some(expr),
        Err(_) => {
            stream.index = i;
            None
        }
    }
}

fn parse_expression<T: CustomParser>(
    stream: &mut Stream<Token<T>>,
    ctx: &ParseContext<T>,
    is_arg: bool,
    is_no_parentheses_arg: bool,
) -> Result<Expression<T>, ParseError> {
    let mut parts = vec![];

    loop {
        let val = parse_value(stream, ctx, is_arg, is_no_parentheses_arg)?;
        parts.push(ExpressionPart::Expression(val));

        if let Some(op) = allow_type_data_read!(TokenType::Operator, stream) {
            parts.push(ExpressionPart::Operator(op));
        } else {
            break;
        }
    }

    let mut part_stream = Stream::new(parts);

    let mut node_stack = vec![];
    let mut op_stack = vec![];

    // half of a shunting yard
    while part_stream.has_more() {
        let part = part_stream.read()?;

        match part {
            ExpressionPart::Expression(v) => {
                node_stack.push(v);
            }
            ExpressionPart::Operator(o1) => {
                while let Some(o2) = op_stack.last().cloned() {
                    let higher_precedence = get_precedence(&o2) > get_precedence(&o1);
                    let eq_precedence = get_precedence(&o2) == get_precedence(&o1);
                    let left_associative = is_left_associative(&o1);

                    if !(higher_precedence || (eq_precedence && left_associative)) {
                        break;
                    }

                    op_stack.pop();

                    let node = build_node(o2.clone(), &mut node_stack);
                    node_stack.push(node);
                }
                op_stack.push(o1.clone());
            }
        }
    }

    while let Some(op) = op_stack.pop() {
        let node = build_node(op, &mut node_stack);
        node_stack.push(node);
    }

    if node_stack.len() != 1 {
        todo!("oh");
    }

    Ok(node_stack.pop().unwrap())
}

fn build_node<T: CustomParser>(op: Operator, node_stack: &mut Vec<Expression<T>>) -> Expression<T> {
    let v1 = node_stack.pop();
    let v2 = node_stack.pop();

    match (v1, v2) {
        (Some(v1), Some(v2)) => Expression::new_operation(Box::new(v1), op, Box::new(v2)),
        (_, _) => todo!("make this result"),
    }
}

#[derive(Clone)]
enum ExpressionPart<T: CustomParser> {
    Expression(Expression<T>),
    Operator(Operator),
}

fn get_precedence(op: &Operator) -> u8 {
    match op {
        Operator::Addition => 0,
        Operator::Subtraction => 0,
        Operator::Multiplication => 1,
        Operator::Division => 1,
    }
}

fn is_left_associative(op: &Operator) -> bool {
    match op {
        Operator::Addition => true,
        Operator::Subtraction => true,
        Operator::Multiplication => false,
        Operator::Division => false,
    }
}

fn parse_value<T: CustomParser>(
    stream: &mut Stream<Token<T>>,
    ctx: &ParseContext<T>,
    is_arg: bool,
    is_no_parentheses_arg: bool,
) -> Result<Expression<T>, ParseError> {
    if allow_type!(TokenType::OpeningParenthesis, stream) {
        stream.read()?;
        let expr = parse_expression(stream, ctx, is_arg, is_no_parentheses_arg)?;
        stream.read()?;
        return Ok(expr);
    }
    if let Some(v) = allow_type_data_read!(TokenType::StringLiteral, stream) {
        return Ok(Expression::Value(Value::String(v)));
    }
    if allow_type_data!(TokenType::NumberLiteral, stream) {
        let (num, unit) = parse_num_literal_unit(stream, ctx)?;

        return Ok(Expression::Value(Value::Number(num, unit)));
    }
    if !is_no_parentheses_arg && let Some(op) = allow_type_data_read!(TokenType::Operator, stream) {
        return match op {
            Operator::Addition => {
                let (num, unit) = parse_num_literal_unit(stream, ctx)?;

                return Ok(Expression::Value(Value::Number(num, unit)));
            }
            Operator::Subtraction => {
                let (num, unit) = parse_num_literal_unit(stream, ctx)?;

                return Ok(Expression::Value(Value::Number(-num, unit)));
            }
            _ => Err(ParseError::new(format!("bad op {:?}", op).as_str())),
        };
    }
    if allow_type_data!(TokenType::Identifier, stream) {
        return Ok(Expression::Value(parse_func_call(stream, ctx)?));
    }

    let v = ctx.custom_parser.parse_value(stream)?;
    match v {
        Some(v) => Ok(Expression::Value(Value::Custom(v))),
        None => Err(ParseError::new("bad value")),
    }
}

fn parse_num_literal_unit<T: CustomParser>(
    stream: &mut Stream<Token<T>>,
    ctx: &ParseContext<T>,
) -> Result<(f64, Option<T::Unit>), ParseError> {
    let num = expect_type_get_data!(TokenType::NumberLiteral, stream);
    let unit = ctx.custom_parser.parse_unit(stream)?;

    return Ok((num, unit));
}
