//! Pawn compiler implementation
//!
//! This crate provides the core compiler functionality for parsing Pawn source code
//! and generating AMX bytecode.

pub mod ast;
pub mod codegen;
pub mod config;
pub mod error;
pub mod formatter;
pub mod lexer;
pub mod linter;
pub mod parser;
pub mod symbol_table;

pub use ast::*;
pub use codegen::*;
pub use config::*;
pub use error::*;
pub use formatter::*;
pub use lexer::*;
pub use linter::*;
pub use parser::*;
pub use symbol_table::*;

/// Compile Pawn source code to AMX bytecode
pub fn compile(source_code: &str) -> CompilerResult<Vec<u8>> {
    // Lexical analysis
    let mut lexer = Lexer::new(source_code);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token()?;
        match token {
            Token::EndOfFile => break,
            _ => tokens.push(token),
        }
    }

    // Parsing
    let mut parser = Parser::new(source_code)?;
    let ast = parser.parse_program()?;

    // Symbol table analysis
    let mut symbol_visitor = SymbolTableVisitor::new();
    symbol_visitor.analyze(&ast)?;

    // Code generation
    let mut codegen = CodeGenerator::new();
    let bytecode = codegen.generate(&ast)?;

    Ok(bytecode)
}
