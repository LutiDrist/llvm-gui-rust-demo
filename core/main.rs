mod lexer;
mod parser;
mod ast;
mod interpreter;
mod codegen;

use inkwell::context::Context;

use std::env;
use std::fs;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interpreter;
use crate::codegen::Codegen;

/// Если запускаешь без аргументов — используется demo-программа.
/// Если передаёшь путь до файла — выполняем его.
fn main() {
    let args: Vec<String> = env::args().collect();

    let input = if args.len() >= 2 {
        let path = &args[1];
        match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("failed to read {}: {}", path, e);
                return;
            }
        }
    } else {
        // demo program
        r#"
        fn main() {
            let x = 0;
            while (x < 5) {
                x = x + 1;
            }
            if (x == 5) {
                42;
            } else {
                0;
            }
        }
        "#.to_string()
    };

    // LEXER
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.tokenize();
    println!("tokens {:?}", tokens);

    // PARSER
    let mut parser = Parser::new(tokens);
    if let Some(func) = parser.parse_function() {
        println!("AST: {:#?}", func);

        // INTERPRETER
        println!("\n=== Interpreter ===");
        Interpreter::run_function(&func);

        // CODEGEN -> LLVM IR (печатаем IR)
        println!("\n=== LLVM IR (generated) ===");
        let context = Context::create();
        let mut codegen = Codegen::new(&context, "simple_module");
        codegen.compile_function(&func);
        codegen.dump_ir();
    } else {
        println!("Parser error");
    }
}
