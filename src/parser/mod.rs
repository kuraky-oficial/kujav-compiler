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
                let inner = pair.into_inner().next().unwrap();
                if let Some(stmt) = process_stmt(inner) {
                    statements.push(stmt);
                }
            }
            _ => {}
        }
    }
    statements
}

fn process_stmt(pair: pest::iterators::Pair<Rule>) -> Option<Stmt> {
    match pair.as_rule() {
        Rule::let_decl => {
            let mut inner_rules = pair.into_inner();
            let name = inner_rules.next().unwrap().as_str().to_string();
            let expr = process_expr(inner_rules.next().unwrap());
            Some(Stmt::Let(name, expr))
        }
        Rule::print_stmt => {
            let expr = process_expr(pair.into_inner().next().unwrap());
            Some(Stmt::Print(expr))
        }
        _ => None,
    }
}

fn process_expr(pair: pest::iterators::Pair<Rule>) -> Expr {
    let mut inner = pair.into_inner();
    let left = process_term(inner.next().unwrap());

    if let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = process_term(inner.next().unwrap());
        Expr::Binary(Box::new(left), op, Box::new(right))
    } else {
        left
    }
}

fn process_term(pair: pest::iterators::Pair<Rule>) -> Expr {
    // Entramos a lo que hay DENTRO del término
    let inner = pair.into_inner().next().expect("Term vacío");
    match inner.as_rule() {
        Rule::string => Expr::String(inner.as_str().replace("\"", "")),
        Rule::number => Expr::Number(inner.as_str().parse().unwrap()),
        Rule::identifier => Expr::Identifier(inner.as_str().to_string()),
        _ => unreachable!("Regla no esperada en term: {:?}", inner.as_rule()),
    }
}