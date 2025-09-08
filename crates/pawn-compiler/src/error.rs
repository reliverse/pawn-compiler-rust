//! Error handling for the Pawn compiler

use thiserror::Error;

/// Compiler error types
#[derive(Error, Debug, Clone)]
pub enum CompilerError {
    #[error("Lexical error: {0}")]
    LexicalError(String),

    #[error("Syntax error: {0}")]
    SyntaxError(String),

    #[error("Parser error: {0}")]
    ParserError(String),

    #[error("Semantic error: {0}")]
    SemanticError(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Symbol error: {0}")]
    SymbolError(String),

    #[error("Code generation error: {0}")]
    CodeGenError(String),

    #[error("File error: {0}")]
    FileError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for compiler operations
pub type CompilerResult<T> = Result<T, CompilerError>;
