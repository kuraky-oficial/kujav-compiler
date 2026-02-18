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
// src/parser/ast.rs
#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    Print(Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    // Añadimos Option<String> para el tipo de retorno (I, S, etc.)
    Function(String, Vec<String>, Vec<Stmt>, Option<String>), 
    Call(String, Vec<Expr>),
    Return(Expr), // Nuevo para soportar 'return'
}