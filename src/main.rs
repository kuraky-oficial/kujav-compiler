mod core;
mod reader;
mod parser;
mod compiler;

use std::fs;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let source_code = r#"
        let x = 10
        if x == 10 {
            print "Es diez!"
        }
        print x * 5 / 2
    "#;

    println!("🔨 Parseando código Kujav...");
    let ast = parser::parse_to_ast(source_code);
    let mut kujav_compiler = compiler::codegen::Compiler::new();
    
    let cls_u = kujav_compiler.cp.add_utf8("Salida");
    let this_c = kujav_compiler.cp.add_class(cls_u);
    let obj_u = kujav_compiler.cp.add_utf8("java/lang/Object");
    let super_c = kujav_compiler.cp.add_class(obj_u);
    let m_n = kujav_compiler.cp.add_utf8("main");
    let m_t = kujav_compiler.cp.add_utf8("([Ljava/lang/String;)V");
    let c_a = kujav_compiler.cp.add_utf8("Code");

    for stmt in ast {
        kujav_compiler.compile_statement(stmt);
    }
    kujav_compiler.bytecode.push(0xB1); 

    let mut file = fs::File::create("Salida.class")?;
    // CAMBIO: Usamos 0x32 (Java 6) para evitar el StackMapTable
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x32])?;
    file.write_all(&kujav_compiler.cp.to_bytes())?;
    file.write_all(&[0x00, 0x21])?;
    file.write_all(&this_c.to_be_bytes())?; 
    file.write_all(&super_c.to_be_bytes())?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x09])?;
    file.write_all(&m_n.to_be_bytes())?; 
    file.write_all(&m_t.to_be_bytes())?; 
    file.write_all(&[0x00, 0x01])?; 

    file.write_all(&c_a.to_be_bytes())?; 
    let attr_len: u32 = 12 + kujav_compiler.bytecode.len() as u32;
    file.write_all(&attr_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x0A])?; // max_stack aumentado por seguridad
    file.write_all(&(kujav_compiler.next_slot as u16).to_be_bytes())?; 
    file.write_all(&(kujav_compiler.bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&kujav_compiler.bytecode)?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?; 

    println!("✅ ¡Salida.class generada con éxito!");
    Ok(())
}