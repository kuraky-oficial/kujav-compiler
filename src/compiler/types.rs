// src/compiler/types.rs
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum KType {
    Int,
    String,
    Bool,
    Void,
    Array(Box<KType>),
    Custom(String), // Para clases de Java externas
}

impl KType {
    #[allow(dead_code)]
    pub fn is_reference(&self) -> bool {
        match self {
            KType::String | KType::Array(_) | KType::Custom(_) => true,
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
            KType::Custom(name) => {
                // Convertimos java.util.ArrayList a java/util/ArrayList
                let internal = name.replace('.', "/");
                format!("L{};", internal)
            }
        }
    }
}