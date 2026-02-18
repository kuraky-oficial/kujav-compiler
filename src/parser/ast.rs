// src/parser/ast.rs
#[allow(dead_code)]
pub enum Expr {
    Number(i32),
    String(String),
    Identifier(String),
}

#[allow(dead_code)]
pub enum Stmt {
    Let(String, Expr),
    Print(Expr),
    Function(String, Vec<String>, Vec<Stmt>),
}