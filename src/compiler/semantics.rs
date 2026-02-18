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
                    return Err("La condición del 'if' debe ser Bool".into());
                }
                for s in if_body { self.check_stmt(s)?; }
                if let Some(eb) = else_body { for s in eb { self.check_stmt(s)?; } }
                Ok(())
            }
            Stmt::While(cond, body) => {
                if self.check_expr(cond)? != KType::Bool {
                    return Err("La condición del 'while' debe ser Bool".into());
                }
                for s in body { self.check_stmt(s)?; }
                Ok(())
            }
            Stmt::Function(name, _params, body, _ret) => {
                self.symbols.insert(name.clone(), KType::Int);
                for s in body { self.check_stmt(s)?; }
                Ok(())
            }
            Stmt::Return(expr) => { self.check_expr(expr)?; Ok(()) }
            Stmt::Call(_, args) => {
                for a in args { self.check_expr(a)?; }
                Ok(())
            }
            // --- CORRECCIÓN: Validación de asignación en arreglos ---
            Stmt::IndexAssign(name, idx_expr, val_expr) => {
                if self.check_expr(idx_expr)? != KType::Int {
                    return Err(format!("El índice para '{}' debe ser un entero", name));
                }
                let val_type = self.check_expr(val_expr)?;
                match self.symbols.get(name) {
                    Some(KType::Array(inner)) => {
                        if **inner != val_type {
                            return Err(format!("No puedes asignar {:?} a un arreglo de {:?}", val_type, inner));
                        }
                    },
                    _ => return Err(format!("'{}' no es un arreglo o no está definido", name)),
                }
                Ok(())
            }
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
                else { Err(format!("Incompatibilidad: {:?} {} {:?}", lt, op, rt)) }
            }
            Expr::ArrayLiteral(elems) => {
                if elems.is_empty() { return Ok(KType::Array(Box::new(KType::Int))); }
                let first_t = self.check_expr(&elems[0])?;
                Ok(KType::Array(Box::new(first_t)))
            }
            Expr::ArrayAccess(name, idx) => {
                if self.check_expr(idx)? != KType::Int { return Err("Índice debe ser Int".into()); }
                self.symbols.get(name)
                    .and_then(|t| if let KType::Array(inner) = t { Some(*inner.clone()) } else { None })
                    .ok_or(format!("'{}' no es un arreglo", name))
            }
            Expr::Input => Ok(KType::Int),
            Expr::Call(_, _) => Ok(KType::Int),
        }
    }
}