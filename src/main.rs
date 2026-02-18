mod core;
mod reader;
mod parser;
mod compiler;

use std::fs;
use std::io::Write; // <--- ESTO ES LO QUE FALTABA PARA EL ERROR write_all

fn main() -> std::io::Result<()> {
    // 1. Código fuente de prueba
    let source_code = r#"
    let nombre = "Edwin"
    print("Hola")
"#;

    println!("🔨 Parseando código Kujav...");
    let ast = parser::parse_to_ast(source_code);

    let mut kujav_compiler = compiler::codegen::Compiler::new();
    
    // REGISTRAMOS LOS METADATOS EN EL POZO DE FORMA DINÁMICA
    // Esto evita el error de "Truncated class file" por índices incorrectos
    let cls_utf8 = kujav_compiler.cp.add_utf8("Salida");
    let this_class = kujav_compiler.cp.add_class(cls_utf8);
    let obj_utf8 = kujav_compiler.cp.add_utf8("java/lang/Object");
    let super_class = kujav_compiler.cp.add_class(obj_utf8);
    
    let main_name = kujav_compiler.cp.add_utf8("main");
    let main_type = kujav_compiler.cp.add_utf8("([Ljava/lang/String;)V");
    let code_attr = kujav_compiler.cp.add_utf8("Code");

    for stmt in ast {
        kujav_compiler.compile_statement(stmt);
    }

    // Agregamos el return final (0xB1)
    kujav_compiler.bytecode.push(0xB1); 

    // 2. ESCRIBIR EL ARCHIVO FINAL
    let mut file = fs::File::create("Salida.class")?;
    
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE])?; // Magic
    file.write_all(&[0x00, 0x00, 0x00, 0x34])?; // Java 8
    file.write_all(&kujav_compiler.cp.to_bytes())?; // Constant Pool
    
    file.write_all(&[0x00, 0x21])?; // Public
    file.write_all(&this_class.to_be_bytes())?; 
    file.write_all(&super_class.to_be_bytes())?;
    
    file.write_all(&[0x00, 0x00])?; // Interfaces
    file.write_all(&[0x00, 0x00])?; // Fields
    
    // MÉTODOS
    file.write_all(&[0x00, 0x01])?; // 1 método
    file.write_all(&[0x00, 0x09])?; // Public Static
    file.write_all(&main_name.to_be_bytes())?; 
    file.write_all(&main_type.to_be_bytes())?; 
    file.write_all(&[0x00, 0x01])?; // 1 Atributo: Code

    // ATRIBUTO CODE
    file.write_all(&code_attr.to_be_bytes())?; 
    let attr_len: u32 = 12 + kujav_compiler.bytecode.len() as u32;
    file.write_all(&attr_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x02, 0x00, 0x01])?; // stacks y locals
    file.write_all(&(kujav_compiler.bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&kujav_compiler.bytecode)?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; // Exceptions y Atributos de método
    file.write_all(&[0x00, 0x00])?; // Atributos de clase

    println!("✅ ¡Salida.class generada con éxito!");
    Ok(())
}