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
    pub func: Box<FuncCall<T>>,
}

impl<T: CustomParser> Definition<T> {
    pub fn new(name: Identifier, func: Box<FuncCall<T>>) -> Self {
        Definition { name, func }
    }
}

#[derive(Clone)]
pub struct FuncCall<T: CustomParser> {
    pub name: Identifier,
    pub args: Vec<Box<Argument<T>>>,
    pub funcs: Vec<Box<FuncCall<T>>>,
}
impl<T: CustomParser> FuncCall<T> {
    pub fn new(
        name: Identifier,
        args: Vec<Box<Argument<T>>>,
        funcs: Vec<Box<FuncCall<T>>>,
    ) -> Self {
        FuncCall { name, args, funcs }
    }
}

#[derive(Clone)]
pub struct Argument<T: CustomParser> {
    pub key: Identifier,
    pub val: Box<Expression<T>>,
}
impl<T: CustomParser> Argument<T> {
    pub fn new(key: Identifier, val: Box<Expression<T>>) -> Self {
        Argument { key, val }
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
    Expression(Box<Expression<T>>),
    Custom(T::Value),
}
