// src/core/constant_pool.rs

pub enum Constant {
    Utf8(String),
    Class(u16),
    String(u16),
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        type_index: u16,
    },
}

pub struct ConstantPool {
    pub entries: Vec<Constant>,
}

impl ConstantPool {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_utf8(&mut self, value: &str) -> u16 {
        self.entries.push(Constant::Utf8(value.to_string()));
        self.entries.len() as u16
    }

    pub fn add_class(&mut self, name_idx: u16) -> u16 {
        self.entries.push(Constant::Class(name_idx));
        self.entries.len() as u16
    }

    pub fn add_string(&mut self, utf8_idx: u16) -> u16 {
        self.entries.push(Constant::String(utf8_idx));
        self.entries.len() as u16
    }

    pub fn add_name_and_type(&mut self, name_idx: u16, type_idx: u16) -> u16 {
        self.entries.push(Constant::NameAndType {
            name_index: name_idx,
            type_index: type_idx,
        });
        self.entries.len() as u16
    }

    pub fn add_field_ref(&mut self, class_idx: u16, nt_idx: u16) -> u16 {
        self.entries.push(Constant::FieldRef {
            class_index: class_idx,
            name_and_type_index: nt_idx,
        });
        self.entries.len() as u16
    }

    pub fn add_method_ref(&mut self, class_idx: u16, nt_idx: u16) -> u16 {
        self.entries.push(Constant::MethodRef {
            class_index: class_idx,
            name_and_type_index: nt_idx,
        });
        self.entries.len() as u16
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.entries.len() as u16 + 1).to_be_bytes());
        for entry in &self.entries {
            match entry {
                Constant::Utf8(s) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&(s.len() as u16).to_be_bytes());
                    bytes.extend_from_slice(s.as_bytes());
                }
                Constant::Class(i) => {
                    bytes.push(7);
                    bytes.extend_from_slice(&i.to_be_bytes());
                }
                Constant::String(i) => {
                    bytes.push(8);
                    bytes.extend_from_slice(&i.to_be_bytes());
                }
                Constant::FieldRef {
                    class_index,
                    name_and_type_index,
                } => {
                    bytes.push(9);
                    bytes.extend_from_slice(&class_index.to_be_bytes());
                    bytes.extend_from_slice(&name_and_type_index.to_be_bytes());
                }
                Constant::MethodRef {
                    class_index,
                    name_and_type_index,
                } => {
                    bytes.push(10);
                    bytes.extend_from_slice(&class_index.to_be_bytes());
                    bytes.extend_from_slice(&name_and_type_index.to_be_bytes());
                }
                Constant::NameAndType {
                    name_index,
                    type_index,
                } => {
                    bytes.push(12);
                    bytes.extend_from_slice(&name_index.to_be_bytes());
                    bytes.extend_from_slice(&type_index.to_be_bytes());
                }
            }
        }
        bytes
    }
}
