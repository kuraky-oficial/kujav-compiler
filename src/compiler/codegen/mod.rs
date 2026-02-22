// src/compiler/codegen/mod.rs
pub mod expressions;
pub mod statements;

use crate::core::constant_pool::ConstantPool;
use std::collections::HashMap;

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
}
