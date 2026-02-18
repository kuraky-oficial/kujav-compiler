// src/compiler/codegen/expressions.rs
use crate::compiler::codegen::Compiler;
use crate::parser::ast::Expr;

impl Compiler {
    pub fn is_string_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::String(_) => true,
            Expr::Identifier(n) => self.variable_types.get(n).map(|t| t.as_str()) == Some("Ljava/lang/String;"),
            Expr::Binary(l, op, r) => op == "+" && (self.is_string_expr(l) || self.is_string_expr(r)),
            _ => false,
        }
    }

    pub fn compile_expression(&mut self, expr: Expr) {
        match expr {
            Expr::Number(val) => { 
                self.current_bytecode.push(0x10); self.current_bytecode.push(val as u8); 
            }
            Expr::Boolean(val) => { 
                self.current_bytecode.push(if val { 0x04 } else { 0x03 }); 
            }
            Expr::String(c) => {
                let u_idx = self.cp.add_utf8(&c);
                let s_idx = self.cp.add_string(u_idx);
                self.current_bytecode.push(0x12); self.current_bytecode.push(s_idx as u8);
            }
            Expr::Identifier(n) => {
                if let Some(&slot) = self.variables.get(&n) {
                    let is_i = self.variable_types.get(&n).map(|s| s.as_str()) == Some("I");
                    self.current_bytecode.push(if is_i { 0x15 } else { 0x19 }); 
                    self.current_bytecode.push(slot);
                }
            }
            Expr::ArrayLiteral(elems) => {
                self.current_bytecode.push(0x10); self.current_bytecode.push(elems.len() as u8);
                self.current_bytecode.push(0xBC); self.current_bytecode.push(10); 
                for (i, e) in elems.into_iter().enumerate() {
                    self.current_bytecode.push(0x59);
                    self.current_bytecode.push(0x10); self.current_bytecode.push(i as u8);
                    self.compile_expression(e);
                    self.current_bytecode.push(0x4F);
                }
            }
            Expr::ArrayAccess(name, idx) => {
                if let Some(&slot) = self.variables.get(&name) {
                    self.current_bytecode.push(0x19); self.current_bytecode.push(slot); 
                    self.compile_expression(*idx);
                    self.current_bytecode.push(0x2E); 
                }
            }
            Expr::Binary(l, op, r) => {
                if op == "+" && (self.is_string_expr(&l) || self.is_string_expr(&r)) {
                    let sb_u = self.cp.add_utf8("java/lang/StringBuilder");
                    let sb_c = self.cp.add_class(sb_u);
                    self.current_bytecode.push(0xBB); self.current_bytecode.extend_from_slice(&sb_c.to_be_bytes());
                    self.current_bytecode.push(0x59);
                    let nt_init = self.cp.add_name_and_type(self.cp.add_utf8("<init>"), self.cp.add_utf8("()V"));
                    self.current_bytecode.push(0xB7); self.current_bytecode.extend_from_slice(&self.cp.add_method_ref(sb_c, nt_init).to_be_bytes());

                    let l_str = self.is_string_expr(&l);
                    self.compile_expression(*l);
                    let sig_l = self.cp.add_utf8(if l_str { "(Ljava/lang/String;)Ljava/lang/StringBuilder;" } else { "(I)Ljava/lang/StringBuilder;" });
                    let nt_l = self.cp.add_name_and_type(self.cp.add_utf8("append"), sig_l);
                    self.current_bytecode.push(0xB6); self.current_bytecode.extend_from_slice(&self.cp.add_method_ref(sb_c, nt_l).to_be_bytes());

                    let r_str = self.is_string_expr(&r);
                    self.compile_expression(*r);
                    let sig_r = self.cp.add_utf8(if r_str { "(Ljava/lang/String;)Ljava/lang/StringBuilder;" } else { "(I)Ljava/lang/StringBuilder;" });
                    let nt_r = self.cp.add_name_and_type(self.cp.add_utf8("append"), sig_r);
                    self.current_bytecode.push(0xB6); self.current_bytecode.extend_from_slice(&self.cp.add_method_ref(sb_c, nt_r).to_be_bytes());

                    let nt_ts = self.cp.add_name_and_type(self.cp.add_utf8("toString"), self.cp.add_utf8("()Ljava/lang/String;"));
                    self.current_bytecode.push(0xB6); self.current_bytecode.extend_from_slice(&self.cp.add_method_ref(sb_c, nt_ts).to_be_bytes());
                } else {
                    self.compile_expression(*l); self.compile_expression(*r);
                    match op.as_str() {
                        "+" => self.current_bytecode.push(0x60), "-" => self.current_bytecode.push(0x64),
                        "*" => self.current_bytecode.push(0x68), "/" => self.current_bytecode.push(0x6C),
                        "==" => self.current_bytecode.extend_from_slice(&[0xA0, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                        _ => {}
                    }
                }
            }
            Expr::Call(name, args) => {
                for a in args { self.compile_expression(a); }
                let cls = self.cp.add_class(self.cp.add_utf8("Salida"));
                let nt = self.cp.add_name_and_type(self.cp.add_utf8(&name), self.cp.add_utf8("()I"));
                self.current_bytecode.push(0xB8); self.current_bytecode.extend_from_slice(&self.cp.add_method_ref(cls, nt).to_be_bytes());
            }
            Expr::Input => {
                let scan_c = self.cp.add_class(self.cp.add_utf8("java/util/Scanner"));
                self.current_bytecode.push(0xBB); self.current_bytecode.extend_from_slice(&scan_c.to_be_bytes());
                self.current_bytecode.push(0x59);
                let sys_in = self.cp.add_field_ref(self.cp.add_class(self.cp.add_utf8("java/lang/System")), self.cp.add_name_and_type(self.cp.add_utf8("in"), self.cp.add_utf8("Ljava/io/InputStream;")));
                self.current_bytecode.push(0xB2); self.current_bytecode.extend_from_slice(&sys_in.to_be_bytes());
                let m_init = self.cp.add_method_ref(scan_c, self.cp.add_name_and_type(self.cp.add_utf8("<init>"), self.cp.add_utf8("(Ljava/io/InputStream;)V")));
                self.current_bytecode.push(0xB7); self.current_bytecode.extend_from_slice(&m_init.to_be_bytes());
                let m_next = self.cp.add_method_ref(scan_c, self.cp.add_name_and_type(self.cp.add_utf8("nextInt"), self.cp.add_utf8("()I")));
                self.current_bytecode.push(0xB6); self.current_bytecode.extend_from_slice(&m_next.to_be_bytes());
            }
        }
    }
}