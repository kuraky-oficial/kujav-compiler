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

                if let Expr::String(content) = expr {
                    let msg_utf8 = self.cp.add_utf8(&content);
                    let s_idx = self.cp.add_string(msg_utf8);
                    
                    self.bytecode.push(0x12); // ldc
                    self.bytecode.push(s_idx as u8);
                    self.bytecode.push(0x3A); // astore
                    self.bytecode.push(slot);
                    println!("📝 Variable '{}' guardada en el slot {}", name, slot);
                }
            }
            Stmt::Print(expr) => {
                let sys_utf8 = self.cp.add_utf8("java/lang/System");
                let sys_cls = self.cp.add_class(sys_utf8);
                let out_name = self.cp.add_utf8("out");
                let out_type = self.cp.add_utf8("Ljava/io/PrintStream;");
                let out_nt = self.cp.add_name_and_type(out_name, out_type);
                let field_out = self.cp.add_field_ref(sys_cls, out_nt);

                let ps_utf8 = self.cp.add_utf8("java/io/PrintStream");
                let ps_cls = self.cp.add_class(ps_utf8);
                let println_name = self.cp.add_utf8("println");
                let println_type = self.cp.add_utf8("(Ljava/lang/String;)V");
                let println_nt = self.cp.add_name_and_type(println_name, println_type);
                let method_println = self.cp.add_method_ref(ps_cls, println_nt);

                self.bytecode.push(0xB2); // getstatic
                self.bytecode.extend_from_slice(&field_out.to_be_bytes());

                match expr {
                    Expr::String(content) => {
                        let msg_utf8 = self.cp.add_utf8(&content);
                        let s_idx = self.cp.add_string(msg_utf8);
                        self.bytecode.push(0x12); // ldc
                        self.bytecode.push(s_idx as u8);
                    }
                    Expr::Identifier(name) => {
                        if let Some(&slot) = self.variables.get(&name) {
                            self.bytecode.push(0x19); // aload
                            self.bytecode.push(slot);
                        }
                    }
                    _ => {}
                }

                self.bytecode.push(0xB6); // invokevirtual
                self.bytecode.extend_from_slice(&method_println.to_be_bytes());
            }
            _ => {}
        }
    }
}