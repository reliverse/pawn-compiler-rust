//! AMX header structure and related functionality

use crate::types::*;
use std::fmt;

/// AMX header structure - both memory and file format
#[derive(Debug, Clone)]
pub struct AmxHeader {
    /// Size of the "file"
    pub size: i32,
    /// Signature (magic number)
    pub magic: u16,
    /// File format version
    pub file_version: u8,
    /// Required version of the AMX
    pub amx_version: u8,
    /// Flags
    pub flags: u16,
    /// Size of a definition record
    pub defsize: i16,
    /// Initial value of COD - code block
    pub cod: i32,
    /// Initial value of DAT - data block
    pub dat: i32,
    /// Initial value of HEA - start of the heap
    pub hea: i32,
    /// Initial value of STP - stack top
    pub stp: i32,
    /// Initial value of CIP - the instruction pointer
    pub cip: i32,
    /// Offset to the "public functions" table
    pub publics: i32,
    /// Offset to the "native functions" table
    pub natives: i32,
    /// Offset to the table of libraries
    pub libraries: i32,
    /// The "public variables" table
    pub pubvars: i32,
    /// The "public tagnames" table
    pub tags: i32,
    /// Name table
    pub nametable: i32,
}

impl AmxHeader {
    /// Create a new AMX header with default values
    pub fn new() -> Self {
        Self {
            size: 0,
            magic: AMX_MAGIC,
            file_version: 9, // Current file version
            amx_version: 10, // Minimum AMX version
            flags: 0,
            defsize: 0,
            cod: 0,
            dat: 0,
            hea: 0,
            stp: 0,
            cip: 0,
            publics: 0,
            natives: 0,
            libraries: 0,
            pubvars: 0,
            tags: 0,
            nametable: 0,
        }
    }

    /// Check if the header uses name table
    pub fn uses_name_table(&self) -> bool {
        self.defsize == std::mem::size_of::<FuncStubNt>() as i16
    }

    /// Get the number of entries in a table
    pub fn num_entries(&self, field: i32, next_field: i32) -> usize {
        ((next_field - field) / self.defsize as i32) as usize
    }

    /// Get an entry from a table
    pub fn get_entry<'a>(&self, base: &'a [u8], table: i32, index: usize) -> &'a [u8] {
        let offset = table as usize + index * self.defsize as usize;
        &base[offset..offset + self.defsize as usize]
    }

    /// Get entry name (works for both FuncStub and FuncStubNt)
    pub fn get_entry_name<'a>(&self, base: &'a [u8], entry: &'a [u8]) -> &'a str {
        if self.uses_name_table() {
            // FuncStubNt - name is stored in name table
            let nameofs = u32::from_le_bytes([entry[4], entry[5], entry[6], entry[7]]);
            let name_start = nameofs as usize;
            let name_end = base[name_start..].iter().position(|&b| b == 0).unwrap_or(0);
            std::str::from_utf8(&base[name_start..name_start + name_end]).unwrap_or("")
        } else {
            // FuncStub - name is stored directly in entry
            let name_end = entry[4..].iter().position(|&b| b == 0).unwrap_or(SEXPMAX);
            std::str::from_utf8(&entry[4..4 + name_end]).unwrap_or("")
        }
    }

    /// Validate the header
    pub fn validate(&self) -> Result<(), AmxError> {
        if self.magic != AMX_MAGIC {
            return Err(AmxError::Format);
        }

        if self.file_version < 6 {
            return Err(AmxError::Version);
        }

        if self.amx_version > 10 {
            return Err(AmxError::Version);
        }

        Ok(())
    }
}

impl Default for AmxHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AmxHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AMX Header:\n")?;
        write!(f, "  Size: {}\n", self.size)?;
        write!(f, "  Magic: 0x{:04x}\n", self.magic)?;
        write!(f, "  File Version: {}\n", self.file_version)?;
        write!(f, "  AMX Version: {}\n", self.amx_version)?;
        write!(f, "  Flags: 0x{:04x}\n", self.flags)?;
        write!(f, "  Def Size: {}\n", self.defsize)?;
        write!(f, "  Code: 0x{:08x}\n", self.cod)?;
        write!(f, "  Data: 0x{:08x}\n", self.dat)?;
        write!(f, "  Heap: 0x{:08x}\n", self.hea)?;
        write!(f, "  Stack Top: 0x{:08x}\n", self.stp)?;
        write!(f, "  CIP: 0x{:08x}\n", self.cip)?;
        write!(f, "  Publics: 0x{:08x}\n", self.publics)?;
        write!(f, "  Natives: 0x{:08x}\n", self.natives)?;
        write!(f, "  Libraries: 0x{:08x}\n", self.libraries)?;
        write!(f, "  Pub Vars: 0x{:08x}\n", self.pubvars)?;
        write!(f, "  Tags: 0x{:08x}\n", self.tags)?;
        write!(f, "  Name Table: 0x{:08x}\n", self.nametable)
    }
}

/// Read AMX header from bytes
pub fn read_header(data: &[u8]) -> Result<AmxHeader, AmxError> {
    if data.len() < std::mem::size_of::<AmxHeader>() {
        return Err(AmxError::Format);
    }

    let mut header = AmxHeader::new();
    let mut offset = 0;

    // Read fields in order
    header.size = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.magic = u16::from_le_bytes([data[offset], data[offset + 1]]);
    offset += 2;

    header.file_version = data[offset];
    offset += 1;

    header.amx_version = data[offset];
    offset += 1;

    header.flags = u16::from_le_bytes([data[offset], data[offset + 1]]);
    offset += 2;

    header.defsize = i16::from_le_bytes([data[offset], data[offset + 1]]);
    offset += 2;

    header.cod = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.dat = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.hea = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.stp = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.cip = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.publics = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.natives = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.libraries = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.pubvars = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.tags = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    header.nametable = i32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);

    header.validate()?;
    Ok(header)
}

/// Write AMX header to bytes
pub fn write_header(header: &AmxHeader) -> Vec<u8> {
    let mut data = Vec::with_capacity(std::mem::size_of::<AmxHeader>());

    data.extend_from_slice(&header.size.to_le_bytes());
    data.extend_from_slice(&header.magic.to_le_bytes());
    data.push(header.file_version);
    data.push(header.amx_version);
    data.extend_from_slice(&header.flags.to_le_bytes());
    data.extend_from_slice(&header.defsize.to_le_bytes());
    data.extend_from_slice(&header.cod.to_le_bytes());
    data.extend_from_slice(&header.dat.to_le_bytes());
    data.extend_from_slice(&header.hea.to_le_bytes());
    data.extend_from_slice(&header.stp.to_le_bytes());
    data.extend_from_slice(&header.cip.to_le_bytes());
    data.extend_from_slice(&header.publics.to_le_bytes());
    data.extend_from_slice(&header.natives.to_le_bytes());
    data.extend_from_slice(&header.libraries.to_le_bytes());
    data.extend_from_slice(&header.pubvars.to_le_bytes());
    data.extend_from_slice(&header.tags.to_le_bytes());
    data.extend_from_slice(&header.nametable.to_le_bytes());

    data
}
