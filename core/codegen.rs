use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::IntPredicate;

use crate::ast::{Expr, Stmt, Function};

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();
        Self { context, module, builder }
    }

    pub fn compile_function(&mut self, func: &Function) {
        let i32_t = self.context.i32_type();
        let fn_ty = i32_t.fn_type(&[], false);
        let f = self.module.add_function(&func.name, fn_ty, None);
        let entry = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);

        let mut locals: HashMap<String, PointerValue<'ctx>> = HashMap::new();
        let mut last_value: Option<IntValue<'ctx>> = None;

        for stmt in &func.body {
            last_value = self.compile_stmt(f, stmt, &i32_t, &mut locals);
        }

        if let Some(v) = last_value {
            let _ = self.builder.build_return(Some(&v));
        } else {
            let zero = i32_t.const_int(0, false);
            let _ = self.builder.build_return(Some(&zero));
        }
    }

    fn compile_stmt(
        &mut self,
        func: FunctionValue<'ctx>,
        stmt: &Stmt,
        i32_t: &IntType<'ctx>,
        locals: &mut HashMap<String, PointerValue<'ctx>>,
    ) -> Option<IntValue<'ctx>> {
        match stmt {
            Stmt::Let(name, expr) => {
                let val = self.compile_expr(expr, i32_t, locals);
                let ptr = self.builder.build_alloca(*i32_t, name).expect("alloca failed");
                self.builder.build_store(ptr, val).expect("store failed");
                locals.insert(name.clone(), ptr);
                None
            }
            Stmt::Assign(name, expr) => {
                let val = self.compile_expr(expr, i32_t, locals);
                let ptr = *locals.get(name).expect("assign to undefined var");
                self.builder.build_store(ptr, val).expect("store failed");
                None
            }
            Stmt::Expr(expr) => {
                let val = self.compile_expr(expr, i32_t, locals);
                Some(val)
            }
            Stmt::If { cond, then_body, else_body } => {
                let cond = self.compile_expr(cond, i32_t, locals);
                let cond_val = self.to_bool(cond, i32_t);


                let then_bb = self.context.append_basic_block(func, "then");
                let else_bb = self.context.append_basic_block(func, "else");
                let cont_bb = self.context.append_basic_block(func, "ifend");

                self.builder
                    .build_conditional_branch(cond_val, then_bb, else_bb)
                    .expect("brcond failed");

                // then
                self.builder.position_at_end(then_bb);
                for s in then_body {
                    self.compile_stmt(func, s, i32_t, locals);
                }
                if then_bb.get_terminator().is_none() {
                    self.builder.build_unconditional_branch(cont_bb).expect("br then->cont");
                }

                // else
                self.builder.position_at_end(else_bb);
                if let Some(eb) = else_body {
                    for s in eb {
                        self.compile_stmt(func, s, i32_t, locals);
                    }
                }
                if else_bb.get_terminator().is_none() {
                    self.builder.build_unconditional_branch(cont_bb).expect("br else->cont");
                }

                // continuation
                self.builder.position_at_end(cont_bb);

                None
            }
            Stmt::While { cond, body } => {
                let cond_bb = self.context.append_basic_block(func, "loop.cond");
                let body_bb = self.context.append_basic_block(func, "loop.body");
                let end_bb  = self.context.append_basic_block(func, "loop.end");

                self.builder.build_unconditional_branch(cond_bb).expect("br -> cond");

                // cond
                self.builder.position_at_end(cond_bb);
                let cond = self.compile_expr(cond, i32_t, locals);
                let cond_val = self.to_bool(cond, i32_t);

                self.builder
                    .build_conditional_branch(cond_val, body_bb, end_bb)
                    .expect("brcond while");

                // body
                self.builder.position_at_end(body_bb);
                for s in body {
                    self.compile_stmt(func, s, i32_t, locals);
                }
                if body_bb.get_terminator().is_none() {
                    self.builder.build_unconditional_branch(cond_bb).expect("br body->cond");
                }

                // end
                self.builder.position_at_end(end_bb);

                None
            }
        }
    }

    fn compile_expr(
        &mut self,
        expr: &Expr,
        i32_t: &IntType<'ctx>,
        locals: &mut HashMap<String, PointerValue<'ctx>>,
    ) -> IntValue<'ctx> {
        match expr {
            Expr::Number(n) => i32_t.const_int(*n as u64, true),
            Expr::Ident(name) => {
                let ptr = *locals.get(name).expect("use of undefined variable");
                let loaded = self
                    .builder
                    .build_load(*i32_t, ptr, &format!("load_{}", name))
                    .expect("load failed");
                loaded.into_int_value()
            }
            Expr::BinaryOp(l, op, r) => {
                let a = self.compile_expr(l, i32_t, locals);
                let b = self.compile_expr(r, i32_t, locals);
                match op.as_str() {
                    "+" => self.builder.build_int_add(a, b, "add").expect("add"),
                    "-" => self.builder.build_int_sub(a, b, "sub").expect("sub"),
                    "*" => self.builder.build_int_mul(a, b, "mul").expect("mul"),
                    "/" => self.builder.build_int_signed_div(a, b, "div").expect("div"),

                    "==" => self.builder.build_int_compare(IntPredicate::EQ,  a, b, "cmpeq").expect("cmp"),
                    "!=" => self.builder.build_int_compare(IntPredicate::NE,  a, b, "cmpne").expect("cmp"),
                    "<"  => self.builder.build_int_compare(IntPredicate::SLT, a, b, "cmplt").expect("cmp"),
                    "<=" => self.builder.build_int_compare(IntPredicate::SLE, a, b, "cmple").expect("cmp"),
                    ">"  => self.builder.build_int_compare(IntPredicate::SGT, a, b, "cmpgt").expect("cmp"),
                    ">=" => self.builder.build_int_compare(IntPredicate::SGE, a, b, "cmpge").expect("cmp"),

                    other => panic!("unsupported op {}", other),
                }
            }
        }
    }

    // Любое IntValue -> i1 (булево для условий).
    fn to_bool(&self, v: IntValue<'ctx>, i32_t: &IntType<'ctx>) -> IntValue<'ctx> {
        if v.get_type().get_bit_width() == 1 {
            v
        } else {
            let zero = i32_t.const_int(0, false);
            self.builder
                .build_int_compare(IntPredicate::NE, v, zero, "tobool")
                .expect("cmp to bool")
        }
    }

    pub fn dump_ir(&self) { self.module.print_to_stderr(); }

    pub fn module(&self) -> &Module<'ctx> { &self.module }
}
