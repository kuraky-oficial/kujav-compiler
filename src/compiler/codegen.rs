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
            Stmt::If(condition, if_body, else_body) => {
                self.compile_expression(condition);
                
                // ifeq (0x99): si es falso, salta al ELSE o al FINAL
                self.bytecode.push(0x99);
                let jump_to_else_idx = self.bytecode.len();
                self.bytecode.extend_from_slice(&[0x00, 0x00]); 

                for s in if_body {
                    self.compile_statement(s);
                }

                if let Some(else_stmts) = else_body {
                    // Al terminar el IF, saltamos el bloque ELSE
                    self.bytecode.push(0xA7); // goto
                    let jump_to_end_idx = self.bytecode.len();
                    self.bytecode.extend_from_slice(&[0x00, 0x00]);

                    // Parcheamos el salto inicial para que venga aquí
                    let offset_to_else = (self.bytecode.len() - (jump_to_else_idx - 1)) as i16;
                    let b = offset_to_else.to_be_bytes();
                    self.bytecode[jump_to_else_idx] = b[0];
                    self.bytecode[jump_to_else_idx + 1] = b[1];

                    for s in else_stmts {
                        self.compile_statement(s);
                    }

                    // Parcheamos el salto final del bloque IF
                    let offset_to_end = (self.bytecode.len() - (jump_to_end_idx - 1)) as i16;
                    let b_end = offset_to_end.to_be_bytes();
                    self.bytecode[jump_to_end_idx] = b_end[0];
                    self.bytecode[jump_to_end_idx + 1] = b_end[1];
                } else {
                    // No hay else, saltamos directo al final
                    let offset = (self.bytecode.len() - (jump_to_else_idx - 1)) as i16;
                    let b = offset.to_be_bytes();
                    self.bytecode[jump_to_else_idx] = b[0];
                    self.bytecode[jump_to_else_idx + 1] = b[1];
                }
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
                    "<"  => self.bytecode.extend_from_slice(&[0xA1, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                    ">"  => self.bytecode.extend_from_slice(&[0xA2, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                    _ => {}
                }
            }
        }
    }
}