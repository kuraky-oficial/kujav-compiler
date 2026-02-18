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
            let mut inner_rules = inner_pair.into_inner();
            let name = inner_rules.next().unwrap().as_str().to_string();
            let expr = process_expr(inner_rules.next().unwrap());
            Some(Stmt::Let(name, expr))
        }
        Rule::print_stmt => {
            let expr = process_expr(inner_pair.into_inner().next().unwrap());
            Some(Stmt::Print(expr))
        }
        Rule::if_stmt => {
            let mut inner = inner_pair.into_inner();
            let condition = process_expr(inner.next().unwrap());
            let if_block = inner.next().unwrap();
            let mut if_body = Vec::new();
            for p in if_block.into_inner() {
                if let Some(s) = process_stmt(p) { if_body.push(s); }
            }
            let mut else_body = None;
            if let Some(else_block) = inner.next() {
                let mut e_body = Vec::new();
                for p in else_block.into_inner() {
                    if let Some(s) = process_stmt(p) { e_body.push(s); }
                }
                else_body = Some(e_body);
            }
            Some(Stmt::If(condition, if_body, else_body))
        }
        Rule::while_stmt => {
            let mut inner = inner_pair.into_inner();
            let condition = process_expr(inner.next().unwrap());
            let block = inner.next().unwrap();
            let mut body = Vec::new();
            for p in block.into_inner() {
                if let Some(s) = process_stmt(p) { body.push(s); }
            }
            Some(Stmt::While(condition, body))
        }
        _ => None,
    }
}

fn process_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
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
        Rule::identifier => Expr::Identifier(inner.as_str().to_string()),
        _ => unreachable!("Error en primary: {:?}", inner.as_rule()),
    }
}