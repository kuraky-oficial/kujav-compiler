// src/compiler/codegen.rs
use std::collections::HashMap;
use crate::core::constant_pool::ConstantPool;
use crate::parser::ast::{Stmt, Expr};

pub struct Compiler {
    pub cp: ConstantPool,
    pub bytecode: Vec<u8>,
    pub variables: HashMap<String, u8>,
    pub next_slot: u8, 
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            cp: ConstantPool::new(),
            bytecode: Vec::new(),
            variables: HashMap::new(),
            next_slot: 1, 
        }
    }

    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Let(name, expr) => {
                let slot = self.next_slot;
                self.variables.insert(name.clone(), slot);
                self.next_slot += 1;

                self.compile_expression(expr);
                self.bytecode.push(0x36); // istore (guardar entero)
                self.bytecode.push(slot);
                println!("📝 Variable numérica '{}' en slot {}", name, slot);
            }
            Stmt::Print(expr) => {
                let sys_cls = self.cp.add_class(self.cp.add_utf8("java/lang/System"));
                let out_nt = self.cp.add_name_and_type(self.cp.add_utf8("out"), self.cp.add_utf8("Ljava/io/PrintStream;"));
                let field_out = self.cp.add_field_ref(sys_cls, out_nt);

                let ps_cls = self.cp.add_class(self.cp.add_utf8("java/io/PrintStream"));
                
                // IMPORTANTE: println para enteros usa la firma (I)V
                let println_nt = self.cp.add_name_and_type(self.cp.add_utf8("println"), self.cp.add_utf8("(I)V"));
                let method_println = self.cp.add_method_ref(ps_cls, println_nt);

                self.bytecode.push(0xB2); // getstatic
                self.bytecode.extend_from_slice(&field_out.to_be_bytes());

                self.compile_expression(expr);

                self.bytecode.push(0xB6); // invokevirtual
                self.bytecode.extend_from_slice(&method_println.to_be_bytes());
            }
            _ => {}
        }
    }

    fn compile_expression(&mut self, expr: Expr) {
        match expr {
            Expr::Number(val) => {
                self.bytecode.push(0x10); // bipush (para números pequeños)
                self.bytecode.push(val as u8);
            }
            Expr::Identifier(name) => {
                if let Some(&slot) = self.variables.get(&name) {
                    self.bytecode.push(0x15); // iload
                    self.bytecode.push(slot);
                }
            }
            Expr::Binary(left, op, right) => {
                self.compile_expression(*left);
                self.compile_expression(*right);
                if op == "+" {
                    self.bytecode.push(0x60); // iadd
                }
            }
            _ => {}
        }
    }
}