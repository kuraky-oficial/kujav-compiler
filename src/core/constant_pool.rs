// src/core/constant_pool.rs

pub enum Constant {
    Utf8(String),
    Class(u16), // Índice a un Utf8 con el nombre de la clase
    NameAndType { name_index: u16, type_index: u16 },
    MethodRef { class_index: u16, name_and_type_index: u16 },
}

pub struct ConstantPool {
    pub entries: Vec<Constant>,
}

impl ConstantPool {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add_utf8(&mut self, value: &str) -> u16 {
        self.entries.push(Constant::Utf8(value.to_string()));
        self.entries.len() as u16 // La JVM empieza índices en 1
    }

    pub fn add_class(&mut self, name_index: u16) -> u16 {
        self.entries.push(Constant::Class(name_index));
        self.entries.len() as u16
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Constant Pool Count (n+1)
        bytes.extend_from_slice(&(self.entries.len() as u16 + 1).to_be_bytes());
        
        for entry in &self.entries {
            match entry {
                Constant::Utf8(s) => {
                    bytes.push(1); // Tag Utf8
                    bytes.extend_from_slice(&(s.len() as u16).to_be_bytes());
                    bytes.extend_from_slice(s.as_bytes());
                }
                Constant::Class(idx) => {
                    bytes.push(7); // Tag Class
                    bytes.extend_from_slice(&idx.to_be_bytes());
                }
                _ => {} // Implementaremos los demás según sea necesario
            }
        }
        bytes
    }
}