use crate::ast::{Expr, Stmt, Function};
use std::collections::HashMap;

pub struct Interpreter {
    vars: HashMap<String, i64>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { vars: HashMap::new() }
    }

    pub fn run_function(func: &Function) {
        let mut interp = Interpreter::new();
        for stmt in &func.body {
            interp.exec_stmt(stmt);
        }
    }

    fn exec_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, expr) => {
                let val = self.eval_expr(expr);
                self.vars.insert(name.clone(), val);
                println!("let {} = {}", name, val);
            }
            Stmt::Assign(name,expr) => {
                let val = self.eval_expr(expr);
                if self.vars.contains_key(name) {
                    self.vars.insert(name.clone(), val);
                } else {
                    panic!("assgin to undefined var {}", name);
                }
                println!("{} = {}", name, val);
            }
            Stmt::Expr(expr) => {
                let val = self.eval_expr(expr);
                println!("expr => {}", val);
            }
            Stmt::If { cond, then_body, else_body } => {
                let c = self.eval_expr(cond) != 0;
                if c {
                    for s in then_body {self.exec_stmt(s);}
                } else if let Some(eb) = else_body {
                    for s in eb {self.exec_stmt(s);}
                }
            }
            &Stmt::While { .. } => {
    todo!("While loop not implemented yet")
}
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> i64 {
        match expr {
            Expr::Number(n) => *n,
            Expr::Ident(name) => *self.vars.get(name).unwrap_or(&0),
            Expr::BinaryOp(left, op, right) => {
                let a = self.eval_expr(left);
                let b = self.eval_expr(right);
                match op.as_str() {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => a / b,
                    "==" => (a == b) as i64,
                    "!=" => (a != b) as i64,
                    "<" => (a < b) as i64,
                    "<=" => (a <= b) as i64,
                    ">" => (a > b) as i64,
                    ">=" => (a >= b) as i64,
                    _ => panic!("Unknown operator {}", op),
                }
            }
        }
    }
}
