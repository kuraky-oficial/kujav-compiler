// src/compiler/types.rs
#[derive(Debug, Clone, PartialEq)]
pub enum KType {
    Int,
    String,
    Bool,
    Void,
    Array(Box<KType>), // <--- Nuevo: Tipo compuesto
}

impl KType {
    pub fn to_jvm_sig(&self) -> String {
        match self {
            KType::Int => "I".into(),
            KType::String => "Ljava/lang/String;".into(),
            KType::Bool => "Z".into(),
            KType::Void => "V".into(),
            KType::Array(t) => format!("[{}", t.to_jvm_sig()), // [I para arreglo de Ints
        }
    }
}