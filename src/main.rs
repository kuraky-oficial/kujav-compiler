// src/main.rs
mod core; mod reader; mod parser; mod compiler;
use std::fs; use std::io::Write;

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
    
    let mut analyzer = compiler::semantics::SemanticAnalyzer::new();
    if let Err(e) = analyzer.analyze(&ast) {
        eprintln!("❌ Error Semántico: {}", e);
        return Ok(());
    }

    let mut kujav = compiler::codegen::Compiler::new();
    
    // Configuración básica del ClassFile (simplificada)
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
    // Escribir bytes... (esto ya lo tienes en tu main original)
    
    println!("✅ Compilación exitosa.");
    Ok(())
}