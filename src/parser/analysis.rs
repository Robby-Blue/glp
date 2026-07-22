use std::collections::HashMap;

use crate::{
    CustomParser,
    errors::ParseError,
    parser::nodes::{Definition, Expression, FuncCall, Identifier, Value},
};

pub fn analyse<T: CustomParser>(ast: Vec<Definition<T>>) -> Result<FuncCall<T>, ParseError> {
    let mut defs = HashMap::new();

    for def in ast {
        let name = def.name.text.clone();
        defs.insert(name, def);
    }

    let main = defs.get(&"main".to_string()).expect("no main").clone();

    let ctx = AnalysisContext::new(defs);
    let expr = analyse_expr(&main.func, &ctx)?;

    match expr {
        Expression::Value(Value::FuncCall(f)) => return Ok(*f),
        _ => unreachable!(),
    }
}

fn analyse_expr<T: CustomParser>(
    expr: &Expression<T>,
    ctx: &AnalysisContext<T>,
) -> Result<Expression<T>, ParseError> {
    let new_expr = match expr {
        Expression::Value(value) => Expression::Value(analyse_value(value, ctx)?),
        Expression::Operation {
            val1,
            operator,
            val2,
        } => Expression::new_operation(
            Box::new(analyse_expr(val1, ctx)?),
            operator.clone(),
            Box::new(analyse_expr(val2, ctx)?),
        ),
    };
    Ok(new_expr)
}

fn analyse_value<T: CustomParser>(
    val: &Value<T>,
    ctx: &AnalysisContext<T>,
) -> Result<Value<T>, ParseError> {
    let new_val = match val {
        Value::Number(_, _) => val.clone(),
        Value::String(_) => val.clone(),
        Value::Custom(_) => val.clone(),
        Value::Variable(name) => {
            let var = ctx.variables.get(&name.text).expect("var not found");
            let new_expr = analyse_expr(var, ctx)?;
            Value::Expression(Box::new(new_expr))
        }
        Value::Expression(expr) => Value::Expression(Box::new(analyse_expr(expr, ctx)?)),
        Value::FuncCall(func_call) => {
            if ctx.func_defs.contains_key(&func_call.name.text) {
                expand_func_call(&*func_call, ctx)?
            } else {
                Value::FuncCall(Box::new(analyse_func_call(&*func_call, ctx)?))
            }
        }
    };
    Ok(new_val)
}

fn merge_parameters<T: CustomParser>(
    func_call: &FuncCall<T>,
    ctx: &AnalysisContext<T>,
) -> Result<HashMap<String, Box<Expression<T>>>, ParseError> {
    let name = &func_call.name.text;

    let args = &func_call.args;
    let kwargs = &func_call.kwargs;

    let def = ctx.func_defs.get(name).unwrap();

    let mut merged = HashMap::new();

    for (arg_name, arg) in def.parameters.iter().zip(args) {
        let new_expr = analyse_expr(&*arg.clone(), ctx)?;
        merged.insert(arg_name.text.clone(), Box::new(new_expr));
    }

    for (arg_name, arg) in kwargs.iter() {
        let new_expr = analyse_expr(&*arg.clone(), ctx)?;
        merged.insert(arg_name.clone(), Box::new(new_expr));
    }

    Ok(merged)
}

fn analyse_func_call<T: CustomParser>(
    func_call: &FuncCall<T>,
    ctx: &AnalysisContext<T>,
) -> Result<FuncCall<T>, ParseError> {
    let name = &func_call.name.text;

    let mut new_args = vec![];
    for arg in &func_call.args {
        let new_arg = analyse_expr(&*arg.clone(), ctx)?;
        new_args.push(Box::new(new_arg));
    }

    let mut new_kwargs = HashMap::new();
    for (key, arg) in &func_call.kwargs {
        let new_arg = analyse_expr(&*arg.clone(), ctx)?;
        new_kwargs.insert(key.clone(), Box::new(new_arg));
    }

    let mut body_funcs = vec![];
    for func in &func_call.funcs {
        let new_call = analyse_func_call(&func, ctx)?;
        body_funcs.push(Box::new(new_call));
    }

    let new_func = FuncCall::new(
        Identifier::new(name.clone()),
        new_args,
        new_kwargs,
        body_funcs,
    );
    Ok(new_func)
}

fn expand_func_call<T: CustomParser>(
    func_call: &FuncCall<T>,
    ctx: &AnalysisContext<T>,
) -> Result<Value<T>, ParseError> {
    // has name of func, and args. lookup def, find map i guess, and keep passing it? with extra
    // stuff

    // find use cases of the arg names as vars, and replace them with teh contents
    // and also actually like parse for vars in the parser somehow
    let def = ctx.func_defs.get(&func_call.name.text).unwrap();
    let expr = def.func.clone();

    let parameters = merge_parameters(func_call, &ctx)?;

    let new_ctx = AnalysisContext::new_from(ctx, parameters);

    let new_expr = analyse_expr(&expr, &new_ctx)?;
    Ok(Value::Expression(Box::new(new_expr)))
}

struct AnalysisContext<T: CustomParser> {
    func_defs: HashMap<String, Definition<T>>,
    variables: HashMap<String, Box<Expression<T>>>,
}

impl<T: CustomParser> AnalysisContext<T> {
    fn new(func_defs: HashMap<String, Definition<T>>) -> Self {
        AnalysisContext {
            func_defs,
            variables: HashMap::new(),
        }
    }
    fn new_from(
        old_ctx: &AnalysisContext<T>,
        variables: HashMap<String, Box<Expression<T>>>,
    ) -> Self {
        AnalysisContext {
            func_defs: old_ctx.func_defs.clone(),
            variables,
        }
    }
}
