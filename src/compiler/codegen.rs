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
                    Expr::String(_) => "Ljava/lang/String;".to_string(),
                    _ => "I".to_string(),
                };
                self.variable_types.insert(name.clone(), type_tag.clone());
                self.compile_expression(expr);
                if type_tag == "I" { self.bytecode.push(0x36); } else { self.bytecode.push(0x3A); }
                self.bytecode.push(slot);
            }
            Stmt::Print(expr) => {
                let sys_n = self.cp.add_utf8("java/lang/System");
                let sys_c = self.cp.add_class(sys_n);
                let out_n = self.cp.add_utf8("out");
                let out_t = self.cp.add_utf8("Ljava/io/PrintStream;");
                let out_nt = self.cp.add_name_and_type(out_n, out_t);
                let f_out = self.cp.add_field_ref(sys_c, out_nt);
                self.bytecode.push(0xB2); 
                self.bytecode.extend_from_slice(&f_out.to_be_bytes());

                let (sig, _) = match &expr {
                    Expr::String(_) => ("(Ljava/lang/String;)V", true),
                    Expr::Identifier(n) => {
                        let t = self.variable_types.get(n).cloned().unwrap_or("I".to_string());
                        (if t == "I" { "(I)V" } else { "(Ljava/lang/String;)V" }, t != "I")
                    }
                    _ => ("(I)V", false),
                };

                self.compile_expression(expr);
                let ps_n = self.cp.add_utf8("java/io/PrintStream");
                let ps_c = self.cp.add_class(ps_n);
                let pr_n = self.cp.add_utf8("println");
                let pr_t = self.cp.add_utf8(sig);
                let pr_nt = self.cp.add_name_and_type(pr_n, pr_t);
                let m_pr = self.cp.add_method_ref(ps_c, pr_nt);
                self.bytecode.push(0xB6); 
                self.bytecode.extend_from_slice(&m_pr.to_be_bytes());
            }
            Stmt::If(condition, body) => {
                self.compile_expression(condition);
                self.bytecode.push(0x99); // ifeq
                let jump_idx = self.bytecode.len();
                self.bytecode.extend_from_slice(&[0x00, 0x00]); 

                for s in body {
                    self.compile_statement(s);
                }

                let offset = (self.bytecode.len() - (jump_idx - 1)) as i16;
                let offset_bytes = offset.to_be_bytes();
                self.bytecode[jump_idx] = offset_bytes[0];
                self.bytecode[jump_idx + 1] = offset_bytes[1];
            }
            _ => {}
        }
    }

    fn compile_expression(&mut self, expr: Expr) {
        match expr {
            Expr::Number(val) => {
                self.bytecode.push(0x10); 
                self.bytecode.push(val as u8);
            }
            Expr::String(content) => {
                let s_utf8 = self.cp.add_utf8(&content);
                let s_idx = self.cp.add_string(s_utf8);
                self.bytecode.push(0x12); 
                self.bytecode.push(s_idx as u8);
            }
            Expr::Identifier(name) => {
                if let Some(&slot) = self.variables.get(&name) {
                    let t = self.variable_types.get(&name).unwrap();
                    if t == "I" { self.bytecode.push(0x15); } else { self.bytecode.push(0x19); }
                    self.bytecode.push(slot);
                }
            }
            Expr::Binary(l, op, r) => {
                self.compile_expression(*l);
                self.compile_expression(*r);
                match op.as_str() {
                    "+" => self.bytecode.push(0x60),
                    "-" => self.bytecode.push(0x64),
                    "*" => self.bytecode.push(0x68),
                    "/" => self.bytecode.push(0x6C),
                    "==" => self.bytecode.extend_from_slice(&[0xA0, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                    _ => {}
                }
            }
        }
    }
}