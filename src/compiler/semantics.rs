// src/compiler/codegen/statements.rs
use crate::compiler::codegen::{Compiler, MethodInfo};
use crate::parser::ast::Stmt; // Importación necesaria

impl Compiler {
    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
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
                let sys_u = self.cp.add_utf8("java/lang/System");
                let sys_c = self.cp.add_class(sys_u);
                let out_u = self.cp.add_utf8("out");
                let out_s = self.cp.add_utf8("Ljava/io/PrintStream;");
                let o_nt = self.cp.add_name_and_type(out_u, out_s);
                let f_out = self.cp.add_field_ref(sys_c, o_nt);
                self.current_bytecode.push(0xB2); 
                self.current_bytecode.extend_from_slice(&f_out.to_be_bytes());

                self.compile_expression(expr);

                let sig = if is_str { "(Ljava/lang/String;)V" } else { "(I)V" };
                let ps_u = self.cp.add_utf8("java/io/PrintStream");
                let ps_c = self.cp.add_class(ps_u);
                let pr_u = self.cp.add_utf8("println");
                let pr_s = self.cp.add_utf8(sig);
                let pr_nt = self.cp.add_name_and_type(pr_n, pr_s);
                let m_pr = self.cp.add_method_ref(ps_c, pr_nt);
                self.current_bytecode.push(0xB6); 
                self.current_bytecode.extend_from_slice(&m_pr.to_be_bytes());
            }
            // ... resto de Stmt (If, While, Function) igual que antes
            _ => {}
        }
    }
}