// src/main.rs
mod core;
mod reader;
mod parser;
mod compiler;

use std::fs;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let source_code = r#"
        let x = 10
        let y = 5
        print(x + y)
    "#;

    println!("🔨 Parseando código Kujav...");
    let ast = parser::parse_to_ast(source_code);
    let mut kujav_compiler = compiler::codegen::Compiler::new();
    
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
    kujav_compiler.bytecode.push(0xB1); 

    let mut file = fs::File::create("Salida.class")?;
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE])?;
    file.write_all(&[0x00, 0x00, 0x00, 0x34])?;
    file.write_all(&kujav_compiler.cp.to_bytes())?;
    
    file.write_all(&[0x00, 0x21])?;
    file.write_all(&this_class.to_be_bytes())?; 
    file.write_all(&super_class.to_be_bytes())?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; 
    
    file.write_all(&[0x00, 0x01, 0x00, 0x09])?; 
    file.write_all(&main_name.to_be_bytes())?; 
    file.write_all(&main_type.to_be_bytes())?; 
    file.write_all(&[0x00, 0x01])?; 

    file.write_all(&code_attr.to_be_bytes())?; 
    let attr_len: u32 = 12 + kujav_compiler.bytecode.len() as u32;
    file.write_all(&attr_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x02])?; // max_stack
    file.write_all(&(kujav_compiler.next_slot as u16).to_be_bytes())?; // max_locals DINÁMICO
    
    file.write_all(&(kujav_compiler.bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&kujav_compiler.bytecode)?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?; 

    println!("✅ ¡Salida.class generada con éxito!");
    Ok(())
}