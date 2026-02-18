use crate::core::constant_pool::ConstantPool;
use crate::parser::ast::{Stmt, Expr};

pub struct Compiler {
    pub cp: ConstantPool,
    pub bytecode: Vec<u8>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            cp: ConstantPool::new(),
            bytecode: Vec::new(),
        }
    }

    pub fn compile_statement(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Print(expr) => {
                // 1. Preparar referencias de System.out.println
                let sys_cls = self.cp.add_class(self.cp.add_utf8("java/lang/System"));
                let out_nt = self.cp.add_name_and_type(
                    self.cp.add_utf8("out"), 
                    self.cp.add_utf8("Ljava/io/PrintStream;")
                );
                let field_out = self.cp.add_field_ref(sys_cls, out_nt);

                let ps_cls = self.cp.add_class(self.cp.add_utf8("java/io/PrintStream"));
                let println_nt = self.cp.add_name_and_type(
                    self.cp.add_utf8("println"), 
                    self.cp.add_utf8("(Ljava/lang/String;)V")
                );
                let method_println = self.cp.add_method_ref(ps_cls, println_nt);

                // 2. Generar instrucciones
                // getstatic
                self.bytecode.push(0xB2);
                self.bytecode.extend_from_slice(&field_out.to_be_bytes());

                // ldc (cargar el string)
                if let Expr::String(content) = expr {
                    let s_idx = self.cp.add_string(self.cp.add_utf8(&content));
                    self.bytecode.push(0x12);
                    self.bytecode.push(s_idx as u8);
                }

                // invokevirtual
                self.bytecode.push(0xB6);
                self.bytecode.extend_from_slice(&method_println.to_be_bytes());
            }
            _ => println!("⚠️ Instrucción no soportada aún."),
        }
    }
}