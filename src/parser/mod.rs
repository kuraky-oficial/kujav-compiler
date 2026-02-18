// src/parser/mod.rs
pub mod ast; 
use pest::Parser; // <--- ESTO ES LO QUE FALTABA
use pest_derive::Parser as PestParser; // Evitamos conflicto de nombres
use self::ast::{Expr, Stmt}; // Usamos self para ser más claros

#[derive(PestParser)]
#[grammar = "parser/kujav.pest"]
pub struct KujavParser;

pub fn parse_to_ast(input: &str) -> Vec<Stmt> {
    // Usamos Rule::program explícitamente para ayudar a la inferencia
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
                        statements.push(Stmt::Let(name, Expr::String(value_str.replace("\"", ""))));
                    }
                    _ => {}
                }
            }
            Rule::EOI => (),
            _ => {}
        }
    }
    statements
}