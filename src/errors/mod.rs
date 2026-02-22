use std::fmt::{Display, Formatter};

pub type KujavResult<T> = Result<T, KujavError>;

#[derive(Debug)]
pub enum KujavError {
    Syntax(String),
    Semantic(String),
    Type(String),
    Dependency(String),
    Bytecode(String),
    Io(String),
    Toml(String),
}

impl KujavError {
    pub fn syntax(msg: impl Into<String>) -> Self {
        Self::Syntax(msg.into())
    }
    pub fn semantic(msg: impl Into<String>) -> Self {
        Self::Semantic(msg.into())
    }
    pub fn type_error(msg: impl Into<String>) -> Self {
        Self::Type(msg.into())
    }
    pub fn dependency(msg: impl Into<String>) -> Self {
        Self::Dependency(msg.into())
    }
    pub fn bytecode(msg: impl Into<String>) -> Self {
        Self::Bytecode(msg.into())
    }
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }
    pub fn toml(msg: impl Into<String>) -> Self {
        Self::Toml(msg.into())
    }
}

impl From<std::io::Error> for KujavError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl Display for KujavError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KujavError::Syntax(m) => write!(f, "Error[SyntaxError]:\n{m}"),
            KujavError::Semantic(m) => write!(f, "Error[SemanticError]:\n{m}"),
            KujavError::Type(m) => write!(f, "Error[TypeError]:\n{m}"),
            KujavError::Dependency(m) => write!(f, "Error[DependencyError]:\n{m}"),
            KujavError::Bytecode(m) => write!(f, "Error[BytecodeError]:\n{m}"),
            KujavError::Io(m) => write!(f, "Error[IoError]:\n{m}"),
            KujavError::Toml(m) => write!(f, "Error[TomlError]:\n{m}"),
        }
    }
}
