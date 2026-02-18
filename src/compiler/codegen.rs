use crate::core::constant_pool::ConstantPool;
use crate::parser::ast::{Stmt, Expr};

pub struct Compiler {
    pub cp: ConstantPool,
    pub bytecode: Vec<u8>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            cp: ConstantPool::new(),
            bytecode: Vec::new(),
        }
    }

    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Let(name, expr) => {
                // Aquí traducirás la lógica de asignar variables
                println!("Compilando variable: {}", name);
            }
            Stmt::Print(expr) => {
                // Aquí usarás los opcodes 0xB2 (getstatic) y 0xB6 (invokevirtual)
                // que ya probaste en tu main.rs
            }
            _ => {}
        }
    }
}