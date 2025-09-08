//! AMX runtime implementation

use crate::error::*;
use crate::header::*;
use crate::instructions::*;
use crate::types::*;
use std::collections::HashMap;

/// AMX runtime for executing bytecode
pub struct AmxRuntime {
    /// The AMX instance
    pub amx: Amx,
    /// Native functions registry
    natives: HashMap<String, NativeInfo>,
    /// Public functions registry
    publics: HashMap<String, FuncStub>,
    /// Public variables registry
    pubvars: HashMap<String, PubVar>,
    /// Tags registry
    tags: HashMap<String, TagInfo>,
}

impl AmxRuntime {
    /// Create a new AMX runtime
    pub fn new() -> Self {
        Self {
            amx: Amx::new(),
            natives: HashMap::new(),
            publics: HashMap::new(),
            pubvars: HashMap::new(),
            tags: HashMap::new(),
        }
    }

    /// Initialize AMX from bytecode
    pub fn init(&mut self, bytecode: &[u8]) -> AmxResult<()> {
        // Read and validate header
        let header = read_header(bytecode)?;

        // Set up AMX state
        self.amx.base = bytecode.to_vec();
        // Start executing at the beginning of the code section
        self.amx.cip = header.cod;
        self.amx.frm = header.dat;
        self.amx.hea = header.hea;
        self.amx.stp = header.stp;
        self.amx.stk = header.dat;
        self.amx.hlw = header.dat;

        // Load symbol tables
        self.load_publics(&header)?;
        self.load_natives(&header)?;
        self.load_pubvars(&header)?;
        self.load_tags(&header)?;

        Ok(())
    }

    /// Execute AMX bytecode
    pub fn exec(&mut self, index: i32) -> AmxResult<Cell> {
        if index == AMX_EXEC_MAIN {
            // Entry point already set during init; do not override
        } else if index == AMX_EXEC_CONT {
            // Continue from current position
            // No change needed
        } else {
            // Jump to specific function
            if let Some(func) = self.publics.get(&format!("func_{}", index)) {
                self.amx.cip = func.address as Cell;
            } else {
                return Err(AmxRuntimeError::PublicNotFound(format!("func_{}", index)));
            }
        }

        let mut _retval = 0;
        self.execute_instructions(&mut _retval)?;
        Ok(0)
    }

    /// Execute instructions until completion
    fn execute_instructions(&mut self, _retval: &mut Cell) -> AmxResult<()> {
        loop {
            // Check bounds
            if self.amx.cip as usize >= self.amx.base.len() {
                break;
            }

            // Read instruction
            let instruction = Instruction::from_bytes(&self.amx.base, self.amx.cip as usize)?;

            // Execute instruction
            match self.execute_instruction(instruction, _retval) {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    self.amx.error = 1; // Generic error for now
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Execute a single instruction
    fn execute_instruction(
        &mut self,
        instruction: Instruction,
        _retval: &mut Cell,
    ) -> AmxResult<bool> {
        match instruction.opcode {
            Opcode::Nop => {
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Halt => Ok(false),

            Opcode::ConstPri => {
                self.amx.pri = instruction.operand;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::ConstAlt => {
                self.amx.alt = instruction.operand;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Add => {
                self.amx.pri = self.amx.pri.wrapping_add(self.amx.alt);
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Sub => {
                self.amx.pri = self.amx.pri.wrapping_sub(self.amx.alt);
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Smul => {
                self.amx.pri = self.amx.pri.wrapping_mul(self.amx.alt);
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Sdiv => {
                if self.amx.alt == 0 {
                    return Err(AmxRuntimeError::DomainError("Division by zero".to_string()));
                }
                self.amx.pri = self.amx.pri.wrapping_div(self.amx.alt);
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Eq => {
                self.amx.pri = if self.amx.pri == self.amx.alt { 1 } else { 0 };
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Neq => {
                self.amx.pri = if self.amx.pri != self.amx.alt { 1 } else { 0 };
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Less => {
                self.amx.pri = if self.amx.pri < self.amx.alt { 1 } else { 0 };
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Leq => {
                self.amx.pri = if self.amx.pri <= self.amx.alt { 1 } else { 0 };
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Grtr => {
                self.amx.pri = if self.amx.pri > self.amx.alt { 1 } else { 0 };
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Geq => {
                self.amx.pri = if self.amx.pri >= self.amx.alt { 1 } else { 0 };
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Jump => {
                self.amx.cip = instruction.operand;
                Ok(true)
            }

            Opcode::Jzer => {
                if self.amx.pri == 0 {
                    self.amx.cip = instruction.operand;
                } else {
                    self.amx.cip += 5;
                }
                Ok(true)
            }

            Opcode::Jnz => {
                if self.amx.pri != 0 {
                    self.amx.cip = instruction.operand;
                } else {
                    self.amx.cip += 5;
                }
                Ok(true)
            }

            Opcode::Call => {
                // Push return address
                self.push_stack(self.amx.cip + 5)?;
                // Jump to function
                self.amx.cip = instruction.operand;
                Ok(true)
            }

            Opcode::Ret => {
                // Pop return address
                self.amx.cip = self.pop_stack()?;
                Ok(true)
            }

            Opcode::Retn => {
                // Pop return address and parameters
                let param_count = instruction.operand;
                self.amx.cip = self.pop_stack()?;
                self.amx.stk += param_count;
                Ok(true)
            }

            Opcode::PushPri => {
                self.push_stack(self.amx.pri)?;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::PopPri => {
                self.amx.pri = self.pop_stack()?;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::PushAlt => {
                self.push_stack(self.amx.alt)?;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::PopAlt => {
                self.amx.alt = self.pop_stack()?;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::LoadPri => {
                let addr = self.amx.frm + instruction.operand;
                self.amx.pri = self.read_cell(addr)?;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::LoadAlt => {
                let addr = self.amx.frm + instruction.operand;
                self.amx.alt = self.read_cell(addr)?;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::StorPri => {
                let addr = self.amx.frm + instruction.operand;
                self.write_cell(addr, self.amx.pri)?;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::StorAlt => {
                let addr = self.amx.frm + instruction.operand;
                self.write_cell(addr, self.amx.alt)?;
                self.amx.cip += 5;
                Ok(true)
            }

            Opcode::Sysreq => {
                // Call native function
                let native_index = instruction.operand as usize;
                if let Some(_native) = self.natives.values().nth(native_index) {
                    // For now, just set return value to 0
                    self.amx.pri = 0;
                } else {
                    return Err(AmxRuntimeError::NativeNotFound(format!(
                        "native_{}",
                        native_index
                    )));
                }
                self.amx.cip += 5;
                Ok(true)
            }

            _ => {
                // Unimplemented instruction
                self.amx.cip += 5;
                Ok(true)
            }
        }
    }

    /// Push value to stack
    fn push_stack(&mut self, value: Cell) -> AmxResult<()> {
        if self.amx.stk >= self.amx.stp {
            return Err(AmxRuntimeError::StackOverflow);
        }

        self.write_cell(self.amx.stk, value)?;
        self.amx.stk += std::mem::size_of::<Cell>() as Cell;
        Ok(())
    }

    /// Pop value from stack
    fn pop_stack(&mut self) -> AmxResult<Cell> {
        if self.amx.stk <= self.amx.frm {
            return Err(AmxRuntimeError::StackUnderflow);
        }

        self.amx.stk -= std::mem::size_of::<Cell>() as Cell;
        self.read_cell(self.amx.stk)
    }

    /// Read cell from memory
    fn read_cell(&self, addr: Cell) -> AmxResult<Cell> {
        let offset = addr as usize;
        if offset + 4 > self.amx.base.len() {
            return Err(AmxRuntimeError::InvalidMemoryAccess(offset));
        }

        Ok(Cell::from_le_bytes([
            self.amx.base[offset],
            self.amx.base[offset + 1],
            self.amx.base[offset + 2],
            self.amx.base[offset + 3],
        ]))
    }

    /// Write cell to memory
    fn write_cell(&mut self, addr: Cell, value: Cell) -> AmxResult<()> {
        let offset = addr as usize;
        if offset + 4 > self.amx.base.len() {
            return Err(AmxRuntimeError::InvalidMemoryAccess(offset));
        }

        let bytes = value.to_le_bytes();
        self.amx.base[offset..offset + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Load public functions from header
    fn load_publics(&mut self, header: &AmxHeader) -> AmxResult<()> {
        if header.publics == 0 {
            return Ok(());
        }

        let num_publics = header.num_entries(header.publics, header.natives);
        for i in 0..num_publics {
            let entry = header.get_entry(&self.amx.base, header.publics, i);
            let address = UCell::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]);
            let name = header.get_entry_name(&self.amx.base, entry);
            self.publics
                .insert(name.to_string(), FuncStub::new(address, name.to_string()));
        }

        Ok(())
    }

    /// Load native functions from header
    fn load_natives(&mut self, header: &AmxHeader) -> AmxResult<()> {
        if header.natives == 0 {
            return Ok(());
        }

        let num_natives = header.num_entries(header.natives, header.libraries);
        for i in 0..num_natives {
            let entry = header.get_entry(&self.amx.base, header.natives, i);
            let _address = UCell::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]);
            let name = header.get_entry_name(&self.amx.base, entry);
            // For now, create a dummy native function
            let native = NativeInfo::new(name.to_string(), |_amx, _params| 0);
            self.natives.insert(name.to_string(), native);
        }

        Ok(())
    }

    /// Load public variables from header
    fn load_pubvars(&mut self, header: &AmxHeader) -> AmxResult<()> {
        if header.pubvars == 0 {
            return Ok(());
        }

        let num_pubvars = header.num_entries(header.pubvars, header.tags);
        for i in 0..num_pubvars {
            let entry = header.get_entry(&self.amx.base, header.pubvars, i);
            let address = UCell::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]);
            let name = header.get_entry_name(&self.amx.base, entry);
            self.pubvars
                .insert(name.to_string(), PubVar::new(address, name.to_string()));
        }

        Ok(())
    }

    /// Load tags from header
    fn load_tags(&mut self, header: &AmxHeader) -> AmxResult<()> {
        if header.tags == 0 {
            return Ok(());
        }

        let num_tags = header.num_entries(header.tags, header.nametable);
        for i in 0..num_tags {
            let entry = header.get_entry(&self.amx.base, header.tags, i);
            let tag_id = Cell::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]);
            let name = header.get_entry_name(&self.amx.base, entry);
            self.tags
                .insert(name.to_string(), TagInfo::new(tag_id, name.to_string()));
        }

        Ok(())
    }

    /// Register a native function
    pub fn register_native(&mut self, name: String, func: NativeFunction) {
        let native = NativeInfo::new(name.clone(), func);
        self.natives.insert(name, native);
    }

    /// Find public function by name
    pub fn find_public(&self, name: &str) -> Option<&FuncStub> {
        self.publics.get(name)
    }

    /// Find native function by name
    pub fn find_native(&self, name: &str) -> Option<&NativeInfo> {
        self.natives.get(name)
    }

    /// Find public variable by name
    pub fn find_pubvar(&self, name: &str) -> Option<&PubVar> {
        self.pubvars.get(name)
    }

    /// Find tag by name
    pub fn find_tag(&self, name: &str) -> Option<&TagInfo> {
        self.tags.get(name)
    }
}

impl Default for AmxRuntime {
    fn default() -> Self {
        Self::new()
    }
}
