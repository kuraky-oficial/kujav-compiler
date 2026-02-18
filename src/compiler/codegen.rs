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

                // CORRECCIÓN DE TIPOS: Todo convertido a String para evitar el error E0308
                let type_tag = match &expr {
                    Expr::Binary(_, _, _) | Expr::Number(_) => "I".to_string(),
                    Expr::String(_) => "Ljava/lang/String;".to_string(),
                    Expr::Identifier(n) => self.variable_types.get(n).cloned().unwrap_or_else(|| "I".to_string()),
                };
                self.variable_types.insert(name.clone(), type_tag.clone());

                self.compile_expression(expr);
                
                if type_tag == "I" {
                    self.bytecode.push(0x36); // istore
                } else {
                    self.bytecode.push(0x3A); // astore
                }
                self.bytecode.push(slot);
            }
            Stmt::Print(expr) => {
                let sys_n = self.cp.add_utf8("java/lang/System");
                let sys_c = self.cp.add_class(sys_n);
                let out_n = self.cp.add_utf8("out");
                let out_t = self.cp.add_utf8("Ljava/io/PrintStream;");
                let out_nt = self.cp.add_name_and_type(out_n, out_t);
                let field_out = self.cp.add_field_ref(sys_c, out_nt);

                self.bytecode.push(0xB2); // getstatic
                self.bytecode.extend_from_slice(&field_out.to_be_bytes());

                let (sig, _is_obj) = match &expr {
                    Expr::String(_) => ("(Ljava/lang/String;)V", true),
                    Expr::Identifier(name) => {
                        let t = self.variable_types.get(name).unwrap_or(&"I".to_string()).clone();
                        (if t == "I" { "(I)V" } else { "(Ljava/lang/String;)V" }, t != "I")
                    },
                    _ => ("(I)V", false),
                };

                self.compile_expression(expr);

                let ps_n = self.cp.add_utf8("java/io/PrintStream");
                let ps_c = self.cp.add_class(ps_n);
                
                // CORRECCIÓN: pr_name definida correctamente para evitar el error E0425
                let pr_name = self.cp.add_utf8("println");
                let pr_t = self.cp.add_utf8(sig);
                let pr_nt = self.cp.add_name_and_type(pr_name, pr_t);
                let method_println = self.cp.add_method_ref(ps_c, pr_nt);

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
                match op.as_str() {
                    "+" => self.bytecode.push(0x60), // iadd
                    "-" => self.bytecode.push(0x64), // isub
                    "*" => self.bytecode.push(0x68), // imul
                    "/" => self.bytecode.push(0x6C), // idiv
                    _ => {}
                }
            }
        }
    }
}