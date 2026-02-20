// src/parser/mod.rs
pub mod ast;
use pest::Parser;
use pest_derive::Parser as PestParser;
use crate::compiler::types::KType;
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
            Rule::import_decl => {
                // Por ahora los imports se pueden manejar como una sentencia especial o metadatos
            }
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
    let mut next = inner.next().unwrap();
    let mut type_name_str = None;

    if next.as_rule() == Rule::type_name {
        // Guardamos el nombre del tipo como String para el AST
        type_name_str = Some(next.as_str().to_string());
        next = inner.next().unwrap();
    }
    let expr = process_expr(next);
    Some(Stmt::Let(name, expr, type_name_str))
}
        Rule::fun_decl => {
            let mut inner = inner_pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let mut params = Vec::new();
            let mut ret_type = KType::Void;
            let mut body = Vec::new();

            for next in inner {
                match next.as_rule() {
                    Rule::parameter_list => {
                        for p in next.into_inner() {
                            let mut p_inner = p.into_inner();
                            let p_name = p_inner.next().unwrap().as_str().to_string();
                            let p_type = parse_type(p_inner.next().unwrap());
                            params.push((p_name, p_type));
                        }
                    }
                    Rule::type_name => {
                        ret_type = parse_type(next);
                    }
                    Rule::block => {
                        for p in next.into_inner() {
                            if let Some(s) = process_stmt(p) { body.push(s); }
                        }
                    }
                    _ => {}
                }
            }
            Some(Stmt::Function(name, params, body, ret_type))
        }
        Rule::if_stmt => {
            let mut inner = inner_pair.into_inner();
            let cond = process_expr(inner.next().unwrap());
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
            Some(Stmt::If(cond, if_body, else_body))
        }
        Rule::while_stmt => {
            let mut inner = inner_pair.into_inner();
            let cond = process_expr(inner.next().unwrap());
            let block = inner.next().unwrap();
            let mut body = Vec::new();
            for p in block.into_inner() {
                if let Some(s) = process_stmt(p) { body.push(s); }
            }
            Some(Stmt::While(cond, body))
        }
        Rule::return_stmt => {
            let expr = inner_pair.into_inner().next().map(process_expr);
            Some(Stmt::Return(expr))
        }
        Rule::print_stmt => {
            let expr = process_expr(inner_pair.into_inner().next().unwrap());
            Some(Stmt::Print(expr))
        }
        Rule::call_stmt => {
            let mut inner = inner_pair.into_inner();
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

// Función auxiliar para parsear tipos (ej: Int, String, Int[])
fn parse_type(pair: pest::iterators::Pair<Rule>) -> KType {
    let mut inner = pair.into_inner();
    let base_name = inner.next().unwrap().as_str();
    let mut ktype = match base_name {
        "Int" => KType::Int,
        "String" => KType::String,
        "Bool" => KType::Bool,
        _ => KType::Custom(base_name.to_string()),
    };
    // Manejar dimensiones de arreglos []
    for _ in inner {
        ktype = KType::Array(Box::new(ktype));
    }
    ktype
}

// (Las funciones process_expr, process_term, etc. se mantienen igual pero apuntando a las reglas de la nueva gramática)
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
        Rule::array_lit => {
            let mut elements = Vec::new();
            for e in inner.into_inner() {
                elements.push(process_expr(e));
            }
            Expr::ArrayLiteral(elements)
        }
        Rule::expression => process_expr(inner), // Para ( expr )
        _ => unreachable!("Error en primary: {:?}", inner.as_rule()),
    }
}