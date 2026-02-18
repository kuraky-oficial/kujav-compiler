// src/parser/ast.rs
#[allow(dead_code)]
pub enum Expr {
    Number(i32),
    String(String),
    Identifier(String),
    Binary(Box<Expr>, String, Box<Expr>),
}

#[allow(dead_code)]
pub enum Stmt {
    Let(String, Expr),
    Print(Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>), // Cuerpo del IF y opcionalmente el del ELSE
    Function(String, Vec<String>, Vec<Stmt>),
}