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
            Stmt::If(cond, _, _) | Stmt::While(cond, _) => {
                if self.check_expr(cond)? != KType::Bool {
                    return Err("La condición debe ser Bool".into());
                }
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
            Expr::Input => Ok(KType::Int),
            _ => Ok(KType::Void),
        }
    }
}