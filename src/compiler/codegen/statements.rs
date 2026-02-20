// src/compiler/codegen/statements.rs
use crate::compiler::codegen::{Compiler, MethodInfo};
use crate::parser::ast::Stmt;

impl Compiler {
    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Let(name, expr, _) => {
                let is_ref = self.is_ref_expr(&expr);
                let slot = self.get_or_create_slot(&name);
                
                let type_sig = if is_ref { "Ljava/lang/Object;".to_string() } else { "I".to_string() };
                self.variable_types.insert(name.clone(), type_sig);

                self.compile_expression(expr);
                self.current_bytecode.push(if is_ref { 0x3A } else { 0x36 }); 
                self.current_bytecode.push(slot);
            }
            Stmt::Print(expr) => {
                let is_ref = self.is_ref_expr(&expr);
                // Llamamos a los métodos auxiliares definidos abajo
                self.prepare_println_call();
                self.compile_expression(expr);
                self.emit_println_invoke(is_ref);
            }
            Stmt::If(cond, if_b, else_b) => {
                self.compile_expression(cond);
                let opcode_pos = self.current_bytecode.len();
                self.current_bytecode.push(0x99); 
                let jump_to_else_idx = self.current_bytecode.len();
                self.current_bytecode.extend_from_slice(&[0x00, 0x00]); 
                for s in if_b { self.compile_statement(s); }
                if let Some(else_stmts) = else_b {
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
            Stmt::While(cond, body) => {
                let start_pos = self.current_bytecode.len();
                self.compile_expression(cond);
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
            Stmt::Function(name, params, body, return_type) => {
                let mut p_sigs = String::new();
                let (old_bc, old_vars, old_types, old_slot) = (
                    std::mem::take(&mut self.current_bytecode),
                    std::mem::take(&mut self.variables),
                    std::mem::take(&mut self.variable_types),
                    self.next_slot
                );

                self.next_slot = 0;
                for (p_name, p_type) in params {
                    p_sigs.push_str(&p_type.to_jvm_sig());
                    self.variables.insert(p_name.clone(), self.next_slot);
                    self.variable_types.insert(p_name, p_type.to_jvm_sig());
                    self.next_slot += 1;
                }

                for s in body { self.compile_statement(s); }
                
                if return_type == crate::compiler::types::KType::Void {
                    self.current_bytecode.push(0xB1);
                }

                self.methods.push(MethodInfo {
                    name_idx: self.cp.add_utf8(&name),
                    sig_idx: self.cp.add_utf8(&format!("({}){}", p_sigs, return_type.to_jvm_sig())),
                    bytecode: std::mem::take(&mut self.current_bytecode),
                    max_locals: self.next_slot as u16,
                });

                self.current_bytecode = old_bc;
                self.variables = old_vars;
                self.variable_types = old_types;
                self.next_slot = old_slot;
            }
            Stmt::Call(name, args) => {
                use crate::parser::ast::Expr;
                self.compile_expression(Expr::Call(name, args));
                self.current_bytecode.push(0x57); 
            }
            Stmt::Return(maybe_expr) => {
                if let Some(expr) = maybe_expr {
                    let is_ref = self.is_ref_expr(&expr);
                    self.compile_expression(expr);
                    self.current_bytecode.push(if is_ref { 0xB0 } else { 0xAC });
                } else {
                    self.current_bytecode.push(0xB1);
                }
            }
            Stmt::IndexAssign(name, idx_expr, val_expr) => {
                if let Some(&slot) = self.variables.get(&name) {
                    self.current_bytecode.push(0x19); // aload
                    self.current_bytecode.push(slot);
                    self.compile_expression(idx_expr);
                    self.compile_expression(val_expr);
                    self.current_bytecode.push(0x4F); // iastore
                }
            }
        }
    }
    pub fn prepare_println_call(&mut self) {
        let sys_u = self.cp.add_utf8("java/lang/System");
        let sys_c = self.cp.add_class(sys_u);
        let out_u = self.cp.add_utf8("out");
        let out_s = self.cp.add_utf8("Ljava/io/PrintStream;");
        let nt_out = self.cp.add_name_and_type(out_u, out_s);
        let f_out = self.cp.add_field_ref(sys_c, nt_out);
        self.current_bytecode.push(0xB2); // getstatic
        self.current_bytecode.extend_from_slice(&f_out.to_be_bytes());
    }

    pub fn emit_println_invoke(&mut self, is_ref: bool) {
        let sig_str = if is_ref { "(Ljava/lang/Object;)V" } else { "(I)V" };
        let ps_u = self.cp.add_utf8("java/io/PrintStream");
        let ps_c = self.cp.add_class(ps_u);
        let pr_u = self.cp.add_utf8("println");
        let pr_s = self.cp.add_utf8(sig_str);
        let nt_pr = self.cp.add_name_and_type(pr_u, pr_s);
        let m_pr = self.cp.add_method_ref(ps_c, nt_pr);
        self.current_bytecode.push(0xB6); // invokevirtual
        self.current_bytecode.extend_from_slice(&m_pr.to_be_bytes());
    }

    fn get_or_create_slot(&mut self, name: &str) -> u8 {
        *self.variables.entry(name.to_string()).or_insert_with(|| {
            let s = self.next_slot;
            self.next_slot += 1;
            s
        })
    }
}