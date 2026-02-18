// src/compiler/types.rs
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum KType {
    Int,
    String,
    Bool,
    Void,
    Array(Box<KType>),
}

impl KType {
    // NUEVO: Ayuda al codegen a decidir entre instrucciones 'i' o 'a'
    pub fn is_reference(&self) -> bool {
        match self {
            KType::String | KType::Array(_) => true,
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn to_jvm_sig(&self) -> String {
        match self {
            KType::Int => "I".into(),
            KType::String => "Ljava/lang/String;".into(),
            KType::Bool => "Z".into(),
            KType::Void => "V".into(),
            KType::Array(t) => format!("[{}", t.to_jvm_sig()),
        }
    }
}