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
            // Manejamos declaraciones (como let)
            Rule::declaration => {
                let inner = pair.into_inner().next().unwrap();
                if let Some(stmt) = process_stmt(inner) {
                    statements.push(stmt);
                }
            }
            // Manejamos sentencias directas (como print)
            Rule::statement => {
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

// Función auxiliar para no repetir código
fn process_stmt(pair: pest::iterators::Pair<Rule>) -> Option<Stmt> {
    match pair.as_rule() {
        Rule::let_decl => {
            let mut inner_rules = pair.into_inner();
            let name = inner_rules.next().unwrap().as_str().to_string();
            let value_str = inner_rules.next().unwrap().as_str();
            Some(Stmt::Let(name, Expr::String(value_str.replace("\"", ""))))
        }
        Rule::print_stmt => {
            let content = pair.into_inner().next().unwrap().as_str();
            Some(Stmt::Print(Expr::String(content.replace("\"", ""))))
        }
        _ => None,
    }
}