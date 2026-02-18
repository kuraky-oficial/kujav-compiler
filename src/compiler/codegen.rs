// src/compiler/codegen.rs
use std::collections::HashMap;
use crate::core::constant_pool::ConstantPool;
use crate::parser::ast::{Stmt, Expr};

// Estructura para guardar cada funcion como un metodo de Java
pub struct MethodInfo {
    pub name: String,
    pub bytecode: Vec<u8>,
    pub max_locals: u16,
}

pub struct Compiler {
    pub cp: ConstantPool,
    pub methods: Vec<MethodInfo>,        // Lista de funciones (fun)
    pub current_bytecode: Vec<u8>,       // El codigo que se esta escribiendo ahora
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
            next_slot: 1, // Slot 0 reservado para args en main
        }
    }

    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Function(name, params, body) => {
                // 1. Guardar el estado de lo que estabamos haciendo (main)
                let old_bc = std::mem::take(&mut self.current_bytecode);
                let old_vars = std::mem::take(&mut self.variables);
                let old_types = std::mem::take(&mut self.variable_types);
                let old_slot = self.next_slot;

                // 2. Preparar el entorno de la nueva funcion
                self.next_slot = 0;
                for p in params {
                    self.variables.insert(p.clone(), self.next_slot);
                    self.variable_types.insert(p, "I".to_string());
                    self.next_slot += 1;
                }

                // 3. Compilar el cuerpo de la funcion
                for s in body { self.compile_statement(s); }
                self.current_bytecode.push(0xB1); // return obligatorio al final

                // 4. Guardar la funcion terminada
                self.methods.push(MethodInfo {
                    name,
                    bytecode: std::mem::take(&mut self.current_bytecode),
                    max_locals: self.next_slot as u16,
                });

                // 5. Restaurar el estado anterior para seguir con main
                self.current_bytecode = old_bc;
                self.variables = old_vars;
                self.variable_types = old_types;
                self.next_slot = old_slot;
            }

            Stmt::Call(name, args) => {
                // Cargar argumentos en la pila
                for arg in args { self.compile_expression(arg); }
                
                let cls = self.cp.add_class(self.cp.add_utf8("Salida"));
                // Por ahora asumimos que no devuelven nada ()V
                let nt = self.cp.add_name_and_type(self.cp.add_utf8(&name), self.cp.add_utf8("()V"));
                let m_ref = self.cp.add_method_ref(cls, nt);
                
                self.current_bytecode.push(0xB8); // invokestatic
                self.current_bytecode.extend_from_slice(&m_ref.to_be_bytes());
            }

            Stmt::Let(name, expr) => {
                let slot = if let Some(&s) = self.variables.get(&name) { s } else {
                    let s = self.next_slot; self.variables.insert(name.clone(), s);
                    self.next_slot += 1; s
                };
                let type_tag = match &expr { Expr::String(_) => "Ljava/lang/String;", _ => "I" };
                self.variable_types.insert(name.clone(), type_tag.to_string());
                self.compile_expression(expr);
                self.current_bytecode.push(if type_tag == "I" { 0x36 } else { 0x3A });
                self.current_bytecode.push(slot);
            }

            Stmt::Print(expr) => {
                let sys_n = self.cp.add_utf8("java/lang/System");
                let sys_c = self.cp.add_class(sys_n);
                let out_nt = self.cp.add_name_and_type(self.cp.add_utf8("out"), self.cp.add_utf8("Ljava/io/PrintStream;"));
                let f_out = self.cp.add_field_ref(sys_c, out_nt);
                self.current_bytecode.push(0xB2); 
                self.current_bytecode.extend_from_slice(&f_out.to_be_bytes());

                let sig = match &expr {
                    Expr::String(_) => "(Ljava/lang/String;)V",
                    Expr::Identifier(n) => if self.variable_types.get(n).map(|s| s.as_str()) == Some("I") { "(I)V" } else { "(Ljava/lang/String;)V" },
                    _ => "(I)V",
                };

                self.compile_expression(expr);
                let ps_c = self.cp.add_class(self.cp.add_utf8("java/io/PrintStream"));
                let pr_nt = self.cp.add_name_and_type(self.cp.add_utf8("println"), self.cp.add_utf8(sig));
                let m_pr = self.cp.add_method_ref(ps_c, pr_nt);
                self.current_bytecode.push(0xB6); 
                self.current_bytecode.extend_from_slice(&m_pr.to_be_bytes());
            }

            Stmt::If(condition, if_body, else_body) => {
                self.compile_expression(condition);
                let opcode_pos = self.current_bytecode.len();
                self.current_bytecode.push(0x99); // ifeq
                let jump_to_else_idx = self.current_bytecode.len();
                self.current_bytecode.extend_from_slice(&[0x00, 0x00]); 

                for s in if_body { self.compile_statement(s); }

                if let Some(else_stmts) = else_body {
                    let goto_pos = self.current_bytecode.len();
                    self.current_bytecode.push(0xA7); // goto end
                    let jump_to_end_idx = self.current_bytecode.len();
                    self.current_bytecode.extend_from_slice(&[0x00, 0x00]);

                    let offset_to_else = (self.current_bytecode.len() - opcode_pos) as i16;
                    let b = offset_to_else.to_be_bytes();
                    self.current_bytecode[jump_to_else_idx] = b[0];
                    self.current_bytecode[jump_to_else_idx + 1] = b[1];

                    for s in else_stmts { self.compile_statement(s); }

                    let offset_to_end = (self.current_bytecode.len() - goto_pos) as i16;
                    let b_end = offset_to_end.to_be_bytes();
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
            Expr::Number(val) => {
                self.current_bytecode.push(0x10); 
                self.current_bytecode.push(val as u8);
            }
            Expr::String(content) => {
                let s_idx = self.cp.add_string(self.cp.add_utf8(&content));
                self.current_bytecode.push(0x12); 
                self.current_bytecode.push(s_idx as u8);
            }
            Expr::Identifier(name) => {
                if let Some(&slot) = self.variables.get(&name) {
                    let is_i = self.variable_types.get(&name).map(|s| s.as_str()) == Some("I");
                    self.current_bytecode.push(if is_i { 0x15 } else { 0x19 });
                    self.current_bytecode.push(slot);
                }
            }
            Expr::Binary(l, op, r) => {
                self.compile_expression(*l);
                self.compile_expression(*r);
                match op.as_str() {
                    "+" => self.current_bytecode.push(0x60),
                    "-" => self.current_bytecode.push(0x64),
                    "*" => self.current_bytecode.push(0x68),
                    "/" => self.current_bytecode.push(0x6C),
                    "==" => self.current_bytecode.extend_from_slice(&[0xA0, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                    "<"  => self.current_bytecode.extend_from_slice(&[0xA2, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                    ">"  => self.current_bytecode.extend_from_slice(&[0xA4, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                    _ => {}
                }
            }
        }
    }
}