mod core;
mod reader;
mod parser;
mod compiler;

use std::fs;

fn main() -> std::io::Result<()> {
    // 1. Ahora probamos con un print, que es lo que el compilador ya entiende
    let source_code = r#"
        print("Hola desde un archivo .kj real!")
    "#;

    println!("🔨 Parseando código Kujav...");
    let ast = parser::parse_to_ast(source_code);

    let mut kujav_compiler = compiler::codegen::Compiler::new();
    for stmt in ast {
        kujav_compiler.compile_statement(stmt);
    }

    // Agregamos el return final para que la JVM no falle
    kujav_compiler.bytecode.push(0xB1); 

    // 2. ESCRIBIR EL ARCHIVO FINAL (Esto quita los warnings de to_bytes y file)
    let mut file = fs::File::create("Salida.class")?;
    
    // Estructura oficial de la JVM (Fase A)
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE])?; // Magic
    file.write_all(&[0x00, 0x00, 0x00, 0x34])?; // Java 8
    file.write_all(&kujav_compiler.cp.to_bytes())?; // Constant Pool dinámico
    file.write_all(&[0x00, 0x21])?; // Public Class
    
    // Aquí usamos índices fijos por ahora para Salida heredando de Object
    file.write_all(&[0x00, 0x02])?; // This Class (basado en el pool)
    file.write_all(&[0x00, 0x04])?; // Super Class
    
    file.write_all(&[0x00, 0x00])?; // Interfaces
    file.write_all(&[0x00, 0x00])?; // Fields
    
    // MÉTODOS (main)
    file.write_all(&[0x00, 0x01])?; // 1 método
    file.write_all(&[0x00, 0x09])?; // Public Static
    file.write_all(&[0x00, 0x0D])?; // nombre "main" (debe coincidir con tu pool)
    file.write_all(&[0x00, 0x0E])?; // tipo "([Ljava/lang/String;)V"
    file.write_all(&[0x00, 0x01])?; // 1 Atributo: Code

    // ATRIBUTO CODE
    let code_attr_name = 15; // índice de "Code" en el pool
    file.write_all(&[0x00, 0x0F])?; 
    let attr_len: u32 = 12 + kujav_compiler.bytecode.len() as u32;
    file.write_all(&attr_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x02, 0x00, 0x01])?; // stacks y locals
    file.write_all(&(kujav_compiler.bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&kujav_compiler.bytecode)?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?; // Final

    println!("✅ ¡Salida.class generada con éxito!");
    Ok(())
}