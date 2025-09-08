//! Code generation from AST to AMX bytecode

use crate::ast::*;
use crate::error::*;
use pawn_amx::instructions::{Instruction, Opcode};
use pawn_amx::*;
use std::collections::HashMap;

/// Code generator for AMX bytecode
pub struct CodeGenerator {
    instructions: Vec<Instruction>,
    data: Vec<u8>,
    strings: Vec<String>,
    string_map: HashMap<String, usize>,
    label_map: HashMap<String, usize>,
    next_label: usize,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            data: Vec::new(),
            strings: Vec::new(),
            string_map: HashMap::new(),
            label_map: HashMap::new(),
            next_label: 0,
        }
    }

    /// Generate AMX bytecode from AST
    pub fn generate(&mut self, ast: &AstNode) -> CompilerResult<Vec<u8>> {
        self.instructions.clear();
        self.data.clear();
        self.strings.clear();
        self.string_map.clear();
        self.label_map.clear();
        self.next_label = 0;

        // Generate code for the AST
        self.generate_node(ast)?;

        // Add halt instruction
        self.instructions.push(Instruction::new(Opcode::Halt, 0));

        // Create AMX header
        let mut header = AmxHeader::new();
        header.size = (std::mem::size_of::<AmxHeader>()
            + self.instructions.len() * 5
            + self.data.len()) as i32;
        header.cod = std::mem::size_of::<AmxHeader>() as i32;
        header.dat = header.cod + (self.instructions.len() * 5) as i32;
        header.hea = header.dat + self.data.len() as i32;
        header.stp = header.hea;
        // Start execution at the first instruction (after the header)
        header.cip = 0;

        // Build final bytecode
        let mut bytecode = Vec::new();
        bytecode.extend_from_slice(&write_header(&header));

        // Add instructions
        for instruction in &self.instructions {
            bytecode.extend_from_slice(&instruction.to_bytes());
        }

        // Add data
        bytecode.extend_from_slice(&self.data);

        Ok(bytecode)
    }

    /// Generate code for an AST node
    fn generate_node(&mut self, node: &AstNode) -> CompilerResult<()> {
        match node {
            AstNode::Program(statements) => {
                for stmt in statements {
                    self.generate_node(stmt)?;
                }
            }

            AstNode::Function { name, body, .. } => {
                if name == "main" {
                    for stmt in body {
                        self.generate_node(stmt)?;
                    }
                }
            }

            AstNode::FunctionCall { name, arguments } => {
                if name == "printf" {
                    self.generate_printf(arguments)?;
                } else {
                    return Err(CompilerError::SemanticError(format!(
                        "Unknown function: {}",
                        name
                    )));
                }
            }

            AstNode::String(s) => {
                // Store string in data section
                let string_id = self.add_string(s);
                self.instructions
                    .push(Instruction::new(Opcode::ConstPri, string_id as i32));
            }

            AstNode::Integer(n) => {
                self.instructions
                    .push(Instruction::new(Opcode::ConstPri, *n));
            }

            AstNode::Float(f) => {
                // Convert float to integer representation for now
                let int_val = *f as i32;
                self.instructions
                    .push(Instruction::new(Opcode::ConstPri, int_val));
            }

            AstNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                self.generate_node(left)?;
                self.instructions.push(Instruction::new(Opcode::PushPri, 0));
                self.generate_node(right)?;
                self.instructions.push(Instruction::new(Opcode::PopAlt, 0));

                match operator {
                    BinaryOperator::Add => {
                        self.instructions.push(Instruction::new(Opcode::Add, 0));
                    }
                    BinaryOperator::Subtract => {
                        self.instructions.push(Instruction::new(Opcode::Sub, 0));
                    }
                    BinaryOperator::Multiply => {
                        self.instructions.push(Instruction::new(Opcode::Smul, 0));
                    }
                    BinaryOperator::Divide => {
                        self.instructions.push(Instruction::new(Opcode::Sdiv, 0));
                    }
                    BinaryOperator::Equal => {
                        self.instructions.push(Instruction::new(Opcode::Eq, 0));
                    }
                    BinaryOperator::NotEqual => {
                        self.instructions.push(Instruction::new(Opcode::Neq, 0));
                    }
                    BinaryOperator::Less => {
                        self.instructions.push(Instruction::new(Opcode::Less, 0));
                    }
                    BinaryOperator::LessEqual => {
                        self.instructions.push(Instruction::new(Opcode::Leq, 0));
                    }
                    BinaryOperator::Greater => {
                        self.instructions.push(Instruction::new(Opcode::Grtr, 0));
                    }
                    BinaryOperator::GreaterEqual => {
                        self.instructions.push(Instruction::new(Opcode::Geq, 0));
                    }
                    _ => {
                        return Err(CompilerError::SemanticError(format!(
                            "Unsupported operator: {:?}",
                            operator
                        )));
                    }
                }
            }

            AstNode::UnaryOp { operator, operand } => {
                self.generate_node(operand)?;
                match operator {
                    UnaryOperator::Plus => {
                        // No operation needed
                    }
                    UnaryOperator::Minus => {
                        self.instructions.push(Instruction::new(Opcode::Neg, 0));
                    }
                    UnaryOperator::LogicalNot => {
                        // For now, just negate the value
                        self.instructions.push(Instruction::new(Opcode::Eq, 0));
                    }
                    _ => {
                        return Err(CompilerError::SemanticError(format!(
                            "Unsupported unary operator: {:?}",
                            operator
                        )));
                    }
                }
            }

            _ => {
                return Err(CompilerError::SemanticError(format!(
                    "Unsupported AST node: {:?}",
                    node
                )));
            }
        }

        Ok(())
    }

    /// Generate printf function call
    fn generate_printf(&mut self, arguments: &[AstNode]) -> CompilerResult<()> {
        if arguments.is_empty() {
            return Err(CompilerError::SemanticError(
                "printf requires at least one argument".to_string(),
            ));
        }

        // For now, just print the first argument as a string
        if let AstNode::String(s) = &arguments[0] {
            // In a real implementation, we would call a native printf function
            // For MVP, we'll just simulate it by storing the string
            let string_id = self.add_string(s);
            self.instructions
                .push(Instruction::new(Opcode::ConstPri, string_id as i32));
            // Call printf native (index 0 for now)
            self.instructions.push(Instruction::new(Opcode::Sysreq, 0));
        } else {
            return Err(CompilerError::SemanticError(
                "printf first argument must be a string".to_string(),
            ));
        }

        Ok(())
    }

    /// Add a string to the data section
    fn add_string(&mut self, s: &str) -> usize {
        if let Some(&id) = self.string_map.get(s) {
            return id;
        }

        let id = self.strings.len();
        self.strings.push(s.to_string());
        self.string_map.insert(s.to_string(), id);

        // Store string in data section
        let string_bytes = s.as_bytes();
        let _start_offset = self.data.len();
        self.data.extend_from_slice(string_bytes);
        self.data.push(0); // Null terminator

        id
    }

    /// Create a new label
    #[allow(dead_code)]
    fn create_label(&mut self) -> String {
        let label = format!("label_{}", self.next_label);
        self.next_label += 1;
        label
    }

    /// Set label position
    #[allow(dead_code)]
    fn set_label(&mut self, label: &str) {
        self.label_map
            .insert(label.to_string(), self.instructions.len());
    }

    /// Get label address
    #[allow(dead_code)]
    fn get_label_address(&self, label: &str) -> Option<i32> {
        self.label_map.get(label).map(|&addr| addr as i32)
    }
}
