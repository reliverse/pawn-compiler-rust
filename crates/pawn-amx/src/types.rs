//! Core types and constants for the AMX runtime

use std::error::Error;
use std::fmt;

/// Cell size in bits - Pawn uses 32-bit cells by default
pub const PAWN_CELL_SIZE: usize = 32;

/// Cell type - 32-bit signed integer
pub type Cell = i32;

/// Unsigned cell type - 32-bit unsigned integer  
pub type UCell = u32;

/// Native function pointer type
pub type NativeFunction = fn(amx: &mut Amx, params: &[Cell]) -> Cell;

/// Callback function type
pub type CallbackFunction =
    fn(amx: &mut Amx, index: Cell, result: &mut Cell, params: &[Cell]) -> i32;

/// Debug function type
pub type DebugFunction = fn(amx: &mut Amx) -> i32;

/// Idle function type
pub type IdleFunction = fn(amx: &mut Amx, exec: fn(&mut Amx, &mut Cell, i32) -> i32) -> i32;

/// AMX magic numbers for different cell sizes
pub const AMX_MAGIC_16: u16 = 0xf1e2;
pub const AMX_MAGIC_32: u16 = 0xf1e0;
pub const AMX_MAGIC_64: u16 = 0xf1e1;

/// Current AMX magic number based on cell size
pub const AMX_MAGIC: u16 = AMX_MAGIC_32;

/// Maximum name length for symbols
pub const SNAMEMAX: usize = 31;

/// Maximum expression length for file version <= 6
pub const SEXPMAX: usize = 19;

/// Number of user data fields
pub const AMX_USERNUM: usize = 4;

/// Stack margin for safety
pub const STKMARGIN: Cell = 16 * std::mem::size_of::<Cell>() as Cell;

/// Unpacked maximum value
pub const UNPACKEDMAX: UCell = ((1u64 << (std::mem::size_of::<UCell>() - 1) * 8) - 1) as UCell;

/// Unlimited value
pub const UNLIMITED: Cell = (!1u32 >> 1) as Cell;

/// AMX execution modes
pub const AMX_EXEC_MAIN: i32 = -1; // Start at program entry point
pub const AMX_EXEC_CONT: i32 = -2; // Continue from last address

/// AMX flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmxFlags {
    pub debug: bool,      // 0x02 - Symbolic info available
    pub compact: bool,    // 0x04 - Compact encoding
    pub sleep: bool,      // 0x08 - Script uses sleep instruction
    pub no_checks: bool,  // 0x10 - No array bounds checking
    pub no_reloc: bool,   // 0x200 - No reallocations done
    pub no_sysreqd: bool, // 0x400 - SYSREQ.D is NOT used
    pub sysreqn: bool,    // 0x800 - Script new optimized SYSREQ
    pub ntvreg: bool,     // 0x1000 - All native functions registered
    pub jitc: bool,       // 0x2000 - Abstract machine is JIT compiled
    pub browse: bool,     // 0x4000 - Busy browsing
    pub reloc: bool,      // 0x8000 - Jump/call addresses relocated
}

impl AmxFlags {
    pub fn new() -> Self {
        Self {
            debug: false,
            compact: false,
            sleep: false,
            no_checks: false,
            no_reloc: false,
            no_sysreqd: false,
            sysreqn: false,
            ntvreg: false,
            jitc: false,
            browse: false,
            reloc: false,
        }
    }

    pub fn from_bits(bits: u16) -> Self {
        Self {
            debug: (bits & 0x02) != 0,
            compact: (bits & 0x04) != 0,
            sleep: (bits & 0x08) != 0,
            no_checks: (bits & 0x10) != 0,
            no_reloc: (bits & 0x200) != 0,
            no_sysreqd: (bits & 0x400) != 0,
            sysreqn: (bits & 0x800) != 0,
            ntvreg: (bits & 0x1000) != 0,
            jitc: (bits & 0x2000) != 0,
            browse: (bits & 0x4000) != 0,
            reloc: (bits & 0x8000) != 0,
        }
    }

    pub fn to_bits(self) -> u16 {
        let mut bits = 0u16;
        if self.debug {
            bits |= 0x02;
        }
        if self.compact {
            bits |= 0x04;
        }
        if self.sleep {
            bits |= 0x08;
        }
        if self.no_checks {
            bits |= 0x10;
        }
        if self.no_reloc {
            bits |= 0x200;
        }
        if self.no_sysreqd {
            bits |= 0x400;
        }
        if self.sysreqn {
            bits |= 0x800;
        }
        if self.ntvreg {
            bits |= 0x1000;
        }
        if self.jitc {
            bits |= 0x2000;
        }
        if self.browse {
            bits |= 0x4000;
        }
        if self.reloc {
            bits |= 0x8000;
        }
        bits
    }
}

impl Default for AmxFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// AMX structure representing the virtual machine state
#[derive(Debug)]
pub struct Amx {
    /// Points to the AMX header plus the code, optionally also the data
    pub base: Vec<u8>,
    /// Points to separate data+stack+heap, may be NULL
    pub data: Option<Vec<u8>>,
    /// Callback function
    pub callback: Option<CallbackFunction>,
    /// Debug function
    pub debug: Option<DebugFunction>,
    /// Instruction pointer: relative to base + amxhdr->cod
    pub cip: Cell,
    /// Stack frame base: relative to base + amxhdr->dat
    pub frm: Cell,
    /// Top of the heap: relative to base + amxhdr->dat
    pub hea: Cell,
    /// Bottom of the heap: relative to base + amxhdr->dat
    pub hlw: Cell,
    /// Stack pointer: relative to base + amxhdr->dat
    pub stk: Cell,
    /// Top of the stack: relative to base + amxhdr->dat
    pub stp: Cell,
    /// Current status flags
    pub flags: AmxFlags,
    /// User data fields
    pub usertags: [i64; AMX_USERNUM],
    pub userdata: [Option<Box<dyn std::any::Any>>; AMX_USERNUM],
    /// Native functions can raise an error
    pub error: i32,
    /// Passing parameters requires a "count" field
    pub paramcount: i32,
    /// The sleep opcode needs to store the full AMX status
    pub pri: Cell,
    pub alt: Cell,
    pub reset_stk: Cell,
    pub reset_hea: Cell,
    /// Extra fields for increased performance
    pub sysreq_d: Cell,
    /// Support variables for the JIT
    pub reloc_size: i32,
    pub code_size: i64,
}

impl Amx {
    pub fn new() -> Self {
        Self {
            base: Vec::new(),
            data: None,
            callback: None,
            debug: None,
            cip: 0,
            frm: 0,
            hea: 0,
            hlw: 0,
            stk: 0,
            stp: 0,
            flags: AmxFlags::new(),
            usertags: [0; AMX_USERNUM],
            userdata: [None, None, None, None],
            error: 0,
            paramcount: 0,
            pri: 0,
            alt: 0,
            reset_stk: 0,
            reset_hea: 0,
            sysreq_d: 0,
            reloc_size: 0,
            code_size: 0,
        }
    }
}

impl Default for Amx {
    fn default() -> Self {
        Self::new()
    }
}

/// Native function information
#[derive(Debug, Clone)]
pub struct NativeInfo {
    pub name: String,
    pub func: NativeFunction,
}

impl NativeInfo {
    pub fn new(name: String, func: NativeFunction) -> Self {
        Self { name, func }
    }
}

/// Function stub for public functions
#[derive(Debug, Clone)]
pub struct FuncStub {
    pub address: UCell,
    pub name: String,
}

impl FuncStub {
    pub fn new(address: UCell, name: String) -> Self {
        Self { address, name }
    }
}

/// Function stub with name table offset
#[derive(Debug, Clone)]
pub struct FuncStubNt {
    pub address: UCell,
    pub nameofs: u32,
}

impl FuncStubNt {
    pub fn new(address: UCell, nameofs: u32) -> Self {
        Self { address, nameofs }
    }
}

/// Public variable information
#[derive(Debug, Clone)]
pub struct PubVar {
    pub address: UCell,
    pub name: String,
}

impl PubVar {
    pub fn new(address: UCell, name: String) -> Self {
        Self { address, name }
    }
}

/// Tag information
#[derive(Debug, Clone)]
pub struct TagInfo {
    pub tag_id: Cell,
    pub name: String,
}

impl TagInfo {
    pub fn new(tag_id: Cell, name: String) -> Self {
        Self { tag_id, name }
    }
}

/// AMX error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmxError {
    None = 0,
    Exit = 1,
    Assert = 2,
    StackErr = 3,
    Bounds = 4,
    MemAccess = 5,
    InvInstr = 6,
    StackLow = 7,
    HeapLow = 8,
    Callback = 9,
    Native = 10,
    Divide = 11,
    Sleep = 12,
    InvState = 13,
    Memory = 16,
    Format = 17,
    Version = 18,
    NotFound = 19,
    Index = 20,
    Debug = 21,
    Init = 22,
    UserData = 23,
    InitJit = 24,
    Params = 25,
    Domain = 26,
    General = 27,
}

impl fmt::Display for AmxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AmxError::None => write!(f, "No error"),
            AmxError::Exit => write!(f, "Forced exit"),
            AmxError::Assert => write!(f, "Assertion failed"),
            AmxError::StackErr => write!(f, "Stack/heap collision"),
            AmxError::Bounds => write!(f, "Index out of bounds"),
            AmxError::MemAccess => write!(f, "Invalid memory access"),
            AmxError::InvInstr => write!(f, "Invalid instruction"),
            AmxError::StackLow => write!(f, "Stack underflow"),
            AmxError::HeapLow => write!(f, "Heap underflow"),
            AmxError::Callback => write!(f, "No callback, or invalid callback"),
            AmxError::Native => write!(f, "Native function failed"),
            AmxError::Divide => write!(f, "Divide by zero"),
            AmxError::Sleep => write!(f, "Go into sleepmode - code can be restarted"),
            AmxError::InvState => write!(f, "Invalid state for this access"),
            AmxError::Memory => write!(f, "Out of memory"),
            AmxError::Format => write!(f, "Invalid file format"),
            AmxError::Version => write!(f, "File is for a newer version of the AMX"),
            AmxError::NotFound => write!(f, "Function not found"),
            AmxError::Index => write!(f, "Invalid index parameter"),
            AmxError::Debug => write!(f, "Debugger cannot run"),
            AmxError::Init => write!(f, "AMX not initialized"),
            AmxError::UserData => write!(f, "Unable to set user data field"),
            AmxError::InitJit => write!(f, "Cannot initialize the JIT"),
            AmxError::Params => write!(f, "Parameter error"),
            AmxError::Domain => write!(f, "Domain error"),
            AmxError::General => write!(f, "General error"),
        }
    }
}

impl From<i32> for AmxError {
    fn from(value: i32) -> Self {
        match value {
            0 => AmxError::None,
            1 => AmxError::Exit,
            2 => AmxError::Assert,
            3 => AmxError::StackErr,
            4 => AmxError::Bounds,
            5 => AmxError::MemAccess,
            6 => AmxError::InvInstr,
            7 => AmxError::StackLow,
            8 => AmxError::HeapLow,
            9 => AmxError::Callback,
            10 => AmxError::Native,
            11 => AmxError::Divide,
            12 => AmxError::Sleep,
            13 => AmxError::InvState,
            16 => AmxError::Memory,
            17 => AmxError::Format,
            18 => AmxError::Version,
            19 => AmxError::NotFound,
            20 => AmxError::Index,
            21 => AmxError::Debug,
            22 => AmxError::Init,
            23 => AmxError::UserData,
            24 => AmxError::InitJit,
            25 => AmxError::Params,
            26 => AmxError::Domain,
            27 => AmxError::General,
            _ => AmxError::General,
        }
    }
}

impl From<AmxError> for i32 {
    fn from(error: AmxError) -> Self {
        error as i32
    }
}

impl Error for AmxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
