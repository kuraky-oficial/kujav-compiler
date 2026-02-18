// src/compiler/semantics.rs
use std::collections::HashMap;
use crate::parser::ast::{Expr, Stmt};
use crate::compiler::types::KType;

pub struct SemanticAnalyzer {
    pub symbols: HashMap<String, KType>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self { symbols: HashMap::new() }
    }

    pub fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let(name, expr) => {
                let t = self.check_expr(expr)?;
                self.symbols.insert(name.clone(), t);
                Ok(())
            }
            Stmt::Print(expr) => { self.check_expr(expr)?; Ok(()) }
            Stmt::If(cond, if_body, else_body) => {
                if self.check_expr(cond)? != KType::Bool {
                    return Err("La condición del 'if' debe ser de tipo Bool".into());
                }
                for s in if_body { self.check_stmt(s)?; }
                if let Some(eb) = else_body {
                    for s in eb { self.check_stmt(s)?; }
                }
                Ok(())
            }
            Stmt::While(cond, body) => {
                if self.check_expr(cond)? != KType::Bool {
                    return Err("La condición del 'while' debe ser de tipo Bool".into());
                }
                for s in body { self.check_stmt(s)?; }
                Ok(())
            }
            Stmt::Function(name, params, body, _ret) => {
                // Por ahora lógica simple: registrar función como Int
                self.symbols.insert(name.clone(), KType::Int);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn check_expr(&self, expr: &Expr) -> Result<KType, String> {
        match expr {
            Expr::Number(_) => Ok(KType::Int),
            Expr::String(_) => Ok(KType::String),
            Expr::Boolean(_) => Ok(KType::Bool),
            Expr::Identifier(n) => self.symbols.get(n).cloned().ok_or(format!("Variable '{}' no definida", n)),
            Expr::Binary(l, op, r) => {
                let lt = self.check_expr(l)?;
                let rt = self.check_expr(r)?;
                if op == "+" && (lt == KType::String || rt == KType::String) { Ok(KType::String) }
                else if lt == rt { Ok(lt) }
                else { Err(format!("Tipos incompatibles: {:?} {} {:?}", lt, op, rt)) }
            }
            Expr::ArrayLiteral(elems) => {
                if elems.is_empty() { return Ok(KType::Array(Box::new(KType::Int))); }
                let first_t = self.check_expr(&elems[0])?;
                for e in elems {
                    if self.check_expr(e)? != first_t { return Err("No se permiten tipos mixtos en arreglos".into()); }
                }
                Ok(KType::Array(Box::new(first_t)))
            }
            Expr::ArrayAccess(name, idx) => {
                if self.check_expr(idx)? != KType::Int { return Err("El índice del arreglo debe ser Int".into()); }
                match self.symbols.get(name) {
                    Some(KType::Array(t)) => Ok(*t.clone()),
                    _ => Err(format!("'{}' no es un arreglo", name)),
                }
            }
            Expr::Input => Ok(KType::Int),
            Expr::Call(_, _) => Ok(KType::Int),
        }
    }
}