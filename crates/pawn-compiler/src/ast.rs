//! Abstract Syntax Tree for Pawn

use crate::error::*;

/// AST node types
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    // Program structure
    Program(Vec<AstNode>),

    // Function definitions
    Function {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<String>,
        body: Vec<AstNode>,
        is_public: bool,
        is_native: bool,
        is_forward: bool,
    },

    // Variable declarations
    VariableDeclaration {
        name: String,
        var_type: String,
        initializer: Option<Box<AstNode>>,
        is_const: bool,
        is_static: bool,
    },

    // Statements
    Block(Vec<AstNode>),
    Expression(Box<AstNode>),
    If {
        condition: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },
    While {
        condition: Box<AstNode>,
        body: Box<AstNode>,
    },
    For {
        init: Option<Box<AstNode>>,
        condition: Option<Box<AstNode>>,
        update: Option<Box<AstNode>>,
        body: Box<AstNode>,
    },
    Return(Option<Box<AstNode>>),
    Break,
    Continue,

    // Expressions
    BinaryOp {
        left: Box<AstNode>,
        operator: BinaryOperator,
        right: Box<AstNode>,
    },
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<AstNode>,
    },
    Assignment {
        target: Box<AstNode>,
        value: Box<AstNode>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<AstNode>,
    },
    ArrayAccess {
        array: Box<AstNode>,
        index: Box<AstNode>,
    },
    MemberAccess {
        object: Box<AstNode>,
        member: String,
    },

    // Literals
    Integer(i32),
    Float(f32),
    String(String),
    Character(char),
    Boolean(bool),
    Identifier(String),

    // Type definitions
    TypeDefinition {
        name: String,
        definition: TypeDefinition,
    },

    // Enum definitions
    EnumDefinition {
        name: String,
        variants: Vec<EnumVariant>,
    },
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub is_reference: bool,
    pub default_value: Option<Box<AstNode>>,
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical
    LogicalAnd,
    LogicalOr,

    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,

    // Assignment
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LeftShiftAssign,
    RightShiftAssign,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    LogicalNot,
    BitwiseNot,
    Increment,
    Decrement,
    AddressOf,
    Dereference,
}

/// Type definitions
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefinition {
    Primitive(String),
    Array {
        element_type: Box<TypeDefinition>,
        size: Option<Box<AstNode>>,
    },
    Pointer(Box<TypeDefinition>),
    Struct {
        fields: Vec<StructField>,
    },
    Union {
        fields: Vec<StructField>,
    },
    Enum {
        variants: Vec<EnumVariant>,
    },
    Function {
        parameters: Vec<Parameter>,
        return_type: Option<String>,
    },
}

/// Struct field
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: TypeDefinition,
}

/// Enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<Box<AstNode>>,
}

/// AST visitor trait
pub trait AstVisitor<T> {
    fn visit_program(&mut self, nodes: &[AstNode]) -> CompilerResult<T>;
    fn visit_function(
        &mut self,
        name: &str,
        parameters: &[Parameter],
        return_type: &Option<String>,
        body: &[AstNode],
        is_public: bool,
        is_native: bool,
        is_forward: bool,
    ) -> CompilerResult<T>;
    fn visit_variable_declaration(
        &mut self,
        name: &str,
        var_type: &str,
        initializer: &Option<Box<AstNode>>,
        is_const: bool,
        is_static: bool,
    ) -> CompilerResult<T>;
    fn visit_block(&mut self, statements: &[AstNode]) -> CompilerResult<T>;
    fn visit_if(
        &mut self,
        condition: &AstNode,
        then_branch: &AstNode,
        else_branch: &Option<Box<AstNode>>,
    ) -> CompilerResult<T>;
    fn visit_while(&mut self, condition: &AstNode, body: &AstNode) -> CompilerResult<T>;
    fn visit_for(
        &mut self,
        init: &Option<Box<AstNode>>,
        condition: &Option<Box<AstNode>>,
        update: &Option<Box<AstNode>>,
        body: &AstNode,
    ) -> CompilerResult<T>;
    fn visit_return(&mut self, value: &Option<Box<AstNode>>) -> CompilerResult<T>;
    fn visit_break(&mut self) -> CompilerResult<T>;
    fn visit_continue(&mut self) -> CompilerResult<T>;
    fn visit_binary_op(
        &mut self,
        left: &AstNode,
        operator: &BinaryOperator,
        right: &AstNode,
    ) -> CompilerResult<T>;
    fn visit_unary_op(&mut self, operator: &UnaryOperator, operand: &AstNode) -> CompilerResult<T>;
    fn visit_assignment(&mut self, target: &AstNode, value: &AstNode) -> CompilerResult<T>;
    fn visit_function_call(&mut self, name: &str, arguments: &[AstNode]) -> CompilerResult<T>;
    fn visit_array_access(&mut self, array: &AstNode, index: &AstNode) -> CompilerResult<T>;
    fn visit_member_access(&mut self, object: &AstNode, member: &str) -> CompilerResult<T>;
    fn visit_integer(&mut self, value: i32) -> CompilerResult<T>;
    fn visit_float(&mut self, value: f32) -> CompilerResult<T>;
    fn visit_string(&mut self, value: &str) -> CompilerResult<T>;
    fn visit_character(&mut self, value: char) -> CompilerResult<T>;
    fn visit_boolean(&mut self, value: bool) -> CompilerResult<T>;
    fn visit_identifier(&mut self, name: &str) -> CompilerResult<T>;
    fn visit_type_definition(
        &mut self,
        name: &str,
        definition: &TypeDefinition,
    ) -> CompilerResult<T>;
    fn visit_enum_definition(&mut self, name: &str, variants: &[EnumVariant]) -> CompilerResult<T>;
}

impl AstNode {
    /// Accept a visitor
    pub fn accept<T>(&self, visitor: &mut dyn AstVisitor<T>) -> CompilerResult<T> {
        match self {
            AstNode::Program(nodes) => visitor.visit_program(nodes),
            AstNode::Function {
                name,
                parameters,
                return_type,
                body,
                is_public,
                is_native,
                is_forward,
            } => visitor.visit_function(
                name,
                parameters,
                return_type,
                body,
                *is_public,
                *is_native,
                *is_forward,
            ),
            AstNode::VariableDeclaration {
                name,
                var_type,
                initializer,
                is_const,
                is_static,
            } => visitor.visit_variable_declaration(
                name,
                var_type,
                initializer,
                *is_const,
                *is_static,
            ),
            AstNode::Block(statements) => visitor.visit_block(statements),
            AstNode::Expression(expr) => expr.accept(visitor),
            AstNode::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if(condition, then_branch, else_branch),
            AstNode::While { condition, body } => visitor.visit_while(condition, body),
            AstNode::For {
                init,
                condition,
                update,
                body,
            } => visitor.visit_for(init, condition, update, body),
            AstNode::Return(value) => visitor.visit_return(value),
            AstNode::Break => visitor.visit_break(),
            AstNode::Continue => visitor.visit_continue(),
            AstNode::BinaryOp {
                left,
                operator,
                right,
            } => visitor.visit_binary_op(left, operator, right),
            AstNode::UnaryOp { operator, operand } => visitor.visit_unary_op(operator, operand),
            AstNode::Assignment { target, value } => visitor.visit_assignment(target, value),
            AstNode::FunctionCall { name, arguments } => {
                visitor.visit_function_call(name, arguments)
            }
            AstNode::ArrayAccess { array, index } => visitor.visit_array_access(array, index),
            AstNode::MemberAccess { object, member } => visitor.visit_member_access(object, member),
            AstNode::Integer(value) => visitor.visit_integer(*value),
            AstNode::Float(value) => visitor.visit_float(*value),
            AstNode::String(value) => visitor.visit_string(value),
            AstNode::Character(value) => visitor.visit_character(*value),
            AstNode::Boolean(value) => visitor.visit_boolean(*value),
            AstNode::Identifier(name) => visitor.visit_identifier(name),
            AstNode::TypeDefinition { name, definition } => {
                visitor.visit_type_definition(name, definition)
            }
            AstNode::EnumDefinition { name, variants } => {
                visitor.visit_enum_definition(name, variants)
            }
        }
    }
}

/// Default implementation for AstVisitor
impl<T> AstVisitor<T> for Box<dyn AstVisitor<T>> {
    fn visit_program(&mut self, nodes: &[AstNode]) -> CompilerResult<T> {
        self.as_mut().visit_program(nodes)
    }

    fn visit_function(
        &mut self,
        name: &str,
        parameters: &[Parameter],
        return_type: &Option<String>,
        body: &[AstNode],
        is_public: bool,
        is_native: bool,
        is_forward: bool,
    ) -> CompilerResult<T> {
        self.as_mut().visit_function(
            name,
            parameters,
            return_type,
            body,
            is_public,
            is_native,
            is_forward,
        )
    }

    fn visit_variable_declaration(
        &mut self,
        name: &str,
        var_type: &str,
        initializer: &Option<Box<AstNode>>,
        is_const: bool,
        is_static: bool,
    ) -> CompilerResult<T> {
        self.as_mut()
            .visit_variable_declaration(name, var_type, initializer, is_const, is_static)
    }

    fn visit_block(&mut self, statements: &[AstNode]) -> CompilerResult<T> {
        self.as_mut().visit_block(statements)
    }

    fn visit_if(
        &mut self,
        condition: &AstNode,
        then_branch: &AstNode,
        else_branch: &Option<Box<AstNode>>,
    ) -> CompilerResult<T> {
        self.as_mut().visit_if(condition, then_branch, else_branch)
    }

    fn visit_while(&mut self, condition: &AstNode, body: &AstNode) -> CompilerResult<T> {
        self.as_mut().visit_while(condition, body)
    }

    fn visit_for(
        &mut self,
        init: &Option<Box<AstNode>>,
        condition: &Option<Box<AstNode>>,
        update: &Option<Box<AstNode>>,
        body: &AstNode,
    ) -> CompilerResult<T> {
        self.as_mut().visit_for(init, condition, update, body)
    }

    fn visit_return(&mut self, value: &Option<Box<AstNode>>) -> CompilerResult<T> {
        self.as_mut().visit_return(value)
    }

    fn visit_break(&mut self) -> CompilerResult<T> {
        self.as_mut().visit_break()
    }

    fn visit_continue(&mut self) -> CompilerResult<T> {
        self.as_mut().visit_continue()
    }

    fn visit_binary_op(
        &mut self,
        left: &AstNode,
        operator: &BinaryOperator,
        right: &AstNode,
    ) -> CompilerResult<T> {
        self.as_mut().visit_binary_op(left, operator, right)
    }

    fn visit_unary_op(&mut self, operator: &UnaryOperator, operand: &AstNode) -> CompilerResult<T> {
        self.as_mut().visit_unary_op(operator, operand)
    }

    fn visit_assignment(&mut self, target: &AstNode, value: &AstNode) -> CompilerResult<T> {
        self.as_mut().visit_assignment(target, value)
    }

    fn visit_function_call(&mut self, name: &str, arguments: &[AstNode]) -> CompilerResult<T> {
        self.as_mut().visit_function_call(name, arguments)
    }

    fn visit_array_access(&mut self, array: &AstNode, index: &AstNode) -> CompilerResult<T> {
        self.as_mut().visit_array_access(array, index)
    }

    fn visit_member_access(&mut self, object: &AstNode, member: &str) -> CompilerResult<T> {
        self.as_mut().visit_member_access(object, member)
    }

    fn visit_integer(&mut self, value: i32) -> CompilerResult<T> {
        self.as_mut().visit_integer(value)
    }

    fn visit_float(&mut self, value: f32) -> CompilerResult<T> {
        self.as_mut().visit_float(value)
    }

    fn visit_string(&mut self, value: &str) -> CompilerResult<T> {
        self.as_mut().visit_string(value)
    }

    fn visit_character(&mut self, value: char) -> CompilerResult<T> {
        self.as_mut().visit_character(value)
    }

    fn visit_boolean(&mut self, value: bool) -> CompilerResult<T> {
        self.as_mut().visit_boolean(value)
    }

    fn visit_identifier(&mut self, name: &str) -> CompilerResult<T> {
        self.as_mut().visit_identifier(name)
    }

    fn visit_type_definition(
        &mut self,
        name: &str,
        definition: &TypeDefinition,
    ) -> CompilerResult<T> {
        self.as_mut().visit_type_definition(name, definition)
    }

    fn visit_enum_definition(&mut self, name: &str, variants: &[EnumVariant]) -> CompilerResult<T> {
        self.as_mut().visit_enum_definition(name, variants)
    }
}
