// src/main.rs
mod core; mod reader; mod parser; mod compiler;
use std::fs;
use std::io::Write;
use crate::compiler::semantics::SemanticAnalyzer;
use crate::compiler::codegen::Compiler;

fn main() -> std::io::Result<()> {
    let source_code = r#"
        function saludar(n: Int)
            print "Número: " + n
        end

        local x: Int = 10
        while x > 0 do
            saludar(x)
            local x = x - 1
        end
    "#;

    println!("🔨 Compilando Kujav (Lua Mode)...");
    let ast = parser::parse_to_ast(source_code);

    let mut analyzer = SemanticAnalyzer::new();
    if let Err(e) = analyzer.analyze(&ast) {
        eprintln!("❌ Error Semántico: {}", e);
        std::process::exit(1);
    }

    let mut kujav = Compiler::new();
    
    // 1. REGISTRAR CONSTANTES EN EL POZO ANTES DE COMPILAR
    let cls_u = kujav.cp.add_utf8("Salida");
    let this_c = kujav.cp.add_class(cls_u);
    let obj_super_u = kujav.cp.add_utf8("java/lang/Object");
    let super_c = kujav.cp.add_class(obj_super_u);
    
    let m_n = kujav.cp.add_utf8("main");
    let m_t = kujav.cp.add_utf8("([Ljava/lang/String;)V");
    let c_a = kujav.cp.add_utf8("Code");

    // 2. COMPILAR EL AST (Esto también añade cosas al pozo)
    for stmt in ast { 
        kujav.compile_statement(stmt);
    }
    kujav.current_bytecode.push(0xB1); // return final del main

    // 3. ESCRIBIR EL ARCHIVO BINARIO
    let mut file = fs::File::create("Salida.class")?;
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x31])?; // Magic & Version
    
    // ¡Aquí se sella el Constant Pool!
    file.write_all(&kujav.cp.to_bytes())?;
    
    file.write_all(&[0x00, 0x21])?; // Access flags: ACC_PUBLIC | ACC_SUPER
    file.write_all(&this_c.to_be_bytes())?; 
    file.write_all(&super_c.to_be_bytes())?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; // Interfaces, Fields

    // Métodos
    let num_methods = (1 + kujav.methods.len()) as u16;
    file.write_all(&num_methods.to_be_bytes())?;

    // Pasamos los índices directamente (m_n, m_t, c_a)
    write_main_method(&mut file, &kujav, m_n, m_t, c_a)?;

    for m in &kujav.methods {
        write_custom_method(&mut file, m, c_a)?;
    }

    file.write_all(&[0x00, 0x00])?; // Class attributes
    println!("✅ Salida.class generada con éxito.");
    Ok(())
}

// --- FUNCIONES DE ESCRITURA DE BYTES ---

fn write_main_method(file: &mut fs::File, kujav: &Compiler, m_n: u16, m_t: u16, c_a: u16) -> std::io::Result<()> {
    file.write_all(&[0x00, 0x09])?; // public static
    file.write_all(&m_n.to_be_bytes())?; 
    file.write_all(&m_t.to_be_bytes())?;
    file.write_all(&[0x00, 0x01])?; // attribute_count: 1
    file.write_all(&c_a.to_be_bytes())?; // "Code" attribute
    
    let main_len: u32 = 12 + kujav.current_bytecode.len() as u32;
    file.write_all(&main_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x0A, 0x00, 0x0A])?; // max_stack (10), max_locals (10)
    file.write_all(&(kujav.current_bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&kujav.current_bytecode)?; // Bytecode del main
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; // exception_table_length (0), attributes_count (0)
    Ok(())
}

fn write_custom_method(file: &mut fs::File, m: &crate::compiler::codegen::MethodInfo, c_a: u16) -> std::io::Result<()> {
    file.write_all(&[0x00, 0x09])?; // public static
    file.write_all(&m.name_idx.to_be_bytes())?; 
    file.write_all(&m.sig_idx.to_be_bytes())?;
    file.write_all(&[0x00, 0x01])?; // attribute_count: 1
    file.write_all(&c_a.to_be_bytes())?; // "Code" attribute
    
    let attr_len: u32 = 12 + m.bytecode.len() as u32;
    file.write_all(&attr_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x0A])?; // max_stack fijo en 10
    file.write_all(&m.max_locals.to_be_bytes())?; // max_locals dinámico
    file.write_all(&(m.bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&m.bytecode)?; // Bytecode de la función
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; // exception_table_length (0), attributes_count (0)
    Ok(())
}