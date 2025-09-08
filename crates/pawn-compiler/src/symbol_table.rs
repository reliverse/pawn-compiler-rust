//! Symbol table for Pawn compiler

use crate::ast::*;
use crate::error::*;
use std::collections::HashMap;

/// Symbol table entry
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub scope_level: usize,
    pub is_defined: bool,
}

/// Types of symbols
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function {
        parameters: Vec<Parameter>,
        return_type: Option<String>,
        is_public: bool,
        is_native: bool,
        is_forward: bool,
    },
    Variable {
        var_type: String,
        is_const: bool,
        is_static: bool,
        offset: Option<usize>,
    },
    Type {
        definition: TypeDefinition,
    },
    Enum {
        variants: Vec<EnumVariant>,
    },
}

/// Symbol table for managing identifiers
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    scope_stack: Vec<Vec<String>>,
    current_scope: usize,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            scope_stack: vec![Vec::new()],
            current_scope: 0,
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.current_scope += 1;
        self.scope_stack.push(Vec::new());
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            // Remove symbols from current scope
            if let Some(scope_symbols) = self.scope_stack.pop() {
                for symbol_name in scope_symbols {
                    self.symbols.remove(&symbol_name);
                }
            }
            self.current_scope -= 1;
        }
    }

    /// Add a symbol to the table
    pub fn add_symbol(&mut self, symbol: Symbol) -> CompilerResult<()> {
        let name = symbol.name.clone();

        // Check if symbol already exists in current scope
        if self.symbols.contains_key(&name) {
            return Err(CompilerError::SemanticError(format!(
                "Symbol '{}' already declared in current scope",
                name
            )));
        }

        // Add to current scope
        if let Some(current_scope) = self.scope_stack.last_mut() {
            current_scope.push(name.clone());
        }

        self.symbols.insert(name, symbol);
        Ok(())
    }

    /// Look up a symbol
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Look up a symbol in current scope only
    pub fn lookup_current_scope(&self, name: &str) -> Option<&Symbol> {
        if let Some(scope_symbols) = self.scope_stack.last() {
            if scope_symbols.contains(&name.to_string()) {
                return self.symbols.get(name);
            }
        }
        None
    }

    /// Check if symbol exists
    pub fn exists(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Get all symbols in current scope
    pub fn get_current_scope_symbols(&self) -> Vec<&Symbol> {
        let mut result = Vec::new();
        if let Some(scope_symbols) = self.scope_stack.last() {
            for symbol_name in scope_symbols {
                if let Some(symbol) = self.symbols.get(symbol_name) {
                    result.push(symbol);
                }
            }
        }
        result
    }

    /// Get current scope level
    pub fn get_scope_level(&self) -> usize {
        self.current_scope
    }

    /// Clear all symbols
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.scope_stack.clear();
        self.scope_stack.push(Vec::new());
        self.current_scope = 0;
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol table visitor for AST analysis
pub struct SymbolTableVisitor {
    symbol_table: SymbolTable,
    errors: Vec<CompilerError>,
}

impl SymbolTableVisitor {
    /// Create a new symbol table visitor
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
        }
    }

    /// Analyze AST and build symbol table
    pub fn analyze(&mut self, ast: &AstNode) -> CompilerResult<()> {
        self.errors.clear();
        self.symbol_table.clear();

        // Add built-in functions
        let printf_symbol = Symbol {
            name: "printf".to_string(),
            symbol_type: SymbolType::Function {
                parameters: vec![Parameter {
                    name: "format".to_string(),
                    param_type: "string".to_string(),
                    is_reference: false,
                    default_value: None,
                }],
                return_type: Some("int".to_string()),
                is_public: true,
                is_native: true,
                is_forward: false,
            },
            scope_level: 0,
            is_defined: true,
        };
        self.symbol_table.add_symbol(printf_symbol).ok();

        match ast.accept::<()>(self) {
            Ok(_) => {
                if self.errors.is_empty() {
                    Ok(())
                } else {
                    Err(self.errors[0].clone())
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Get the symbol table
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Get errors
    pub fn get_errors(&self) -> &[CompilerError] {
        &self.errors
    }
}

impl AstVisitor<()> for SymbolTableVisitor {
    fn visit_program(&mut self, nodes: &[AstNode]) -> CompilerResult<()> {
        for node in nodes {
            node.accept(self)?;
        }
        Ok(())
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
    ) -> CompilerResult<()> {
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::Function {
                parameters: parameters.to_vec(),
                return_type: return_type.clone(),
                is_public,
                is_native,
                is_forward,
            },
            scope_level: self.symbol_table.get_scope_level(),
            is_defined: true,
        };

        if let Err(e) = self.symbol_table.add_symbol(symbol) {
            self.errors.push(e);
        }

        // Enter function scope
        self.symbol_table.enter_scope();

        // Add parameters to symbol table
        for param in parameters {
            let param_symbol = Symbol {
                name: param.name.clone(),
                symbol_type: SymbolType::Variable {
                    var_type: param.param_type.clone(),
                    is_const: false,
                    is_static: false,
                    offset: None,
                },
                scope_level: self.symbol_table.get_scope_level(),
                is_defined: true,
            };

            if let Err(e) = self.symbol_table.add_symbol(param_symbol) {
                self.errors.push(e);
            }
        }

        // Analyze function body
        for stmt in body {
            stmt.accept(self)?;
        }

        // Exit function scope
        self.symbol_table.exit_scope();

        Ok(())
    }

    fn visit_variable_declaration(
        &mut self,
        name: &str,
        var_type: &str,
        initializer: &Option<Box<AstNode>>,
        is_const: bool,
        is_static: bool,
    ) -> CompilerResult<()> {
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::Variable {
                var_type: var_type.to_string(),
                is_const,
                is_static,
                offset: None,
            },
            scope_level: self.symbol_table.get_scope_level(),
            is_defined: true,
        };

        if let Err(e) = self.symbol_table.add_symbol(symbol) {
            self.errors.push(e);
        }

        // Analyze initializer if present
        if let Some(init) = initializer {
            init.accept(self)?;
        }

        Ok(())
    }

    fn visit_block(&mut self, statements: &[AstNode]) -> CompilerResult<()> {
        self.symbol_table.enter_scope();

        for stmt in statements {
            stmt.accept(self)?;
        }

        self.symbol_table.exit_scope();
        Ok(())
    }

    fn visit_identifier(&mut self, name: &str) -> CompilerResult<()> {
        if self.symbol_table.lookup(name).is_none() {
            self.errors.push(CompilerError::SemanticError(format!(
                "Undefined identifier: {}",
                name
            )));
        }
        Ok(())
    }

    // Default implementations for other visitor methods
    fn visit_if(
        &mut self,
        condition: &AstNode,
        then_branch: &AstNode,
        else_branch: &Option<Box<AstNode>>,
    ) -> CompilerResult<()> {
        condition.accept(self)?;
        then_branch.accept(self)?;
        if let Some(else_stmt) = else_branch {
            else_stmt.accept(self)?;
        }
        Ok(())
    }

    fn visit_while(&mut self, condition: &AstNode, body: &AstNode) -> CompilerResult<()> {
        condition.accept(self)?;
        body.accept(self)?;
        Ok(())
    }

    fn visit_for(
        &mut self,
        init: &Option<Box<AstNode>>,
        condition: &Option<Box<AstNode>>,
        update: &Option<Box<AstNode>>,
        body: &AstNode,
    ) -> CompilerResult<()> {
        if let Some(init_stmt) = init {
            init_stmt.accept(self)?;
        }
        if let Some(cond) = condition {
            cond.accept(self)?;
        }
        body.accept(self)?;
        if let Some(update_stmt) = update {
            update_stmt.accept(self)?;
        }
        Ok(())
    }

    fn visit_return(&mut self, value: &Option<Box<AstNode>>) -> CompilerResult<()> {
        if let Some(val) = value {
            val.accept(self)?;
        }
        Ok(())
    }

    fn visit_break(&mut self) -> CompilerResult<()> {
        Ok(())
    }

    fn visit_continue(&mut self) -> CompilerResult<()> {
        Ok(())
    }

    fn visit_binary_op(
        &mut self,
        left: &AstNode,
        _operator: &BinaryOperator,
        right: &AstNode,
    ) -> CompilerResult<()> {
        left.accept(self)?;
        right.accept(self)?;
        Ok(())
    }

    fn visit_unary_op(
        &mut self,
        _operator: &UnaryOperator,
        operand: &AstNode,
    ) -> CompilerResult<()> {
        operand.accept(self)?;
        Ok(())
    }

    fn visit_assignment(&mut self, target: &AstNode, value: &AstNode) -> CompilerResult<()> {
        target.accept(self)?;
        value.accept(self)?;
        Ok(())
    }

    fn visit_function_call(&mut self, name: &str, arguments: &[AstNode]) -> CompilerResult<()> {
        if self.symbol_table.lookup(name).is_none() {
            self.errors.push(CompilerError::SemanticError(format!(
                "Undefined function: {}",
                name
            )));
        }

        for arg in arguments {
            arg.accept(self)?;
        }
        Ok(())
    }

    fn visit_array_access(&mut self, array: &AstNode, index: &AstNode) -> CompilerResult<()> {
        array.accept(self)?;
        index.accept(self)?;
        Ok(())
    }

    fn visit_member_access(&mut self, object: &AstNode, _member: &str) -> CompilerResult<()> {
        object.accept(self)?;
        Ok(())
    }

    fn visit_integer(&mut self, _value: i32) -> CompilerResult<()> {
        Ok(())
    }

    fn visit_float(&mut self, _value: f32) -> CompilerResult<()> {
        Ok(())
    }

    fn visit_string(&mut self, _value: &str) -> CompilerResult<()> {
        Ok(())
    }

    fn visit_character(&mut self, _value: char) -> CompilerResult<()> {
        Ok(())
    }

    fn visit_boolean(&mut self, _value: bool) -> CompilerResult<()> {
        Ok(())
    }

    fn visit_type_definition(
        &mut self,
        name: &str,
        definition: &TypeDefinition,
    ) -> CompilerResult<()> {
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::Type {
                definition: definition.clone(),
            },
            scope_level: self.symbol_table.get_scope_level(),
            is_defined: true,
        };

        if let Err(e) = self.symbol_table.add_symbol(symbol) {
            self.errors.push(e);
        }

        Ok(())
    }

    fn visit_enum_definition(
        &mut self,
        name: &str,
        variants: &[EnumVariant],
    ) -> CompilerResult<()> {
        let symbol = Symbol {
            name: name.to_string(),
            symbol_type: SymbolType::Enum {
                variants: variants.to_vec(),
            },
            scope_level: self.symbol_table.get_scope_level(),
            is_defined: true,
        };

        if let Err(e) = self.symbol_table.add_symbol(symbol) {
            self.errors.push(e);
        }

        Ok(())
    }
}
