//! Parser for Pawn source code

use crate::ast::*;
use crate::error::*;
use crate::lexer::*;

/// Parser for Pawn source code
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Option<Token>,
}

impl Parser {
    /// Create a new parser
    pub fn new(input: &str) -> CompilerResult<Self> {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;
        let peek_token = Some(lexer.next_token()?);

        Ok(Parser {
            lexer,
            current_token,
            peek_token,
        })
    }

    /// Advance to the next token
    fn advance(&mut self) -> CompilerResult<()> {
        self.current_token = self.peek_token.take().unwrap_or(Token::EndOfFile);
        if self.current_token != Token::EndOfFile {
            self.peek_token = Some(self.lexer.next_token()?);
        }
        Ok(())
    }

    /// Check if current token matches expected
    fn expect(&mut self, expected: Token) -> CompilerResult<()> {
        if self.current_token == expected {
            self.advance()?;
            Ok(())
        } else {
            Err(CompilerError::ParserError(format!(
                "Expected {:?}, found {:?}",
                expected, self.current_token
            )))
        }
    }

    /// Parse a complete program
    pub fn parse_program(&mut self) -> CompilerResult<AstNode> {
        let mut statements = Vec::new();

        while self.current_token != Token::EndOfFile {
            match self.parse_statement()? {
                Some(stmt) => statements.push(stmt),
                None => break,
            }
        }

        Ok(AstNode::Program(statements))
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> CompilerResult<Option<AstNode>> {
        match &self.current_token {
            Token::Main => {
                self.advance()?;
                self.expect(Token::LeftParen)?;
                self.expect(Token::RightParen)?;

                // Check if there's a left brace, if not, parse single statement
                let mut body = Vec::new();
                if self.current_token == Token::LeftBrace {
                    self.advance()?;
                    while self.current_token != Token::RightBrace
                        && self.current_token != Token::EndOfFile
                    {
                        if let Some(stmt) = self.parse_statement()? {
                            body.push(stmt);
                        }
                    }
                    self.expect(Token::RightBrace)?;
                } else {
                    // Parse single statement without braces: skip trivia first
                    loop {
                        match self.current_token {
                            Token::Newline | Token::Semicolon | Token::Comment(_) => {
                                self.advance()?;
                            }
                            _ => break,
                        }
                    }
                    if let Some(stmt) = self.parse_statement()? {
                        body.push(stmt);
                    }
                }

                Ok(Some(AstNode::Function {
                    name: "main".to_string(),
                    parameters: Vec::new(),
                    return_type: None,
                    body,
                    is_public: false,
                    is_native: false,
                    is_forward: false,
                }))
            }

            Token::Identifier(name) => {
                if name == "printf" {
                    self.advance()?;

                    // Check if there's a left parenthesis
                    if self.current_token == Token::LeftParen {
                        self.advance()?;

                        let format_string = if let Token::String(s) = &self.current_token {
                            let s = s.clone();
                            self.advance()?;
                            s
                        } else {
                            return Err(CompilerError::ParserError(
                                "Expected format string".to_string(),
                            ));
                        };

                        self.expect(Token::RightParen)?;
                        self.expect(Token::Semicolon)?;

                        Ok(Some(AstNode::FunctionCall {
                            name: "printf".to_string(),
                            arguments: vec![AstNode::String(format_string)],
                        }))
                    } else {
                        // printf without parentheses - just take the next string
                        let format_string = if let Token::String(s) = &self.current_token {
                            let s = s.clone();
                            self.advance()?;
                            s
                        } else {
                            return Err(CompilerError::ParserError(
                                "Expected format string".to_string(),
                            ));
                        };

                        Ok(Some(AstNode::FunctionCall {
                            name: "printf".to_string(),
                            arguments: vec![AstNode::String(format_string)],
                        }))
                    }
                } else {
                    // For MVP, skip unknown identifier-started statements until EOL or semicolon
                    while self.current_token != Token::Semicolon
                        && self.current_token != Token::Newline
                        && self.current_token != Token::EndOfFile
                    {
                        self.advance()?;
                    }
                    if self.current_token == Token::Semicolon {
                        self.advance()?;
                    }
                    Ok(None)
                }
            }

            Token::Semicolon => {
                self.advance()?;
                Ok(None)
            }

            Token::Comment(_) => {
                self.advance()?;
                Ok(None)
            }

            Token::Newline => {
                self.advance()?;
                Ok(None)
            }

            // Gracefully skip constructs we don't implement in MVP
            Token::Enum | Token::Forward | Token::New | Token::Const | Token::Static => {
                // Skip until end of line or closing brace or semicolon
                while self.current_token != Token::Semicolon
                    && self.current_token != Token::Newline
                    && self.current_token != Token::RightBrace
                    && self.current_token != Token::EndOfFile
                {
                    self.advance()?;
                }
                if self.current_token == Token::Semicolon {
                    self.advance()?;
                }
                Ok(None)
            }

            _ => {
                // Skip unrecognized token lines conservatively
                while self.current_token != Token::Semicolon
                    && self.current_token != Token::Newline
                    && self.current_token != Token::EndOfFile
                {
                    self.advance()?;
                }
                if self.current_token == Token::Semicolon {
                    self.advance()?;
                }
                Ok(None)
            }
        }
    }

    /// Parse an expression
    #[allow(dead_code)]
    fn parse_expression(&mut self) -> CompilerResult<AstNode> {
        self.parse_equality()
    }

    /// Parse equality expressions
    #[allow(dead_code)]
    fn parse_equality(&mut self) -> CompilerResult<AstNode> {
        let mut left = self.parse_comparison()?;

        while matches!(self.current_token, Token::Equal | Token::NotEqual) {
            let operator = match self.current_token {
                Token::Equal => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => {
                    return Err(CompilerError::ParserError(
                        "Invalid equality operator".into(),
                    ));
                }
            };
            self.advance()?;
            let right = self.parse_comparison()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse comparison expressions
    #[allow(dead_code)]
    fn parse_comparison(&mut self) -> CompilerResult<AstNode> {
        let mut left = self.parse_term()?;

        while matches!(
            self.current_token,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual
        ) {
            let operator = match self.current_token {
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => {
                    return Err(CompilerError::ParserError(
                        "Invalid comparison operator".into(),
                    ));
                }
            };
            self.advance()?;
            let right = self.parse_term()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse term expressions
    #[allow(dead_code)]
    fn parse_term(&mut self) -> CompilerResult<AstNode> {
        let mut left = self.parse_factor()?;

        while matches!(self.current_token, Token::Plus | Token::Minus) {
            let operator = match self.current_token {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => return Err(CompilerError::ParserError("Invalid term operator".into())),
            };
            self.advance()?;
            let right = self.parse_factor()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse factor expressions
    #[allow(dead_code)]
    fn parse_factor(&mut self) -> CompilerResult<AstNode> {
        let mut left = self.parse_unary()?;

        while matches!(
            self.current_token,
            Token::Multiply | Token::Divide | Token::Modulo
        ) {
            let operator = match self.current_token {
                Token::Multiply => BinaryOperator::Multiply,
                Token::Divide => BinaryOperator::Divide,
                Token::Modulo => BinaryOperator::Modulo,
                _ => return Err(CompilerError::ParserError("Invalid factor operator".into())),
            };
            self.advance()?;
            let right = self.parse_unary()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse unary expressions
    #[allow(dead_code)]
    fn parse_unary(&mut self) -> CompilerResult<AstNode> {
        match self.current_token {
            Token::Plus => {
                self.advance()?;
                let operand = self.parse_unary()?;
                Ok(AstNode::UnaryOp {
                    operator: UnaryOperator::Plus,
                    operand: Box::new(operand),
                })
            }
            Token::Minus => {
                self.advance()?;
                let operand = self.parse_unary()?;
                Ok(AstNode::UnaryOp {
                    operator: UnaryOperator::Minus,
                    operand: Box::new(operand),
                })
            }
            Token::LogicalNot => {
                self.advance()?;
                let operand = self.parse_unary()?;
                Ok(AstNode::UnaryOp {
                    operator: UnaryOperator::LogicalNot,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_primary(),
        }
    }

    /// Parse primary expressions
    #[allow(dead_code)]
    fn parse_primary(&mut self) -> CompilerResult<AstNode> {
        match &self.current_token {
            Token::Number(n) => {
                let value = *n;
                self.advance()?;
                Ok(AstNode::Integer(value))
            }
            Token::Float(f) => {
                let value = *f;
                self.advance()?;
                Ok(AstNode::Float(value))
            }
            Token::String(s) => {
                let value = s.clone();
                self.advance()?;
                Ok(AstNode::String(value))
            }
            Token::Character(c) => {
                let value = *c;
                self.advance()?;
                Ok(AstNode::Character(value))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                Ok(AstNode::Identifier(name))
            }
            Token::LeftParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(CompilerError::ParserError(format!(
                "Unexpected token in expression: {:?}",
                self.current_token
            ))),
        }
    }
}
