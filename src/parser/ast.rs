// src/parser/ast.rs
use crate::compiler::types::KType;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    String(String),
    Boolean(bool),
    Identifier(String),
    Binary(Box<Expr>, String, Box<Expr>),
    Call(String, Vec<Expr>),
    Input,
    ArrayLiteral(Vec<Expr>),
    ArrayAccess(String, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr, Option<String>), // (Nombre, Expr, Tipo Opcional)
    Print(Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    Function(String, Vec<(String, KType)>, Vec<Stmt>, KType), // (Nombre, Params, Cuerpo, Retorno)
    Call(String, Vec<Expr>),
    Return(Option<Expr>),
    IndexAssign(String, Expr, Expr),
}
