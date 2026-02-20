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

    pub fn analyze(&mut self, ast: &[Stmt]) -> Result<(), String> {
        for stmt in ast { self.check_stmt(stmt)?; }
        Ok(())
    }

    pub fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let(name, expr, _type_ann) => {
                let t = self.check_expr(expr)?;
                self.symbols.insert(name.clone(), t);
                Ok(())
            }
            Stmt::Print(expr) => { self.check_expr(expr)?; Ok(()) }
            Stmt::Function(name, params, body, ret_type) => {
                self.symbols.insert(name.clone(), ret_type.clone());
                
                // --- SOLUCIÓN AL ERROR: Gestionar el ámbito de los parámetros ---
                let old_symbols = self.symbols.clone(); // Guardamos ámbito superior
                for (p_name, p_type) in params {
                    self.symbols.insert(p_name.clone(), p_type.clone());
                }

                for s in body { self.check_stmt(s)?; }
                
                self.symbols = old_symbols; // Restauramos el ámbito original
                Ok(())
            }
            Stmt::If(cond, if_body, else_body) => {
                if self.check_expr(cond)? != KType::Bool { return Err("Condición debe ser Bool".into()); }
                for s in if_body { self.check_stmt(s)?; }
                if let Some(eb) = else_body { for s in eb { self.check_stmt(s)?; } }
                Ok(())
            }
            Stmt::While(cond, body) => {
                if self.check_expr(cond)? != KType::Bool { return Err("Condición debe ser Bool".into()); }
                for s in body { self.check_stmt(s)?; }
                Ok(())
            }
            Stmt::Return(maybe_expr) => {
                if let Some(expr) = maybe_expr { self.check_expr(expr)?; }
                Ok(())
            }
            Stmt::Call(_, args) => { for a in args { self.check_expr(a)?; } Ok(()) }
            Stmt::IndexAssign(name, idx, val) => {
                self.check_expr(idx)?;
                let val_t = self.check_expr(val)?;
                match self.symbols.get(name) {
                    Some(KType::Array(inner)) if **inner == val_t => Ok(()),
                    _ => Err(format!("Error de tipo en arreglo '{}'", name)),
                }
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
                else { Ok(lt) }
            }
            Expr::ArrayLiteral(elems) => {
                if elems.is_empty() { return Ok(KType::Array(Box::new(KType::Int))); }
                Ok(KType::Array(Box::new(self.check_expr(&elems[0])?)))
            }
            Expr::ArrayAccess(name, idx) => {
                self.check_expr(idx)?;
                match self.symbols.get(name) {
                    Some(KType::Array(inner)) => Ok(*inner.clone()),
                    _ => Err(format!("'{}' no es un arreglo", name)),
                }
            }
            _ => Ok(KType::Int),
        }
    }
}