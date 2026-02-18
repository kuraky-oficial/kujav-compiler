// src/compiler/codegen/expressions.rs
use crate::compiler::codegen::Compiler;
use crate::parser::ast::Expr;

impl Compiler {
    pub fn is_string_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::String(_) => true,
            Expr::Identifier(n) => self.variable_types.get(n).map(|t| t.as_str()) == Some("Ljava/lang/String;"),
            Expr::Binary(l, op, r) => op == "+" && (self.is_string_expr(l) || self.is_string_expr(r)),
            _ => false,
        }
    }

    pub fn compile_expression(&mut self, expr: Expr) {
        match expr {
            Expr::Number(val) => { self.current_bytecode.push(0x10); self.current_bytecode.push(val as u8); }
            Expr::Boolean(val) => { self.current_bytecode.push(if val { 0x04 } else { 0x03 }); }
            Expr::String(c) => { /* Lógica LDC */ }
            Expr::Binary(l, op, r) => {
                // Aquí va la lógica de StringBuilder si es string, o iadd si es int
            }
            Expr::Input => { /* Lógica Scanner */ }
            Expr::Identifier(n) => { /* Lógica ILOAD/ALOAD */ }
            _ => {}
        }
    }
}