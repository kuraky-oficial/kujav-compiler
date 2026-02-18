// src/parser/ast.rs
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    String(String),
    Identifier(String),
    Binary(Box<Expr>, String, Box<Expr>),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    Print(Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    Function(String, Vec<String>, Vec<Stmt>), // Definicion
    Call(String, Vec<Expr>),                 // Llamada
}