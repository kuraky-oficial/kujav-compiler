use pest::Parser;
use crate::parser::ast::{Expr, Stmt};

#[derive(pest_derive::Parser)]
#[grammar = "parser/kujav.pest"]
pub struct KujavParser;

pub fn parse_to_ast(input: &str) -> Vec<Stmt> {
    let pairs = KujavParser::parse(Rule::program, input)
        .expect("Error al parsear el archivo .kj")
        .next().unwrap();

    let mut statements = Vec::new();

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::declaration => {
                let inner = pair.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::let_decl => {
                        let mut inner_rules = inner.into_inner();
                        let name = inner_rules.next().unwrap().as_str().to_string();
                        let value_str = inner_rules.next().unwrap().as_str();
                        // Simplificación: tratamos todo como String por ahora
                        statements.push(Stmt::Let(name, Expr::String(value_str.replace("\"", ""))));
                    }
                    _ => {} // Implementar fun_decl después
                }
            }
            Rule::EOI => (),
            _ => {}
        }
    }
    statements
}