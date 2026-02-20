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
    // Nombre, Expresión, Tipo Anotado (como String para el parser)
    Let(String, Expr, Option<String>), 
    Print(Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    // Nombre, Parámetros (Nombre, Tipo), Cuerpo, Tipo de Retorno
    Function(String, Vec<(String, KType)>, Vec<Stmt>, KType),
    Call(String, Vec<Expr>),
    Return(Option<Expr>), // El retorno puede ser vacío en Lua
    IndexAssign(String, Expr, Expr),
}