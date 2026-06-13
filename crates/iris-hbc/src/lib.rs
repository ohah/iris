//! Hermes bytecode metadata parsing for Iris.
//!
//! This crate only validates and exposes metadata from an immutable byte slice.
//! It does not execute bytecode and does not copy the bytecode payload.

use std::fmt;

#[cxx::bridge(namespace = "iris::hbc")]
mod ffi {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HbcMetadata {
        pub version: u32,
        pub file_length: u32,
        pub global_code_index: u32,
        pub function_count: u32,
        pub string_count: u32,
        pub cjs_module_count: u32,
        pub debug_info_offset: u32,
        pub options: u8,
    }

    extern "Rust" {
        fn parse_hbc_metadata(bytes: &[u8]) -> Result<HbcMetadata>;
    }
}

/// Hermes bytecode magic value from `BytecodeFileFormat.h`.
pub const HERMES_BYTECODE_MAGIC: u64 = 0x1F19_03C1_03BC_1FC6;

/// Number of bytes in Hermes' packed `BytecodeFileHeader`.
pub const HERMES_BYTECODE_HEADER_SIZE: usize = 128;

const HERMES_SOURCE_HASH_SIZE: usize = 20;
const MAGIC_OFFSET: usize = 0;
const VERSION_OFFSET: usize = 8;
const SOURCE_HASH_OFFSET: usize = 12;
const FILE_LENGTH_OFFSET: usize = 32;
const GLOBAL_CODE_INDEX_OFFSET: usize = 36;
const FUNCTION_COUNT_OFFSET: usize = 40;
const STRING_KIND_COUNT_OFFSET: usize = 44;
const IDENTIFIER_COUNT_OFFSET: usize = 48;
const STRING_COUNT_OFFSET: usize = 52;
const OVERFLOW_STRING_COUNT_OFFSET: usize = 56;
const STRING_STORAGE_SIZE_OFFSET: usize = 60;
const BIG_INT_COUNT_OFFSET: usize = 64;
const BIG_INT_STORAGE_SIZE_OFFSET: usize = 68;
const REG_EXP_COUNT_OFFSET: usize = 72;
const REG_EXP_STORAGE_SIZE_OFFSET: usize = 76;
const LITERAL_VALUE_BUFFER_SIZE_OFFSET: usize = 80;
const OBJ_KEY_BUFFER_SIZE_OFFSET: usize = 84;
const OBJ_SHAPE_TABLE_COUNT_OFFSET: usize = 88;
const NUM_STRING_SWITCH_IMMS_OFFSET: usize = 92;
const SEGMENT_ID_OFFSET: usize = 96;
const CJS_MODULE_COUNT_OFFSET: usize = 100;
const FUNCTION_SOURCE_COUNT_OFFSET: usize = 104;
const DEBUG_INFO_OFFSET_OFFSET: usize = 108;
const OPTIONS_OFFSET: usize = 112;

/// A validated, zero-copy view over a Hermes bytecode buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HermesBytecode<'a> {
    bytes: &'a [u8],
    header: HermesBytecodeHeader,
}

impl<'a> HermesBytecode<'a> {
    /// Parses and validates a bytecode buffer without copying the payload.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] when the buffer is too small, has a non-Hermes
    /// magic value, or declares a file length larger than the provided bytes.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ParseError> {
        let header = HermesBytecodeHeader::parse(bytes)?;
        Ok(Self { bytes, header })
    }

    /// Returns the original bytecode bytes.
    #[must_use]
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }

    /// Returns the parsed bytecode header.
    #[must_use]
    pub const fn header(self) -> HermesBytecodeHeader {
        self.header
    }
}

/// Parsed Hermes bytecode file header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HermesBytecodeHeader {
    /// Hermes bytecode format version.
    pub version: u32,
    /// SHA-1 hash of the source bytes used by Hermes.
    pub source_hash: [u8; HERMES_SOURCE_HASH_SIZE],
    /// Declared bytecode file length through the footer.
    pub file_length: u32,
    /// Function index for global code.
    pub global_code_index: u32,
    /// Number of function headers in the bytecode.
    pub function_count: u32,
    /// Number of string kind table entries.
    pub string_kind_count: u32,
    /// Number of strings that are identifiers.
    pub identifier_count: u32,
    /// Number of strings in the string table.
    pub string_count: u32,
    /// Number of overflow string table entries.
    pub overflow_string_count: u32,
    /// Size in bytes of string storage.
    pub string_storage_size: u32,
    /// Number of bigint entries.
    pub big_int_count: u32,
    /// Size in bytes of bigint storage.
    pub big_int_storage_size: u32,
    /// Number of regular expression entries.
    pub reg_exp_count: u32,
    /// Size in bytes of regular expression storage.
    pub reg_exp_storage_size: u32,
    /// Size in bytes of literal value buffers.
    pub literal_value_buffer_size: u32,
    /// Size in bytes of object key buffers.
    pub obj_key_buffer_size: u32,
    /// Number of object shape table entries.
    pub obj_shape_table_count: u32,
    /// Number of string switch immediate entries.
    pub num_string_switch_imms: u32,
    /// Hermes bytecode segment id.
    pub segment_id: u32,
    /// Number of CommonJS modules.
    pub cjs_module_count: u32,
    /// Number of preserved function sources.
    pub function_source_count: u32,
    /// Offset of debug info, or zero when absent.
    pub debug_info_offset: u32,
    /// Bytecode option flags.
    pub options: BytecodeOptions,
}

impl HermesBytecodeHeader {
    /// Parses a Hermes bytecode file header.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] when the buffer is too small, has a non-Hermes
    /// magic value, or declares a file length larger than the provided bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() < HERMES_BYTECODE_HEADER_SIZE {
            return Err(ParseError::BufferTooSmall {
                actual: bytes.len(),
                minimum: HERMES_BYTECODE_HEADER_SIZE,
            });
        }

        let magic = read_u64(bytes, MAGIC_OFFSET);
        if magic != HERMES_BYTECODE_MAGIC {
            return Err(ParseError::MissingMagic { actual: magic });
        }

        let file_length = read_u32(bytes, FILE_LENGTH_OFFSET);
        let file_length_usize =
            usize::try_from(file_length).expect("u32 always fits in usize on supported targets");
        if file_length_usize > bytes.len() {
            return Err(ParseError::FileLengthExceedsBuffer {
                file_length,
                buffer_len: bytes.len(),
            });
        }

        let mut source_hash = [0; HERMES_SOURCE_HASH_SIZE];
        source_hash.copy_from_slice(
            &bytes[SOURCE_HASH_OFFSET..SOURCE_HASH_OFFSET + HERMES_SOURCE_HASH_SIZE],
        );

        Ok(Self {
            version: read_u32(bytes, VERSION_OFFSET),
            source_hash,
            file_length,
            global_code_index: read_u32(bytes, GLOBAL_CODE_INDEX_OFFSET),
            function_count: read_u32(bytes, FUNCTION_COUNT_OFFSET),
            string_kind_count: read_u32(bytes, STRING_KIND_COUNT_OFFSET),
            identifier_count: read_u32(bytes, IDENTIFIER_COUNT_OFFSET),
            string_count: read_u32(bytes, STRING_COUNT_OFFSET),
            overflow_string_count: read_u32(bytes, OVERFLOW_STRING_COUNT_OFFSET),
            string_storage_size: read_u32(bytes, STRING_STORAGE_SIZE_OFFSET),
            big_int_count: read_u32(bytes, BIG_INT_COUNT_OFFSET),
            big_int_storage_size: read_u32(bytes, BIG_INT_STORAGE_SIZE_OFFSET),
            reg_exp_count: read_u32(bytes, REG_EXP_COUNT_OFFSET),
            reg_exp_storage_size: read_u32(bytes, REG_EXP_STORAGE_SIZE_OFFSET),
            literal_value_buffer_size: read_u32(bytes, LITERAL_VALUE_BUFFER_SIZE_OFFSET),
            obj_key_buffer_size: read_u32(bytes, OBJ_KEY_BUFFER_SIZE_OFFSET),
            obj_shape_table_count: read_u32(bytes, OBJ_SHAPE_TABLE_COUNT_OFFSET),
            num_string_switch_imms: read_u32(bytes, NUM_STRING_SWITCH_IMMS_OFFSET),
            segment_id: read_u32(bytes, SEGMENT_ID_OFFSET),
            cjs_module_count: read_u32(bytes, CJS_MODULE_COUNT_OFFSET),
            function_source_count: read_u32(bytes, FUNCTION_SOURCE_COUNT_OFFSET),
            debug_info_offset: read_u32(bytes, DEBUG_INFO_OFFSET_OFFSET),
            options: BytecodeOptions::from_flags(bytes[OPTIONS_OFFSET]),
        })
    }
}

/// Hermes bytecode option flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytecodeOptions {
    flags: u8,
}

impl BytecodeOptions {
    /// Builds options from the raw Hermes header flag byte.
    #[must_use]
    pub const fn from_flags(flags: u8) -> Self {
        Self { flags }
    }

    /// Returns the raw Hermes header flag byte.
    #[must_use]
    pub const fn flags(self) -> u8 {
        self.flags
    }

    /// Returns whether static builtins are enabled.
    #[must_use]
    pub const fn static_builtins(self) -> bool {
        self.flags & 0b0000_0001 != 0
    }

    /// Returns whether CommonJS modules are statically resolved.
    #[must_use]
    pub const fn cjs_modules_statically_resolved(self) -> bool {
        self.flags & 0b0000_0010 != 0
    }
}

/// Error returned when Hermes bytecode metadata cannot be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// The provided buffer is smaller than a Hermes bytecode header.
    BufferTooSmall {
        /// Actual provided byte count.
        actual: usize,
        /// Minimum required byte count.
        minimum: usize,
    },
    /// The first eight bytes do not match Hermes bytecode magic.
    MissingMagic {
        /// Actual magic value read from the buffer.
        actual: u64,
    },
    /// The header-declared file length exceeds the provided buffer.
    FileLengthExceedsBuffer {
        /// Declared Hermes bytecode file length.
        file_length: u32,
        /// Provided buffer length.
        buffer_len: usize,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooSmall { actual, minimum } => write!(
                formatter,
                "Hermes bytecode header requires at least {minimum} bytes, got {actual}",
            ),
            Self::MissingMagic { actual } => write!(
                formatter,
                "Hermes bytecode magic mismatch: expected {HERMES_BYTECODE_MAGIC:#018x}, got {actual:#018x}",
            ),
            Self::FileLengthExceedsBuffer {
                file_length,
                buffer_len,
            } => write!(
                formatter,
                "Hermes bytecode fileLength {file_length} exceeds provided buffer length {buffer_len}",
            ),
        }
    }
}

impl std::error::Error for ParseError {}

fn parse_hbc_metadata(bytes: &[u8]) -> Result<ffi::HbcMetadata, ParseError> {
    let header = HermesBytecodeHeader::parse(bytes)?;
    Ok(ffi::HbcMetadata {
        version: header.version,
        file_length: header.file_length,
        global_code_index: header.global_code_index,
        function_count: header.function_count,
        string_count: header.string_count,
        cjs_module_count: header.cjs_module_count,
        debug_info_offset: header.debug_info_offset,
        options: header.options.flags(),
    })
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + size_of::<u32>()].try_into().expect(
        "HermesBytecodeHeader::parse validates the full header length before reading fields",
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + size_of::<u64>()].try_into().expect(
        "HermesBytecodeHeader::parse validates the full header length before reading fields",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + size_of::<u32>()].copy_from_slice(&value.to_le_bytes());
    }

    fn fixture_header() -> Vec<u8> {
        let mut bytes = vec![0; HERMES_BYTECODE_HEADER_SIZE];
        bytes[MAGIC_OFFSET..MAGIC_OFFSET + size_of::<u64>()]
            .copy_from_slice(&HERMES_BYTECODE_MAGIC.to_le_bytes());
        write_u32(&mut bytes, VERSION_OFFSET, 98);
        bytes[SOURCE_HASH_OFFSET..SOURCE_HASH_OFFSET + HERMES_SOURCE_HASH_SIZE]
            .copy_from_slice(&[7; HERMES_SOURCE_HASH_SIZE]);
        write_u32(
            &mut bytes,
            FILE_LENGTH_OFFSET,
            HERMES_BYTECODE_HEADER_SIZE as u32,
        );
        write_u32(&mut bytes, GLOBAL_CODE_INDEX_OFFSET, 17);
        write_u32(&mut bytes, FUNCTION_COUNT_OFFSET, 29);
        write_u32(&mut bytes, STRING_KIND_COUNT_OFFSET, 3);
        write_u32(&mut bytes, IDENTIFIER_COUNT_OFFSET, 5);
        write_u32(&mut bytes, STRING_COUNT_OFFSET, 11);
        write_u32(&mut bytes, OVERFLOW_STRING_COUNT_OFFSET, 13);
        write_u32(&mut bytes, STRING_STORAGE_SIZE_OFFSET, 128);
        write_u32(&mut bytes, BIG_INT_COUNT_OFFSET, 2);
        write_u32(&mut bytes, BIG_INT_STORAGE_SIZE_OFFSET, 16);
        write_u32(&mut bytes, REG_EXP_COUNT_OFFSET, 4);
        write_u32(&mut bytes, REG_EXP_STORAGE_SIZE_OFFSET, 32);
        write_u32(&mut bytes, LITERAL_VALUE_BUFFER_SIZE_OFFSET, 64);
        write_u32(&mut bytes, OBJ_KEY_BUFFER_SIZE_OFFSET, 96);
        write_u32(&mut bytes, OBJ_SHAPE_TABLE_COUNT_OFFSET, 6);
        write_u32(&mut bytes, NUM_STRING_SWITCH_IMMS_OFFSET, 8);
        write_u32(&mut bytes, SEGMENT_ID_OFFSET, 1);
        write_u32(&mut bytes, CJS_MODULE_COUNT_OFFSET, 10);
        write_u32(&mut bytes, FUNCTION_SOURCE_COUNT_OFFSET, 12);
        write_u32(&mut bytes, DEBUG_INFO_OFFSET_OFFSET, 120);
        bytes[OPTIONS_OFFSET] = 0b0000_0011;
        bytes
    }

    #[test]
    fn parses_header_without_copying_payload() {
        let bytes = fixture_header();
        let bytecode = HermesBytecode::parse(&bytes).expect("valid Hermes bytecode header");
        let header = bytecode.header();

        assert!(std::ptr::eq(bytecode.bytes().as_ptr(), bytes.as_ptr()));
        assert_eq!(header.version, 98);
        assert_eq!(header.source_hash, [7; HERMES_SOURCE_HASH_SIZE]);
        assert_eq!(header.file_length, HERMES_BYTECODE_HEADER_SIZE as u32);
        assert_eq!(header.global_code_index, 17);
        assert_eq!(header.function_count, 29);
        assert_eq!(header.string_kind_count, 3);
        assert_eq!(header.identifier_count, 5);
        assert_eq!(header.string_count, 11);
        assert_eq!(header.overflow_string_count, 13);
        assert_eq!(header.string_storage_size, 128);
        assert_eq!(header.big_int_count, 2);
        assert_eq!(header.big_int_storage_size, 16);
        assert_eq!(header.reg_exp_count, 4);
        assert_eq!(header.reg_exp_storage_size, 32);
        assert_eq!(header.literal_value_buffer_size, 64);
        assert_eq!(header.obj_key_buffer_size, 96);
        assert_eq!(header.obj_shape_table_count, 6);
        assert_eq!(header.num_string_switch_imms, 8);
        assert_eq!(header.segment_id, 1);
        assert_eq!(header.cjs_module_count, 10);
        assert_eq!(header.function_source_count, 12);
        assert_eq!(header.debug_info_offset, 120);
        assert_eq!(header.options.flags(), 0b0000_0011);
        assert!(header.options.static_builtins());
        assert!(header.options.cjs_modules_statically_resolved());
    }

    #[test]
    fn exposes_metadata_for_cxx_hosts() {
        let bytes = fixture_header();
        let metadata = parse_hbc_metadata(&bytes).expect("valid Hermes bytecode metadata");

        assert_eq!(metadata.version, 98);
        assert_eq!(metadata.file_length, HERMES_BYTECODE_HEADER_SIZE as u32);
        assert_eq!(metadata.global_code_index, 17);
        assert_eq!(metadata.function_count, 29);
        assert_eq!(metadata.string_count, 11);
        assert_eq!(metadata.cjs_module_count, 10);
        assert_eq!(metadata.debug_info_offset, 120);
        assert_eq!(metadata.options, 0b0000_0011);
    }

    #[test]
    fn rejects_short_buffer() {
        let error = HermesBytecodeHeader::parse(&[0; 16]).expect_err("short header must fail");
        assert_eq!(
            error,
            ParseError::BufferTooSmall {
                actual: 16,
                minimum: HERMES_BYTECODE_HEADER_SIZE,
            },
        );
    }

    #[test]
    fn rejects_non_hermes_magic() {
        let mut bytes = fixture_header();
        bytes[MAGIC_OFFSET] = 0;

        let error = HermesBytecodeHeader::parse(&bytes).expect_err("bad magic must fail");
        assert!(matches!(error, ParseError::MissingMagic { .. }));
    }

    #[test]
    fn rejects_file_length_larger_than_buffer() {
        let mut bytes = fixture_header();
        write_u32(
            &mut bytes,
            FILE_LENGTH_OFFSET,
            HERMES_BYTECODE_HEADER_SIZE as u32 + 1,
        );

        let error = HermesBytecodeHeader::parse(&bytes).expect_err("oversized file must fail");
        assert_eq!(
            error,
            ParseError::FileLengthExceedsBuffer {
                file_length: HERMES_BYTECODE_HEADER_SIZE as u32 + 1,
                buffer_len: HERMES_BYTECODE_HEADER_SIZE,
            },
        );
    }
}
