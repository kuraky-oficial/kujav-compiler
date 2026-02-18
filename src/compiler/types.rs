// src/compiler/types.rs
#[derive(Debug, Clone, PartialEq)]
pub enum KType {
    Int,
    String,
    Bool,
    Void,
}

impl KType {
    pub fn to_jvm_sig(&self) -> String {
        match self {
            KType::Int => "I".to_string(),
            KType::String => "Ljava/lang/String;".to_string(),
            KType::Bool => "Z".to_string(),
            KType::Void => "V".to_string(),
        }
    }
}