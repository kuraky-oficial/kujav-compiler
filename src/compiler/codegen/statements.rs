// src/compiler/codegen/statements.rs
use crate::compiler::codegen::{Compiler, MethodInfo};
use crate::parser::ast::{Stmt, Expr};

impl Compiler {
    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Function(name, params, body, return_type) => {
                let ret_sig = match return_type.as_deref() { 
                    Some("S") => "Ljava/lang/String;", 
                    _ => "I" 
                };
                let mut param_sigs = String::new();
                for _ in &params { param_sigs.push('I'); }
                let sig = format!("({}){}", param_sigs, ret_sig);
                
                let n_idx = self.cp.add_utf8(&name);
                let s_idx = self.cp.add_utf8(&sig);

                let old_bc = std::mem::take(&mut self.current_bytecode);
                let old_vars = std::mem::take(&mut self.variables);
                let old_types = std::mem::take(&mut self.variable_types);
                let old_slot = self.next_slot;

                self.next_slot = 0;
                for p in params {
                    let slot = self.next_slot;
                    self.variables.insert(p.clone(), slot);
                    self.variable_types.insert(p, "I".to_string());
                    self.next_slot += 1;
                }

                for s in body { self.compile_statement(s); }
                if ret_sig == "V" { self.current_bytecode.push(0xB1); }

                self.methods.push(MethodInfo { 
                    name_idx: n_idx, sig_idx: s_idx, 
                    bytecode: std::mem::take(&mut self.current_bytecode), 
                    max_locals: self.next_slot as u16 
                });

                self.current_bytecode = old_bc; self.variables = old_vars; 
                self.variable_types = old_types; self.next_slot = old_slot;
            }
            Stmt::Let(name, expr) => {
                let is_str = self.is_string_expr(&expr);
                let slot = if let Some(&s) = self.variables.get(&name) { s } else {
                    let s = self.next_slot; self.variables.insert(name.clone(), s);
                    self.next_slot += 1; s
                };
                self.variable_types.insert(name, if is_str { "Ljava/lang/String;".into() } else { "I".into() });
                self.compile_expression(expr);
                self.current_bytecode.push(if is_str { 0x3A } else { 0x36 });
                self.current_bytecode.push(slot);
            }
            Stmt::Print(expr) => {
                let is_str = self.is_string_expr(&expr);
                // Bytecode para PrintStream... (mantén tu lógica de System.out)
                self.compile_expression(expr);
                // Instrucción invokevirtual...
            }
            // Implementar resto de casos (If, While, etc.)
            _ => {}
        }
    }
}