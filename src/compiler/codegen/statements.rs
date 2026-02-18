// src/compiler/codegen/statements.rs
use crate::compiler::codegen::{Compiler, MethodInfo};
use crate::parser::ast::Stmt;

impl Compiler {
    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Function(name, params, body, return_type) => {
                let ret_sig = match return_type.as_deref() { 
                    Some("S") => "Ljava/lang/String;", 
                    Some("V") => "V", 
                    _ => "I" 
                };
                let mut p_sigs = String::new();
                for _ in &params { p_sigs.push('I'); }
                let sig = format!("({}){}", p_sigs, ret_sig);
                
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

                self.current_bytecode = old_bc; 
                self.variables = old_vars; 
                self.variable_types = old_types; 
                self.next_slot = old_slot;
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
                let sys_c = self.cp.add_class(self.cp.add_utf8("java/lang/System"));
                let o_nt = self.cp.add_name_and_type(self.cp.add_utf8("out"), self.cp.add_utf8("Ljava/io/PrintStream;"));
                let f_out = self.cp.add_field_ref(sys_c, o_nt);
                self.current_bytecode.push(0xB2); 
                self.current_bytecode.extend_from_slice(&f_out.to_be_bytes());

                self.compile_expression(expr);

                let sig = if is_str { "(Ljava/lang/String;)V" } else { "(I)V" };
                let ps_c = self.cp.add_class(self.cp.add_utf8("java/io/PrintStream"));
                let pr_nt = self.cp.add_name_and_type(self.cp.add_utf8("println"), self.cp.add_utf8(sig));
                let m_pr = self.cp.add_method_ref(ps_c, pr_nt);
                self.current_bytecode.push(0xB6); 
                self.current_bytecode.extend_from_slice(&m_pr.to_be_bytes());
            }
            Stmt::Call(name, args) => {
                // Reusamos la lógica de expresiones para la llamada
                use crate::parser::ast::Expr;
                self.compile_expression(Expr::Call(name, args));
                // Si la función devuelve I pero se llamó como sentencia, quitamos el valor del stack
                self.current_bytecode.push(0x57); // pop
            }
            Stmt::Return(expr) => {
                let is_str = self.is_string_expr(&expr);
                self.compile_expression(expr);
                self.current_bytecode.push(if is_str { 0xB0 } else { 0xAC });
            }
            Stmt::If(condition, if_body, else_body) => {
                self.compile_expression(condition);
                let opcode_pos = self.current_bytecode.len();
                self.current_bytecode.push(0x99); 
                let jump_to_else_idx = self.current_bytecode.len();
                self.current_bytecode.extend_from_slice(&[0x00, 0x00]); 
                for s in if_body { self.compile_statement(s); }
                if let Some(else_stmts) = else_body {
                    let goto_pos = self.current_bytecode.len();
                    self.current_bytecode.push(0xA7); 
                    let jump_to_end_idx = self.current_bytecode.len();
                    self.current_bytecode.extend_from_slice(&[0x00, 0x00]);
                    let off_else = (self.current_bytecode.len() - opcode_pos) as i16;
                    self.current_bytecode[jump_to_else_idx..jump_to_else_idx+2].copy_from_slice(&off_else.to_be_bytes());
                    for s in else_stmts { self.compile_statement(s); }
                    let off_end = (self.current_bytecode.len() - goto_pos) as i16;
                    self.current_bytecode[jump_to_end_idx..jump_to_end_idx+2].copy_from_slice(&off_end.to_be_bytes());
                } else {
                    let off = (self.current_bytecode.len() - opcode_pos) as i16;
                    self.current_bytecode[jump_to_else_idx..jump_to_else_idx+2].copy_from_slice(&off.to_be_bytes());
                }
            }
            Stmt::While(condition, body) => {
                let start_pos = self.current_bytecode.len();
                self.compile_expression(condition);
                let ifeq_pos = self.current_bytecode.len();
                self.current_bytecode.push(0x99); 
                let jump_to_end_idx = self.current_bytecode.len();
                self.current_bytecode.extend_from_slice(&[0x00, 0x00]);
                for s in body { self.compile_statement(s); }
                let goto_pos = self.current_bytecode.len();
                self.current_bytecode.push(0xA7); 
                let off_start = (start_pos as i32 - goto_pos as i32) as i16;
                self.current_bytecode.extend_from_slice(&off_start.to_be_bytes());
                let off_end = (self.current_bytecode.len() - ifeq_pos) as i16;
                self.current_bytecode[jump_to_end_idx..jump_to_end_idx+2].copy_from_slice(&off_end.to_be_bytes());
            }
        }
    }
}