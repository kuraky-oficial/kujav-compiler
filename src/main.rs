// src/main.rs
mod core; mod reader; mod parser; mod compiler;
use std::fs;
use std::io::Write;
use crate::compiler::semantics::SemanticAnalyzer;

fn main() -> std::io::Result<()> {
    let source_code = r#"
        let lista = [10, 20, 30]
        let x = lista[1]
        print "El valor es: " + x
    "#;

    println!("🔨 Compilando Kujav...");
    let ast = parser::parse_to_ast(source_code);

    // FASE 1: Análisis Semántico con el nuevo objeto
    let mut analyzer = SemanticAnalyzer::new();
    for stmt in &ast {
        if let Err(e) = analyzer.check_stmt(stmt) {
            eprintln!("❌ Error Semántico: {}", e);
            std::process::exit(1);
        }
    }

    // 2. GENERACIÓN DE CÓDIGO
    let mut kujav = compiler::codegen::Compiler::new();
    
    let cls_u = kujav.cp.add_utf8("Salida");
    let this_c = kujav.cp.add_class(cls_u);
    let obj_u = kujav.cp.add_utf8("java/lang/Object");
    let super_c = kujav.cp.add_class(obj_u);
    let m_n = kujav.cp.add_utf8("main");
    let m_t = kujav.cp.add_utf8("([Ljava/lang/String;)V");
    let c_a = kujav.cp.add_utf8("Code");

    for stmt in ast { 
        kujav.compile_statement(stmt);
    }
    kujav.current_bytecode.push(0xB1); 

    let mut file = fs::File::create("Salida.class")?;
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x31])?; 
    file.write_all(&kujav.cp.to_bytes())?;
    file.write_all(&[0x00, 0x21])?;
    file.write_all(&this_c.to_be_bytes())?; file.write_all(&super_c.to_be_bytes())?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; 

    let num_methods = (1 + kujav.methods.len()) as u16;
    file.write_all(&num_methods.to_be_bytes())?;

    // Main
    file.write_all(&[0x00, 0x09])?; 
    file.write_all(&m_n.to_be_bytes())?; file.write_all(&m_t.to_be_bytes())?;
    file.write_all(&[0x00, 0x01])?; file.write_all(&c_a.to_be_bytes())?;
    let main_len: u32 = 12 + kujav.current_bytecode.len() as u32;
    file.write_all(&main_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x0A, 0x00, 0x0A])?; 
    file.write_all(&(kujav.current_bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&kujav.current_bytecode)?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?;

    for m in &kujav.methods {
        file.write_all(&[0x00, 0x09])?; 
        file.write_all(&m.name_idx.to_be_bytes())?; file.write_all(&m.sig_idx.to_be_bytes())?;
        file.write_all(&[0x00, 0x01])?; file.write_all(&c_a.to_be_bytes())?;
        let attr_len: u32 = 12 + m.bytecode.len() as u32;
        file.write_all(&attr_len.to_be_bytes())?;
        file.write_all(&[0x00, 0x0A])?; file.write_all(&m.max_locals.to_be_bytes())?;
        file.write_all(&(m.bytecode.len() as u32).to_be_bytes())?;
        file.write_all(&m.bytecode)?;
        file.write_all(&[0x00, 0x00, 0x00, 0x00])?;
    }
    file.write_all(&[0x00, 0x00])?; 
    println!("✅ Salida.class generada con éxito.");
    Ok(())
}