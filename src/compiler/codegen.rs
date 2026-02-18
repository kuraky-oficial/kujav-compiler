// src/compiler/codegen.rs
use std::collections::HashMap;
use crate::core::constant_pool::ConstantPool;
use crate::parser::ast::{Stmt, Expr};

pub struct Compiler {
    pub cp: ConstantPool,
    pub bytecode: Vec<u8>,
    pub variables: HashMap<String, u8>,
    pub variable_types: HashMap<String, String>, 
    pub next_slot: u8, 
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            cp: ConstantPool::new(),
            bytecode: Vec::new(),
            variables: HashMap::new(),
            variable_types: HashMap::new(),
            next_slot: 1, 
        }
    }

    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Let(name, expr) => {
                let slot = self.next_slot;
                self.variables.insert(name.clone(), slot);
                self.next_slot += 1;

                let type_tag = match &expr {
                    Expr::String(_) => "Ljava/lang/String;",
                    _ => "I", 
                };
                self.variable_types.insert(name.clone(), type_tag.to_string());

                self.compile_expression(expr);
                
                if type_tag == "I" {
                    self.bytecode.push(0x36); // istore
                } else {
                    self.bytecode.push(0x3A); // astore
                }
                self.bytecode.push(slot);
            }
            Stmt::Print(expr) => {
                let sys_name = self.cp.add_utf8("java/lang/System");
                let sys_cls = self.cp.add_class(sys_name);
                let out_name = self.cp.add_utf8("out");
                let out_type = self.cp.add_utf8("Ljava/io/PrintStream;");
                let out_nt = self.cp.add_name_and_type(out_name, out_type);
                let field_out = self.cp.add_field_ref(sys_cls, out_nt);

                self.bytecode.push(0xB2); // getstatic
                self.bytecode.extend_from_slice(&field_out.to_be_bytes());

                // Cambiado is_obj por _is_obj para evitar la advertencia
                let (sig, _is_obj) = match &expr {
                    Expr::String(_) => ("(Ljava/lang/String;)V", true),
                    Expr::Identifier(name) => {
                        let t = self.variable_types.get(name).unwrap_or(&"I".to_string()).clone();
                        (if t == "I" { "(I)V" } else { "(Ljava/lang/String;)V" }, t != "I")
                    },
                    _ => ("(I)V", false),
                };

                self.compile_expression(expr);

                let ps_name = self.cp.add_utf8("java/io/PrintStream");
                let ps_cls = self.cp.add_class(ps_name);
                let pr_name = self.cp.add_utf8("println");
                let pr_type = self.cp.add_utf8(sig);
                let pr_nt = self.cp.add_name_and_type(pr_name, pr_type);
                let method_println = self.cp.add_method_ref(ps_cls, pr_nt);

                self.bytecode.push(0xB6); // invokevirtual
                self.bytecode.extend_from_slice(&method_println.to_be_bytes());
            }
            _ => {}
        }
    }

    fn compile_expression(&mut self, expr: Expr) {
        match expr {
            Expr::Number(val) => {
                self.bytecode.push(0x10); // bipush
                self.bytecode.push(val as u8);
            }
            Expr::String(content) => {
                let msg_utf8 = self.cp.add_utf8(&content);
                let s_idx = self.cp.add_string(msg_utf8);
                self.bytecode.push(0x12); // ldc
                self.bytecode.push(s_idx as u8);
            }
            Expr::Identifier(name) => {
                if let Some(&slot) = self.variables.get(&name) {
                    let t = self.variable_types.get(&name).unwrap();
                    if t == "I" { self.bytecode.push(0x15); } else { self.bytecode.push(0x19); }
                    self.bytecode.push(slot);
                }
            }
            Expr::Binary(left, op, right) => {
                self.compile_expression(*left);
                self.compile_expression(*right);
                if op == "+" { self.bytecode.push(0x60); } // iadd
            }
        }
    }
}