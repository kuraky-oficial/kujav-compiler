// src/compiler/codegen.rs
use std::collections::HashMap;
use crate::core::constant_pool::ConstantPool;
use crate::parser::ast::{Stmt, Expr};

pub struct MethodInfo {
    pub name_idx: u16,
    pub sig_idx: u16,
    pub bytecode: Vec<u8>,
    pub max_locals: u16,
}

pub struct Compiler {
    pub cp: ConstantPool,
    pub methods: Vec<MethodInfo>,        
    pub current_bytecode: Vec<u8>,       
    pub variables: HashMap<String, u8>,
    pub variable_types: HashMap<String, String>, 
    pub next_slot: u8, 
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            cp: ConstantPool::new(),
            methods: Vec::new(),
            current_bytecode: Vec::new(),
            variables: HashMap::new(),
            variable_types: HashMap::new(),
            next_slot: 1, 
        }
    }

    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Function(name, params, body, return_type) => {
                // CORRECCIÓN: Si no se especifica, asumimos "I" para que el return funcione
                let ret_sig = match return_type.as_deref() {
                    Some("S") => "Ljava/lang/String;",
                    Some("V") => "V",
                    _ => "I", 
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
                    self.variables.insert(p.clone(), self.next_slot);
                    self.variable_types.insert(p, "I".to_string());
                    self.next_slot += 1;
                }

                for s in body { self.compile_statement(s); }
                
                // Si al final no hubo return y es void, lo agregamos
                if ret_sig == "V" { self.current_bytecode.push(0xB1); }

                self.methods.push(MethodInfo {
                    name_idx: n_idx,
                    sig_idx: s_idx,
                    bytecode: std::mem::take(&mut self.current_bytecode),
                    max_locals: self.next_slot as u16,
                });

                self.current_bytecode = old_bc;
                self.variables = old_vars;
                self.variable_types = old_types;
                self.next_slot = old_slot;
            }

            Stmt::Return(expr) => {
                let is_str = matches!(expr, Expr::String(_));
                self.compile_expression(expr);
                self.current_bytecode.push(if is_str { 0xB0 } else { 0xAC });
            }

            Stmt::Call(name, args) => {
                // Si se llama como instrucción suelta, asumimos firma void ()V
                for arg in &args { self.compile_expression(arg.clone()); }
                let cls_u = self.cp.add_utf8("Salida");
                let cls = self.cp.add_class(cls_u);
                let n_u = self.cp.add_utf8(&name);
                
                let mut p_sigs = String::new();
                for _ in 0..args.len() { p_sigs.push('I'); }
                let sig = format!("({})V", p_sigs); 
                
                let s_u = self.cp.add_utf8(&sig);
                let nt = self.cp.add_name_and_type(n_u, s_u);
                let m_ref = self.cp.add_method_ref(cls, nt);
                self.current_bytecode.push(0xB8); 
                self.current_bytecode.extend_from_slice(&m_ref.to_be_bytes());
            }

            Stmt::Let(name, expr) => {
                let slot = if let Some(&s) = self.variables.get(&name) { s } else {
                    let s = self.next_slot; self.variables.insert(name.clone(), s);
                    self.next_slot += 1; s
                };
                self.variable_types.insert(name, "I".to_string());
                self.compile_expression(expr);
                self.current_bytecode.push(0x36); // istore
                self.current_bytecode.push(slot);
            }

            Stmt::Print(expr) => {
                let s_u = self.cp.add_utf8("java/lang/System");
                let s_c = self.cp.add_class(s_u);
                let o_u = self.cp.add_utf8("out");
                let o_t = self.cp.add_utf8("Ljava/io/PrintStream;");
                let o_nt = self.cp.add_name_and_type(o_u, o_t);
                let f_out = self.cp.add_field_ref(s_c, o_nt);
                self.current_bytecode.push(0xB2); 
                self.current_bytecode.extend_from_slice(&f_out.to_be_bytes());

                let sig = match &expr {
                    Expr::String(_) => "(Ljava/lang/String;)V",
                    _ => "(I)V",
                };

                self.compile_expression(expr);
                let p_u = self.cp.add_utf8("java/io/PrintStream");
                let p_c = self.cp.add_class(p_u);
                let pr_n = self.cp.add_utf8("println");
                let pr_s = self.cp.add_utf8(sig);
                let pr_nt = self.cp.add_name_and_type(pr_n, pr_s);
                let m_pr = self.cp.add_method_ref(p_c, pr_nt);
                self.current_bytecode.push(0xB6); 
                self.current_bytecode.extend_from_slice(&m_pr.to_be_bytes());
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
                    let b = off_else.to_be_bytes();
                    self.current_bytecode[jump_to_else_idx] = b[0];
                    self.current_bytecode[jump_to_else_idx + 1] = b[1];
                    for s in else_stmts { self.compile_statement(s); }
                    let off_end = (self.current_bytecode.len() - goto_pos) as i16;
                    let b_end = off_end.to_be_bytes();
                    self.current_bytecode[jump_to_end_idx] = b_end[0];
                    self.current_bytecode[jump_to_end_idx + 1] = b_end[1];
                } else {
                    let offset = (self.current_bytecode.len() - opcode_pos) as i16;
                    let b = offset.to_be_bytes();
                    self.current_bytecode[jump_to_else_idx] = b[0];
                    self.current_bytecode[jump_to_else_idx + 1] = b[1];
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
                let offset_to_start = (start_pos as i32 - goto_pos as i32) as i16;
                self.current_bytecode.extend_from_slice(&offset_to_start.to_be_bytes());
                let offset_to_end = (self.current_bytecode.len() - ifeq_pos) as i16;
                let b = offset_to_end.to_be_bytes();
                self.current_bytecode[jump_to_end_idx] = b[0];
                self.current_bytecode[jump_to_end_idx + 1] = b[1];
            }
        }
    }

    fn compile_expression(&mut self, expr: Expr) {
        match expr {
            Expr::Number(val) => { self.current_bytecode.push(0x10); self.current_bytecode.push(val as u8); }
            Expr::String(c) => { 
                let s_u = self.cp.add_utf8(&c);
                let s_idx = self.cp.add_string(s_u);
                self.current_bytecode.push(0x12); self.current_bytecode.push(s_idx as u8);
            }
            Expr::Identifier(name) => if let Some(&slot) = self.variables.get(&name) {
                let is_i = self.variable_types.get(&name).map(|s| s.as_str()) == Some("I");
                self.current_bytecode.push(if is_i { 0x15 } else { 0x19 });
                self.current_bytecode.push(slot);
            },
            Expr::Binary(l, op, r) => {
                self.compile_expression(*l); self.compile_expression(*r);
                match op.as_str() {
                    "+" => self.current_bytecode.push(0x60), "-" => self.current_bytecode.push(0x64),
                    "*" => self.current_bytecode.push(0x68), "/" => self.current_bytecode.push(0x6C),
                    "==" => self.current_bytecode.extend_from_slice(&[0xA0, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                    "<"  => self.current_bytecode.extend_from_slice(&[0xA2, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                    ">"  => self.current_bytecode.extend_from_slice(&[0xA4, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                    _ => {}
                }
            }
            Expr::Call(name, args) => {
                for arg in &args { self.compile_expression(arg.clone()); }
                let cls_u = self.cp.add_utf8("Salida");
                let cls = self.cp.add_class(cls_u);
                let n_u = self.cp.add_utf8(&name);
                
                let mut p_sigs = String::new();
                for _ in 0..args.len() { p_sigs.push('I'); }
                // Si se usa como EXPRESIÓN, la firma DEBE terminar en I
                let sig = format!("({})I", p_sigs); 
                
                let s_u = self.cp.add_utf8(&sig);
                let nt = self.cp.add_name_and_type(n_u, s_u);
                let m_ref = self.cp.add_method_ref(cls, nt);
                self.current_bytecode.push(0xB8); 
                self.current_bytecode.extend_from_slice(&m_ref.to_be_bytes());
            }
        }
    }
}