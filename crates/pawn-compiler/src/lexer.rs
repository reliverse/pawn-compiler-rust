//! Lexical analysis for Pawn source code

use crate::error::*;
use std::collections::HashMap;

/// Token types for Pawn
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(i32),
    Float(f32),
    String(String),
    Character(char),

    // Identifiers
    Identifier(String),

    // Keywords
    Main,
    Public,
    Native,
    Forward,
    Static,
    Const,
    New,
    Delete,
    If,
    Else,
    While,
    For,
    Do,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Return,
    Goto,
    Enum,
    Struct,
    Union,
    Typedef,
    Extern,
    Auto,
    Register,
    Volatile,
    Inline,
    Restrict,

    // Operators
    Plus,           // +
    Minus,          // -
    Multiply,       // *
    Divide,         // /
    Modulo,         // %
    Assign,         // =
    PlusAssign,     // +=
    MinusAssign,    // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=
    ModuloAssign,   // %=
    Equal,          // ==
    NotEqual,       // !=
    Less,           // <
    LessEqual,      // <=
    Greater,        // >
    GreaterEqual,   // >=
    LogicalAnd,     // &&
    LogicalOr,      // ||
    LogicalNot,     // !
    BitwiseAnd,     // &
    BitwiseOr,      // |
    BitwiseXor,     // ^
    BitwiseNot,     // ~
    LeftShift,      // <<
    RightShift,     // >>
    Increment,      // ++
    Decrement,      // --

    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Semicolon,    // ;
    Comma,        // ,
    Dot,          // .
    Arrow,        // ->
    Colon,        // :
    Question,     // ?

    // Special
    EndOfFile,
    Newline,
    Comment(String),
}

/// Lexer for Pawn source code
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    keywords: HashMap<String, Token>,
}

impl Lexer {
    /// Create a new lexer
    pub fn new(input: &str) -> Self {
        let mut lexer = Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            keywords: HashMap::new(),
        };

        lexer.init_keywords();
        lexer
    }

    /// Initialize keyword mapping
    fn init_keywords(&mut self) {
        self.keywords.insert("main".to_string(), Token::Main);
        self.keywords.insert("public".to_string(), Token::Public);
        self.keywords.insert("native".to_string(), Token::Native);
        self.keywords.insert("forward".to_string(), Token::Forward);
        self.keywords.insert("static".to_string(), Token::Static);
        self.keywords.insert("const".to_string(), Token::Const);
        self.keywords.insert("new".to_string(), Token::New);
        self.keywords.insert("delete".to_string(), Token::Delete);
        self.keywords.insert("if".to_string(), Token::If);
        self.keywords.insert("else".to_string(), Token::Else);
        self.keywords.insert("while".to_string(), Token::While);
        self.keywords.insert("for".to_string(), Token::For);
        self.keywords.insert("do".to_string(), Token::Do);
        self.keywords.insert("switch".to_string(), Token::Switch);
        self.keywords.insert("case".to_string(), Token::Case);
        self.keywords.insert("default".to_string(), Token::Default);
        self.keywords.insert("break".to_string(), Token::Break);
        self.keywords
            .insert("continue".to_string(), Token::Continue);
        self.keywords.insert("return".to_string(), Token::Return);
        self.keywords.insert("goto".to_string(), Token::Goto);
        self.keywords.insert("enum".to_string(), Token::Enum);
        self.keywords.insert("struct".to_string(), Token::Struct);
        self.keywords.insert("union".to_string(), Token::Union);
        self.keywords.insert("typedef".to_string(), Token::Typedef);
        self.keywords.insert("extern".to_string(), Token::Extern);
        self.keywords.insert("auto".to_string(), Token::Auto);
        self.keywords
            .insert("register".to_string(), Token::Register);
        self.keywords
            .insert("volatile".to_string(), Token::Volatile);
        self.keywords.insert("inline".to_string(), Token::Inline);
        self.keywords
            .insert("restrict".to_string(), Token::Restrict);
    }

    /// Get the current character
    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    /// Peek at the next character
    #[allow(dead_code)]
    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    /// Advance to the next character
    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
    }

    /// Skip whitespace
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Read a number
    fn read_number(&mut self) -> CompilerResult<Token> {
        let mut value = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            let float_val = value.parse::<f32>().map_err(|_| {
                CompilerError::LexicalError(format!("Invalid float literal: {}", value))
            })?;
            Ok(Token::Float(float_val))
        } else {
            let int_val = value.parse::<i32>().map_err(|_| {
                CompilerError::LexicalError(format!("Invalid integer literal: {}", value))
            })?;
            Ok(Token::Number(int_val))
        }
    }

    /// Read a string literal
    fn read_string(&mut self) -> CompilerResult<Token> {
        let mut value = String::new();
        self.advance(); // Skip opening quote

        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance(); // Skip closing quote
                break;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char() {
                    let escaped_char = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        _ => escaped,
                    };
                    value.push(escaped_char);
                    self.advance();
                }
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Ok(Token::String(value))
    }

    /// Read a character literal
    fn read_character(&mut self) -> CompilerResult<Token> {
        let mut value = '\0';
        self.advance(); // Skip opening quote

        if let Some(ch) = self.current_char() {
            if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char() {
                    value = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '\'' => '\'',
                        _ => escaped,
                    };
                    self.advance();
                }
            } else {
                value = ch;
                self.advance();
            }
        }

        if let Some(ch) = self.current_char() {
            if ch == '\'' {
                self.advance(); // Skip closing quote
            }
        }

        Ok(Token::Character(value))
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> CompilerResult<Token> {
        let mut value = String::new();

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a keyword
        if let Some(keyword) = self.keywords.get(&value) {
            Ok(keyword.clone())
        } else {
            Ok(Token::Identifier(value))
        }
    }

    /// Read a comment
    fn read_comment(&mut self) -> CompilerResult<Token> {
        let mut value = String::new();
        self.advance(); // Skip first /

        if let Some(ch) = self.current_char() {
            if ch == '/' {
                // Single-line comment
                self.advance();
                while let Some(ch) = self.current_char() {
                    if ch == '\n' {
                        break;
                    }
                    value.push(ch);
                    self.advance();
                }
            } else if ch == '*' {
                // Multi-line comment
                self.advance();
                while let Some(ch) = self.current_char() {
                    if ch == '*' {
                        self.advance();
                        if let Some(next_ch) = self.current_char() {
                            if next_ch == '/' {
                                self.advance();
                                break;
                            }
                        }
                    }
                    value.push(ch);
                    self.advance();
                }
            }
        }

        Ok(Token::Comment(value))
    }

    /// Get the next token
    pub fn next_token(&mut self) -> CompilerResult<Token> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Ok(Token::EndOfFile);
        }

        let Some(ch) = self.current_char() else {
            return Ok(Token::EndOfFile);
        };

        match ch {
            '\n' => {
                self.advance();
                Ok(Token::Newline)
            }

            '+' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    match next_ch {
                        '+' => {
                            self.advance();
                            Ok(Token::Increment)
                        }
                        '=' => {
                            self.advance();
                            Ok(Token::PlusAssign)
                        }
                        _ => Ok(Token::Plus),
                    }
                } else {
                    Ok(Token::Plus)
                }
            }

            '-' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    match next_ch {
                        '-' => {
                            self.advance();
                            Ok(Token::Decrement)
                        }
                        '=' => {
                            self.advance();
                            Ok(Token::MinusAssign)
                        }
                        '>' => {
                            self.advance();
                            Ok(Token::Arrow)
                        }
                        _ => Ok(Token::Minus),
                    }
                } else {
                    Ok(Token::Minus)
                }
            }

            '*' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    if next_ch == '=' {
                        self.advance();
                        Ok(Token::MultiplyAssign)
                    } else {
                        Ok(Token::Multiply)
                    }
                } else {
                    Ok(Token::Multiply)
                }
            }

            '/' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    match next_ch {
                        '/' | '*' => self.read_comment(),
                        '=' => {
                            self.advance();
                            Ok(Token::DivideAssign)
                        }
                        _ => Ok(Token::Divide),
                    }
                } else {
                    Ok(Token::Divide)
                }
            }

            '%' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    if next_ch == '=' {
                        self.advance();
                        Ok(Token::ModuloAssign)
                    } else {
                        Ok(Token::Modulo)
                    }
                } else {
                    Ok(Token::Modulo)
                }
            }

            '=' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    if next_ch == '=' {
                        self.advance();
                        Ok(Token::Equal)
                    } else {
                        Ok(Token::Assign)
                    }
                } else {
                    Ok(Token::Assign)
                }
            }

            '!' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    if next_ch == '=' {
                        self.advance();
                        Ok(Token::NotEqual)
                    } else if next_ch == '"' {
                        // Packed string literal, treat same as normal string for MVP
                        self.read_string()
                    } else {
                        Ok(Token::LogicalNot)
                    }
                } else {
                    Ok(Token::LogicalNot)
                }
            }

            '<' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    match next_ch {
                        '<' => {
                            self.advance();
                            Ok(Token::LeftShift)
                        }
                        '=' => {
                            self.advance();
                            Ok(Token::LessEqual)
                        }
                        _ => Ok(Token::Less),
                    }
                } else {
                    Ok(Token::Less)
                }
            }

            '>' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    match next_ch {
                        '>' => {
                            self.advance();
                            Ok(Token::RightShift)
                        }
                        '=' => {
                            self.advance();
                            Ok(Token::GreaterEqual)
                        }
                        _ => Ok(Token::Greater),
                    }
                } else {
                    Ok(Token::Greater)
                }
            }

            '&' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    if next_ch == '&' {
                        self.advance();
                        Ok(Token::LogicalAnd)
                    } else {
                        Ok(Token::BitwiseAnd)
                    }
                } else {
                    Ok(Token::BitwiseAnd)
                }
            }

            '|' => {
                self.advance();
                if let Some(next_ch) = self.current_char() {
                    if next_ch == '|' {
                        self.advance();
                        Ok(Token::LogicalOr)
                    } else {
                        Ok(Token::BitwiseOr)
                    }
                } else {
                    Ok(Token::BitwiseOr)
                }
            }

            '^' => {
                self.advance();
                Ok(Token::BitwiseXor)
            }

            '~' => {
                self.advance();
                Ok(Token::BitwiseNot)
            }

            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }

            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }

            '[' => {
                self.advance();
                Ok(Token::LeftBracket)
            }

            ']' => {
                self.advance();
                Ok(Token::RightBracket)
            }

            '{' => {
                self.advance();
                Ok(Token::LeftBrace)
            }

            '}' => {
                self.advance();
                Ok(Token::RightBrace)
            }

            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }

            ',' => {
                self.advance();
                Ok(Token::Comma)
            }

            '.' => {
                self.advance();
                Ok(Token::Dot)
            }

            ':' => {
                self.advance();
                Ok(Token::Colon)
            }

            '?' => {
                self.advance();
                Ok(Token::Question)
            }

            '"' => self.read_string(),

            '\'' => self.read_character(),

            // Preprocessor directive, skip to end of line
            '#' => {
                // consume '#'
                self.advance();
                let mut txt = String::new();
                while let Some(c) = self.current_char() {
                    if c == '\n' {
                        break;
                    }
                    txt.push(c);
                    self.advance();
                }
                Ok(Token::Comment(txt))
            }

            // Event labels like @timer, @keypressed — treat as comment for MVP
            '@' => {
                self.advance();
                let mut txt = String::new();
                while let Some(c) = self.current_char() {
                    if c == '\n' {
                        break;
                    }
                    txt.push(c);
                    self.advance();
                }
                Ok(Token::Comment(txt))
            }

            _ => {
                if ch.is_ascii_alphabetic() || ch == '_' {
                    self.read_identifier()
                } else if ch.is_ascii_digit() {
                    self.read_number()
                } else {
                    Err(CompilerError::LexicalError(format!(
                        "Unexpected character: {}",
                        ch
                    )))
                }
            }
        }
    }

    /// Get current line number
    pub fn line(&self) -> usize {
        self.line
    }

    /// Get current column number
    pub fn column(&self) -> usize {
        self.column
    }
}
