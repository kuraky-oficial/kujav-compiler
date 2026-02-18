// src/parser/mod.rs
pub mod ast; 
use pest::Parser;
use pest_derive::Parser as PestParser;
use self::ast::{Expr, Stmt};

#[derive(PestParser)]
#[grammar = "parser/kujav.pest"]
pub struct KujavParser;

pub fn parse_to_ast(input: &str) -> Vec<Stmt> {
    let pairs = KujavParser::parse(Rule::program, input)
        .expect("Error al parsear el archivo .kj")
        .next().unwrap();

    let mut statements = Vec::new();
    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::declaration | Rule::statement => {
                if let Some(stmt) = process_stmt(pair) {
                    statements.push(stmt);
                }
            }
            _ => {}
        }
    }
    statements
}

fn process_stmt(pair: pest::iterators::Pair<Rule>) -> Option<Stmt> {
    let inner_pair = pair.into_inner().next().unwrap();
    match inner_pair.as_rule() {
        Rule::let_decl => {
            let mut inner = inner_pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let expr = process_expr(inner.next().unwrap());
            Some(Stmt::Let(name, expr))
        }
        Rule::print_stmt => {
            let expr = process_expr(inner_pair.into_inner().next().unwrap());
            Some(Stmt::Print(expr))
        }
        Rule::return_stmt => {
            let expr = process_expr(inner_pair.into_inner().next().unwrap());
            Some(Stmt::Return(expr))
        }
        Rule::if_stmt => {
            let mut inner = inner_pair.into_inner();
            let condition = process_expr(inner.next().unwrap());
            let if_block = inner.next().unwrap();
            let mut if_body = Vec::new();
            for p in if_block.into_inner() { if let Some(s) = process_stmt(p) { if_body.push(s); } }
            let mut else_body = None;
            if let Some(else_block) = inner.next() {
                let mut e_body = Vec::new();
                for p in else_block.into_inner() { if let Some(s) = process_stmt(p) { e_body.push(s); } }
                else_body = Some(e_body);
            }
            Some(Stmt::If(condition, if_body, else_body))
        }
        Rule::while_stmt => {
            let mut inner = inner_pair.into_inner();
            let condition = process_expr(inner.next().unwrap());
            let block = inner.next().unwrap();
            let mut body = Vec::new();
            for p in block.into_inner() { if let Some(s) = process_stmt(p) { body.push(s); } }
            Some(Stmt::While(condition, body))
        }
        Rule::fun_decl => {
            let mut inner = inner_pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let mut params = Vec::new();
            let mut return_type = None;
            let mut block_pair = None;
            for part in inner {
                match part.as_rule() {
                    Rule::parameter_list => params = part.into_inner().map(|p| p.as_str().to_string()).collect(),
                    Rule::identifier => return_type = Some(part.as_str().to_string()),
                    Rule::block => block_pair = Some(part),
                    _ => {}
                }
            }
            let mut body = Vec::new();
            if let Some(bp) = block_pair {
                for p in bp.into_inner() { if let Some(s) = process_stmt(p) { body.push(s); } }
            }
            Some(Stmt::Function(name, params, body, return_type))
        }
        Rule::call_stmt => {
            let call_expr = inner_pair.into_inner().next().unwrap();
            let mut inner = call_expr.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let mut args = Vec::new();
            if let Some(arg_list) = inner.next() {
                for arg in arg_list.into_inner() { args.push(process_expr(arg)); }
            }
            Some(Stmt::Call(name, args))
        }
        _ => None,
    }
}

fn process_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut expr = process_comp_expr(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = process_comp_expr(inner.next().unwrap());
        expr = Expr::Binary(Box::new(expr), op, Box::new(right));
    }
    expr
}

fn process_comp_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut expr = process_term(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = process_term(inner.next().unwrap());
        expr = Expr::Binary(Box::new(expr), op, Box::new(right));
    }
    expr
}

fn process_term(pair: pest::iterators::Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut expr = process_factor(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = process_factor(inner.next().unwrap());
        expr = Expr::Binary(Box::new(expr), op, Box::new(right));
    }
    expr
}

fn process_factor(pair: pest::iterators::Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let mut expr = process_primary_expr(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = process_primary_expr(inner.next().unwrap());
        expr = Expr::Binary(Box::new(expr), op, Box::new(right));
    }
    expr
}

fn process_primary_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    let inner = pair.into_inner().next().expect("Primary vacío");
    match inner.as_rule() {
        Rule::string => Expr::String(inner.as_str().replace("\"", "")),
        Rule::number => Expr::Number(inner.as_str().parse().unwrap()),
        Rule::boolean => Expr::Boolean(inner.as_str() == "true"),
        Rule::input_expr => Expr::Input,
        Rule::expression => process_expr(inner), // Para los paréntesis
        Rule::identifier => Expr::Identifier(inner.as_str().to_string()),
        Rule::call_expr => {
            let mut inner_call = inner.into_inner();
            let name = inner_call.next().unwrap().as_str().to_string();
            let mut args = Vec::new();
            if let Some(arg_list) = inner_call.next() {
                for arg in arg_list.into_inner() { args.push(process_expr(arg)); }
            }
            Expr::Call(name, args)
        },
        _ => unreachable!("Error en primary: {:?}", inner.as_rule()),
    }
}