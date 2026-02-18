pub enum Expr {
    Number(i32),
    String(String),
    Identifier(String),
}

pub enum Stmt {
    Let(String, Expr),
    Print(Expr),
    Function(String, Vec<String>, Vec<Stmt>),
}