mod core;
mod reader;
mod parser;
mod compiler;

use std::fs;

fn main() -> std::io::Result<()> {
    // 1. Leer archivo fuente
    let source_code = r#"
        let mensaje = "Hola desde el archivo .kj"
    "#;

    // 2. Convertir a AST
    println!("🔨 Parseando código Kujav...");
    let ast = parser::parse_to_ast(source_code);

    // 3. Compilar (Fase en desarrollo)
    let mut kujav_compiler = compiler::codegen::Compiler::new();
    for stmt in ast {
        kujav_compiler.compile_statement(stmt);
    }

    println!("✅ Compilación finalizada.");
    Ok(())
}