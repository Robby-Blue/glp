use std::collections::HashMap;

use crate::{CustomParser, tokenizer::Operator};

#[derive(Clone)]
pub struct Identifier {
    pub text: String,
}

impl Identifier {
    pub fn new(text: String) -> Self {
        Identifier { text }
    }
}

#[derive(Clone)]
pub struct Definition<T: CustomParser> {
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub func: Box<Expression<T>>,
}

impl<T: CustomParser> Definition<T> {
    pub fn new(name: Identifier, parameters: Vec<Identifier>, func: Box<Expression<T>>) -> Self {
        Definition {
            name,
            parameters,
            func,
        }
    }
}

#[derive(Clone)]
pub struct FuncCall<T: CustomParser> {
    pub name: Identifier,
    pub args: Vec<Box<Expression<T>>>,
    pub kwargs: HashMap<String, Box<Expression<T>>>,
    pub funcs: Vec<Box<FuncCall<T>>>,
}
impl<T: CustomParser> FuncCall<T> {
    pub fn new(
        name: Identifier,
        args: Vec<Box<Expression<T>>>,
        kwargs: HashMap<String, Box<Expression<T>>>,
        funcs: Vec<Box<FuncCall<T>>>,
    ) -> Self {
        FuncCall {
            name,
            args,
            kwargs,
            funcs,
        }
    }
}

#[derive(Clone)]
pub enum Expression<T: CustomParser> {
    Value(Value<T>),
    Operation {
        val1: Box<Expression<T>>,
        operator: Operator,
        val2: Box<Expression<T>>,
    },
}

impl<T: CustomParser> Expression<T> {
    pub fn new_operation(
        val1: Box<Expression<T>>,
        operator: Operator,
        val2: Box<Expression<T>>,
    ) -> Self {
        Expression::Operation {
            val1,
            operator,
            val2,
        }
    }
}

#[derive(Clone)]
pub enum Value<T: CustomParser> {
    Number(f64, Option<T::Unit>),
    String(String),
    Bool(bool),
    Expression(Box<Expression<T>>),
    FuncCall(Box<FuncCall<T>>),
    Variable(Identifier),
    Custom(T::Value),
}
