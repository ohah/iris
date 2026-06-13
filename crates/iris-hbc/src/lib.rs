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
        pub function_headers_offset: u32,
        pub function_headers_size: u32,
        pub string_count: u32,
        pub string_kinds_offset: u32,
        pub identifier_hashes_offset: u32,
        pub small_string_table_offset: u32,
        pub overflow_string_table_offset: u32,
        pub string_storage_offset: u32,
        pub string_storage_size: u32,
        pub cjs_module_count: u32,
        pub cjs_module_table_offset: u32,
        pub cjs_module_table_size: u32,
        pub function_source_table_offset: u32,
        pub function_source_table_size: u32,
        pub function_bodies_offset: u32,
        pub global_function_offset: u32,
        pub global_function_size: u32,
        pub global_function_name: u32,
        pub global_function_param_count: u32,
        pub global_function_frame_size: u32,
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

/// Number of bytes in Hermes' `BytecodeFileFooter`.
pub const HERMES_BYTECODE_FOOTER_SIZE: usize = 20;

/// Byte alignment used by Hermes bytecode file sections.
pub const HERMES_BYTECODE_ALIGNMENT: usize = 4;

const HERMES_SOURCE_HASH_SIZE: usize = 20;
const SMALL_FUNC_HEADER_SIZE: usize = 12;
const LARGE_FUNC_HEADER_SIZE: usize = 36;
const STRING_KIND_ENTRY_SIZE: usize = 4;
const IDENTIFIER_HASH_SIZE: usize = 4;
const SMALL_STRING_TABLE_ENTRY_SIZE: usize = 4;
const OVERFLOW_STRING_TABLE_ENTRY_SIZE: usize = 8;
const SHAPE_TABLE_ENTRY_SIZE: usize = 8;
const BIG_INT_TABLE_ENTRY_SIZE: usize = 8;
const REG_EXP_TABLE_ENTRY_SIZE: usize = 8;
const U32_PAIR_ENTRY_SIZE: usize = 8;

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
    sections: HermesBytecodeSections<'a>,
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
        let sections = HermesBytecodeSections::parse(bytes, header)?;
        Ok(Self {
            bytes,
            header,
            sections,
        })
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

    /// Returns the parsed section views.
    #[must_use]
    pub const fn sections(self) -> HermesBytecodeSections<'a> {
        self.sections
    }

    /// Returns one parsed function header by function id.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if `function_id` is out of range or an overflow
    /// header points outside the bytecode file.
    pub fn function_header(self, function_id: u32) -> Result<HermesFunctionHeader, ParseError> {
        function_header_at(
            self.bytes,
            self.header,
            self.sections.function_headers,
            function_id,
        )
    }

    /// Returns the parsed global function header.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if the global function id is invalid or its
    /// header points outside the bytecode file.
    pub fn global_function_header(self) -> Result<HermesFunctionHeader, ParseError> {
        self.function_header(self.header.global_code_index)
    }
}

/// Zero-copy views over structured Hermes bytecode file sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HermesBytecodeSections<'a> {
    function_headers: SectionView<'a>,
    string_kinds: SectionView<'a>,
    identifier_hashes: SectionView<'a>,
    small_string_table: SectionView<'a>,
    overflow_string_table: SectionView<'a>,
    string_storage: SectionView<'a>,
    literal_value_buffer: SectionView<'a>,
    obj_key_buffer: SectionView<'a>,
    obj_shape_table: SectionView<'a>,
    big_int_table: SectionView<'a>,
    big_int_storage: SectionView<'a>,
    reg_exp_table: SectionView<'a>,
    reg_exp_storage: SectionView<'a>,
    cjs_module_table: SectionView<'a>,
    function_source_table: SectionView<'a>,
    function_bodies_offset: u32,
}

impl<'a> HermesBytecodeSections<'a> {
    fn parse(bytes: &'a [u8], header: HermesBytecodeHeader) -> Result<Self, ParseError> {
        validate_header_counts(header)?;

        let file_length = usize::try_from(header.file_length)
            .expect("u32 always fits in usize on supported targets");
        let file_bytes = &bytes[..file_length];
        let mut cursor = HERMES_BYTECODE_HEADER_SIZE;

        let function_headers = take_section(
            file_bytes,
            &mut cursor,
            "function_headers",
            header.function_count,
            SMALL_FUNC_HEADER_SIZE,
        )?;
        let string_kinds = take_section(
            file_bytes,
            &mut cursor,
            "string_kinds",
            header.string_kind_count,
            STRING_KIND_ENTRY_SIZE,
        )?;
        let identifier_hashes = take_section(
            file_bytes,
            &mut cursor,
            "identifier_hashes",
            header.identifier_count,
            IDENTIFIER_HASH_SIZE,
        )?;
        let small_string_table = take_section(
            file_bytes,
            &mut cursor,
            "small_string_table",
            header.string_count,
            SMALL_STRING_TABLE_ENTRY_SIZE,
        )?;
        let overflow_string_table = take_section(
            file_bytes,
            &mut cursor,
            "overflow_string_table",
            header.overflow_string_count,
            OVERFLOW_STRING_TABLE_ENTRY_SIZE,
        )?;
        let string_storage = take_section(
            file_bytes,
            &mut cursor,
            "string_storage",
            header.string_storage_size,
            1,
        )?;
        let literal_value_buffer = take_section(
            file_bytes,
            &mut cursor,
            "literal_value_buffer",
            header.literal_value_buffer_size,
            1,
        )?;
        let obj_key_buffer = take_section(
            file_bytes,
            &mut cursor,
            "obj_key_buffer",
            header.obj_key_buffer_size,
            1,
        )?;
        let obj_shape_table = take_section(
            file_bytes,
            &mut cursor,
            "obj_shape_table",
            header.obj_shape_table_count,
            SHAPE_TABLE_ENTRY_SIZE,
        )?;
        let big_int_table = take_section(
            file_bytes,
            &mut cursor,
            "big_int_table",
            header.big_int_count,
            BIG_INT_TABLE_ENTRY_SIZE,
        )?;
        let big_int_storage = take_section(
            file_bytes,
            &mut cursor,
            "big_int_storage",
            header.big_int_storage_size,
            1,
        )?;
        let reg_exp_table = take_section(
            file_bytes,
            &mut cursor,
            "reg_exp_table",
            header.reg_exp_count,
            REG_EXP_TABLE_ENTRY_SIZE,
        )?;
        let reg_exp_storage = take_section(
            file_bytes,
            &mut cursor,
            "reg_exp_storage",
            header.reg_exp_storage_size,
            1,
        )?;
        let cjs_module_table = take_section(
            file_bytes,
            &mut cursor,
            "cjs_module_table",
            header.cjs_module_count,
            U32_PAIR_ENTRY_SIZE,
        )?;
        let function_source_table = take_section(
            file_bytes,
            &mut cursor,
            "function_source_table",
            header.function_source_count,
            U32_PAIR_ENTRY_SIZE,
        )?;

        let function_bodies_offset =
            u32::try_from(cursor).map_err(|_| ParseError::OffsetExceedsU32 {
                section: "function_bodies",
                offset: cursor,
            })?;

        validate_string_kind_runs(string_kinds.bytes, header.string_count)?;
        validate_string_table(
            small_string_table.bytes,
            overflow_string_table.bytes,
            header.overflow_string_count,
            header.string_storage_size,
        )?;
        validate_function_headers(bytes, header, function_headers, function_bodies_offset)?;
        validate_cjs_module_table(cjs_module_table.bytes, header)?;
        validate_function_source_table(function_source_table.bytes, header)?;

        if cursor
            .checked_add(HERMES_BYTECODE_FOOTER_SIZE)
            .is_none_or(|minimum| minimum > file_length)
        {
            return Err(ParseError::MissingFooter {
                cursor,
                file_length,
                footer_size: HERMES_BYTECODE_FOOTER_SIZE,
            });
        }

        Ok(Self {
            function_headers,
            string_kinds,
            identifier_hashes,
            small_string_table,
            overflow_string_table,
            string_storage,
            literal_value_buffer,
            obj_key_buffer,
            obj_shape_table,
            big_int_table,
            big_int_storage,
            reg_exp_table,
            reg_exp_storage,
            cjs_module_table,
            function_source_table,
            function_bodies_offset,
        })
    }

    /// Returns the raw small function header table.
    #[must_use]
    pub const fn function_headers(self) -> SectionView<'a> {
        self.function_headers
    }

    /// Returns the raw string-kind run-length table.
    #[must_use]
    pub const fn string_kinds(self) -> SectionView<'a> {
        self.string_kinds
    }

    /// Returns the raw identifier hash table.
    #[must_use]
    pub const fn identifier_hashes(self) -> SectionView<'a> {
        self.identifier_hashes
    }

    /// Returns the raw small string table.
    #[must_use]
    pub const fn small_string_table(self) -> SectionView<'a> {
        self.small_string_table
    }

    /// Returns the raw overflow string table.
    #[must_use]
    pub const fn overflow_string_table(self) -> SectionView<'a> {
        self.overflow_string_table
    }

    /// Returns the string content storage.
    #[must_use]
    pub const fn string_storage(self) -> SectionView<'a> {
        self.string_storage
    }

    /// Returns the raw literal value buffer.
    #[must_use]
    pub const fn literal_value_buffer(self) -> SectionView<'a> {
        self.literal_value_buffer
    }

    /// Returns the raw object key buffer.
    #[must_use]
    pub const fn obj_key_buffer(self) -> SectionView<'a> {
        self.obj_key_buffer
    }

    /// Returns the raw object shape table.
    #[must_use]
    pub const fn obj_shape_table(self) -> SectionView<'a> {
        self.obj_shape_table
    }

    /// Returns the raw bigint table.
    #[must_use]
    pub const fn big_int_table(self) -> SectionView<'a> {
        self.big_int_table
    }

    /// Returns the raw bigint byte storage.
    #[must_use]
    pub const fn big_int_storage(self) -> SectionView<'a> {
        self.big_int_storage
    }

    /// Returns the raw regular expression table.
    #[must_use]
    pub const fn reg_exp_table(self) -> SectionView<'a> {
        self.reg_exp_table
    }

    /// Returns the raw regular expression byte storage.
    #[must_use]
    pub const fn reg_exp_storage(self) -> SectionView<'a> {
        self.reg_exp_storage
    }

    /// Returns the raw CommonJS module table.
    #[must_use]
    pub const fn cjs_module_table(self) -> SectionView<'a> {
        self.cjs_module_table
    }

    /// Returns the raw function source table.
    #[must_use]
    pub const fn function_source_table(self) -> SectionView<'a> {
        self.function_source_table
    }

    /// Returns the file offset where serialized function bodies begin.
    #[must_use]
    pub const fn function_bodies_offset(self) -> u32 {
        self.function_bodies_offset
    }
}

/// A byte slice plus its offset inside the Hermes bytecode file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionView<'a> {
    offset: u32,
    bytes: &'a [u8],
}

impl<'a> SectionView<'a> {
    /// Returns this section's byte offset from the start of the file.
    #[must_use]
    pub const fn offset(self) -> u32 {
        self.offset
    }

    /// Returns this section's byte length.
    #[must_use]
    pub fn len(self) -> u32 {
        u32::try_from(self.bytes.len()).expect("validated HBC section length fits in u32")
    }

    /// Returns whether this section has zero bytes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    /// Returns the original section bytes.
    #[must_use]
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }
}

/// Parsed Hermes runtime function header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HermesFunctionHeader {
    /// Offset of this function's bytecode body from the start of the file.
    pub offset: u32,
    /// Bytecode body size in bytes.
    pub bytecode_size_in_bytes: u32,
    /// Number of declared parameters.
    pub param_count: u32,
    /// Function loop nesting depth.
    pub loop_depth: u32,
    /// String table id of the function name.
    pub function_name: u32,
    /// Count of number registers.
    pub number_reg_count: u32,
    /// Count of non-pointer registers.
    pub non_ptr_reg_count: u32,
    /// Stack frame size.
    pub frame_size: u32,
    /// Read cache size.
    pub read_cache_size: u8,
    /// Write cache size.
    pub write_cache_size: u8,
    /// Private name cache size.
    pub private_name_cache_size: u8,
    /// Raw Hermes function header flag byte.
    pub flags: u8,
}

impl HermesFunctionHeader {
    /// Returns whether this function is strict mode.
    #[must_use]
    pub const fn strict_mode(self) -> bool {
        self.flags & 0b0000_0100 != 0
    }

    /// Returns whether this function has an exception handler table.
    #[must_use]
    pub const fn has_exception_handler(self) -> bool {
        self.flags & 0b0000_1000 != 0
    }

    /// Returns whether this function has debug info.
    #[must_use]
    pub const fn has_debug_info(self) -> bool {
        self.flags & 0b0001_0000 != 0
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
    /// Header counts fail basic invariants.
    InvalidHeaderCount {
        /// Field that failed validation.
        field: &'static str,
        /// The invalid field value.
        value: u32,
        /// The related limit.
        limit: u32,
    },
    /// Multiplying a section element count by element size overflowed.
    SectionSizeOverflow {
        /// Section name.
        section: &'static str,
        /// Number of entries in the section.
        count: u32,
        /// Size of each entry in bytes.
        entry_size: usize,
    },
    /// A section would extend beyond the declared bytecode file length.
    SectionOutOfBounds {
        /// Section name.
        section: &'static str,
        /// Section offset in bytes.
        offset: usize,
        /// Section size in bytes.
        size: usize,
        /// Declared bytecode file length.
        file_length: usize,
    },
    /// A section offset cannot be represented in the C++ bridge metadata.
    OffsetExceedsU32 {
        /// Section name.
        section: &'static str,
        /// Section offset in bytes.
        offset: usize,
    },
    /// Section layout ended before Hermes' required file footer.
    MissingFooter {
        /// Offset after the last parsed structured section.
        cursor: usize,
        /// Declared bytecode file length.
        file_length: usize,
        /// Required footer size in bytes.
        footer_size: usize,
    },
    /// String-kind run-length encoding does not cover exactly all strings.
    InvalidStringKindRunLength {
        /// Declared string count.
        string_count: u32,
        /// Count produced by summing the string-kind runs.
        run_length: u32,
    },
    /// A small string table entry points past the overflow string table.
    InvalidOverflowStringIndex {
        /// Index of the string table entry.
        string_index: u32,
        /// Referenced overflow table index.
        overflow_index: u32,
        /// Declared overflow table entry count.
        overflow_count: u32,
    },
    /// A string table entry points outside string storage.
    StringTableEntryOutOfBounds {
        /// Index of the string table entry.
        string_index: u32,
        /// String byte offset in string storage.
        offset: u32,
        /// String byte length.
        length: u32,
        /// Declared string storage byte length.
        storage_size: u32,
    },
    /// A requested function id is outside the function table.
    InvalidFunctionIndex {
        /// Function id being read.
        function_id: u32,
        /// Declared function count.
        function_count: u32,
    },
    /// A small function header's overflow pointer is outside the bytecode.
    LargeFunctionHeaderOutOfBounds {
        /// Function id whose header overflowed.
        function_id: u32,
        /// File offset of the large function header.
        offset: u32,
        /// Declared bytecode file length.
        file_length: u32,
    },
    /// A function header points to bytecode outside the executable function region.
    FunctionBodyOutOfBounds {
        /// Function id whose body is invalid.
        function_id: u32,
        /// Function bytecode body offset.
        offset: u32,
        /// Function bytecode body size.
        size: u32,
        /// First valid function body offset.
        function_bodies_offset: u32,
        /// Last accepted function body byte offset.
        limit: u32,
    },
    /// A CommonJS module table entry references an invalid function or string id.
    InvalidCjsModuleEntry {
        /// Table entry index.
        entry_index: u32,
        /// First raw value in the pair.
        first: u32,
        /// Second raw value in the pair.
        second: u32,
        /// Whether this is the statically resolved CJS module table.
        statically_resolved: bool,
    },
    /// A function-source table entry references an invalid function or string id.
    InvalidFunctionSourceEntry {
        /// Table entry index.
        entry_index: u32,
        /// Referenced function id.
        function_id: u32,
        /// Referenced string id.
        string_id: u32,
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
            Self::InvalidHeaderCount {
                field,
                value,
                limit,
            } => write!(
                formatter,
                "Hermes bytecode header field {field} has value {value}, exceeding limit {limit}",
            ),
            Self::SectionSizeOverflow {
                section,
                count,
                entry_size,
            } => write!(
                formatter,
                "Hermes bytecode section {section} size overflow for {count} entries of {entry_size} bytes",
            ),
            Self::SectionOutOfBounds {
                section,
                offset,
                size,
                file_length,
            } => write!(
                formatter,
                "Hermes bytecode section {section} at {offset} with size {size} exceeds fileLength {file_length}",
            ),
            Self::OffsetExceedsU32 { section, offset } => write!(
                formatter,
                "Hermes bytecode section {section} offset {offset} cannot fit in u32 metadata",
            ),
            Self::MissingFooter {
                cursor,
                file_length,
                footer_size,
            } => write!(
                formatter,
                "Hermes bytecode sections end at {cursor}, leaving less than {footer_size} bytes for footer before fileLength {file_length}",
            ),
            Self::InvalidStringKindRunLength {
                string_count,
                run_length,
            } => write!(
                formatter,
                "Hermes bytecode string-kind runs cover {run_length} strings, expected {string_count}",
            ),
            Self::InvalidOverflowStringIndex {
                string_index,
                overflow_index,
                overflow_count,
            } => write!(
                formatter,
                "Hermes bytecode string entry {string_index} references overflow index {overflow_index}, but overflow count is {overflow_count}",
            ),
            Self::StringTableEntryOutOfBounds {
                string_index,
                offset,
                length,
                storage_size,
            } => write!(
                formatter,
                "Hermes bytecode string entry {string_index} at offset {offset} with length {length} exceeds string storage size {storage_size}",
            ),
            Self::InvalidFunctionIndex {
                function_id,
                function_count,
            } => write!(
                formatter,
                "Hermes bytecode function id {function_id} exceeds function count {function_count}",
            ),
            Self::LargeFunctionHeaderOutOfBounds {
                function_id,
                offset,
                file_length,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} large header at {offset} exceeds fileLength {file_length}",
            ),
            Self::FunctionBodyOutOfBounds {
                function_id,
                offset,
                size,
                function_bodies_offset,
                limit,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} body at {offset} with size {size} is outside function body region {function_bodies_offset}..{limit}",
            ),
            Self::InvalidCjsModuleEntry {
                entry_index,
                first,
                second,
                statically_resolved,
            } => write!(
                formatter,
                "Hermes bytecode CJS module entry {entry_index} ({first}, {second}) is invalid; staticallyResolved={statically_resolved}",
            ),
            Self::InvalidFunctionSourceEntry {
                entry_index,
                function_id,
                string_id,
            } => write!(
                formatter,
                "Hermes bytecode function source entry {entry_index} references function {function_id} and string {string_id}",
            ),
        }
    }
}

impl std::error::Error for ParseError {}

fn parse_hbc_metadata(bytes: &[u8]) -> Result<ffi::HbcMetadata, ParseError> {
    let bytecode = HermesBytecode::parse(bytes)?;
    let header = bytecode.header();
    let sections = bytecode.sections();
    let global_function = bytecode.global_function_header()?;
    Ok(ffi::HbcMetadata {
        version: header.version,
        file_length: header.file_length,
        global_code_index: header.global_code_index,
        function_count: header.function_count,
        function_headers_offset: sections.function_headers().offset(),
        function_headers_size: sections.function_headers().len(),
        string_count: header.string_count,
        string_kinds_offset: sections.string_kinds().offset(),
        identifier_hashes_offset: sections.identifier_hashes().offset(),
        small_string_table_offset: sections.small_string_table().offset(),
        overflow_string_table_offset: sections.overflow_string_table().offset(),
        string_storage_offset: sections.string_storage().offset(),
        string_storage_size: sections.string_storage().len(),
        cjs_module_count: header.cjs_module_count,
        cjs_module_table_offset: sections.cjs_module_table().offset(),
        cjs_module_table_size: sections.cjs_module_table().len(),
        function_source_table_offset: sections.function_source_table().offset(),
        function_source_table_size: sections.function_source_table().len(),
        function_bodies_offset: sections.function_bodies_offset(),
        global_function_offset: global_function.offset,
        global_function_size: global_function.bytecode_size_in_bytes,
        global_function_name: global_function.function_name,
        global_function_param_count: global_function.param_count,
        global_function_frame_size: global_function.frame_size,
        debug_info_offset: header.debug_info_offset,
        options: header.options.flags(),
    })
}

fn validate_header_counts(header: HermesBytecodeHeader) -> Result<(), ParseError> {
    if header.identifier_count > header.string_count {
        return Err(ParseError::InvalidHeaderCount {
            field: "identifier_count",
            value: header.identifier_count,
            limit: header.string_count,
        });
    }
    if header.function_count == 0 {
        if header.global_code_index != 0 {
            return Err(ParseError::InvalidHeaderCount {
                field: "global_code_index",
                value: header.global_code_index,
                limit: 0,
            });
        }
    } else if header.global_code_index >= header.function_count {
        return Err(ParseError::InvalidHeaderCount {
            field: "global_code_index",
            value: header.global_code_index,
            limit: header.function_count - 1,
        });
    }
    Ok(())
}

fn take_section<'a>(
    file_bytes: &'a [u8],
    cursor: &mut usize,
    section: &'static str,
    count: u32,
    entry_size: usize,
) -> Result<SectionView<'a>, ParseError> {
    *cursor = align_offset(*cursor, section)?;
    let size = section_size(section, count, entry_size)?;
    let offset = *cursor;
    let end = offset
        .checked_add(size)
        .ok_or(ParseError::SectionOutOfBounds {
            section,
            offset,
            size,
            file_length: file_bytes.len(),
        })?;

    let bytes = file_bytes
        .get(offset..end)
        .ok_or(ParseError::SectionOutOfBounds {
            section,
            offset,
            size,
            file_length: file_bytes.len(),
        })?;
    *cursor = end;

    let offset_u32 =
        u32::try_from(offset).map_err(|_| ParseError::OffsetExceedsU32 { section, offset })?;
    Ok(SectionView {
        offset: offset_u32,
        bytes,
    })
}

fn align_offset(offset: usize, section: &'static str) -> Result<usize, ParseError> {
    offset
        .checked_add(HERMES_BYTECODE_ALIGNMENT - 1)
        .map(|adjusted| adjusted / HERMES_BYTECODE_ALIGNMENT * HERMES_BYTECODE_ALIGNMENT)
        .ok_or(ParseError::OffsetExceedsU32 { section, offset })
}

fn section_size(section: &'static str, count: u32, entry_size: usize) -> Result<usize, ParseError> {
    usize::try_from(count)
        .expect("u32 always fits in usize on supported targets")
        .checked_mul(entry_size)
        .ok_or(ParseError::SectionSizeOverflow {
            section,
            count,
            entry_size,
        })
}

fn validate_string_kind_runs(bytes: &[u8], string_count: u32) -> Result<(), ParseError> {
    let mut run_length = 0_u32;
    for entry in bytes.chunks_exact(STRING_KIND_ENTRY_SIZE) {
        let datum = read_u32(entry, 0);
        let count = datum & 0x7fff_ffff;
        run_length =
            run_length
                .checked_add(count)
                .ok_or(ParseError::InvalidStringKindRunLength {
                    string_count,
                    run_length: u32::MAX,
                })?;
    }
    if run_length != string_count {
        return Err(ParseError::InvalidStringKindRunLength {
            string_count,
            run_length,
        });
    }
    Ok(())
}

fn validate_string_table(
    small_entries: &[u8],
    overflow_entries: &[u8],
    overflow_count: u32,
    storage_size: u32,
) -> Result<(), ParseError> {
    for (index, entry) in small_entries
        .chunks_exact(SMALL_STRING_TABLE_ENTRY_SIZE)
        .enumerate()
    {
        let small = read_u32(entry, 0);
        let is_overflowed = small >> 24 == 0xff;
        let (offset, length) = if is_overflowed {
            let overflow_index = (small >> 1) & 0x7f_ffff;
            if overflow_index >= overflow_count {
                return Err(ParseError::InvalidOverflowStringIndex {
                    string_index: u32::try_from(index)
                        .expect("validated string table index fits in u32"),
                    overflow_index,
                    overflow_count,
                });
            }
            let overflow_offset = usize::try_from(overflow_index)
                .expect("u32 always fits in usize on supported targets")
                * OVERFLOW_STRING_TABLE_ENTRY_SIZE;
            let overflow = &overflow_entries
                [overflow_offset..overflow_offset + OVERFLOW_STRING_TABLE_ENTRY_SIZE];
            (read_u32(overflow, 0), read_u32(overflow, 4))
        } else {
            ((small >> 1) & 0x7f_ffff, small >> 24)
        };

        if offset
            .checked_add(length)
            .is_none_or(|end| end > storage_size)
        {
            return Err(ParseError::StringTableEntryOutOfBounds {
                string_index: u32::try_from(index)
                    .expect("validated string table index fits in u32"),
                offset,
                length,
                storage_size,
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DecodedSmallFunctionHeader {
    header: HermesFunctionHeader,
    overflowed: bool,
}

fn function_header_at(
    bytes: &[u8],
    header: HermesBytecodeHeader,
    function_headers: SectionView<'_>,
    function_id: u32,
) -> Result<HermesFunctionHeader, ParseError> {
    if function_id >= header.function_count {
        return Err(ParseError::InvalidFunctionIndex {
            function_id,
            function_count: header.function_count,
        });
    }

    let function_index =
        usize::try_from(function_id).expect("u32 always fits in usize on supported targets");
    let offset = function_index * SMALL_FUNC_HEADER_SIZE;
    let small_header = &function_headers.bytes[offset..offset + SMALL_FUNC_HEADER_SIZE];
    let decoded = parse_small_function_header(small_header);
    if !decoded.overflowed {
        return Ok(decoded.header);
    }

    let large_header_offset = large_header_offset_from_small(decoded.header);
    let large_header_start = usize::try_from(large_header_offset)
        .expect("u32 always fits in usize on supported targets");
    let file_length =
        usize::try_from(header.file_length).expect("u32 always fits in usize on supported targets");
    let large_header_end = large_header_start
        .checked_add(LARGE_FUNC_HEADER_SIZE)
        .ok_or(ParseError::LargeFunctionHeaderOutOfBounds {
            function_id,
            offset: large_header_offset,
            file_length: header.file_length,
        })?;
    if large_header_end > file_length {
        return Err(ParseError::LargeFunctionHeaderOutOfBounds {
            function_id,
            offset: large_header_offset,
            file_length: header.file_length,
        });
    }

    Ok(parse_large_function_header(
        &bytes[large_header_start..large_header_end],
    ))
}

fn parse_small_function_header(bytes: &[u8]) -> DecodedSmallFunctionHeader {
    let first_word = read_u32(bytes, 0);
    let second_word = read_u32(bytes, 4);
    let frame_size = bytes[8];
    let read_cache_size = bytes[9];
    let cache_sizes = bytes[10];
    let flags = bytes[11];

    DecodedSmallFunctionHeader {
        header: HermesFunctionHeader {
            offset: first_word & 0x01ff_ffff,
            param_count: (first_word >> 25) & 0x1f,
            loop_depth: (first_word >> 30) & 0x03,
            bytecode_size_in_bytes: second_word & 0x3fff,
            function_name: (second_word >> 14) & 0xff,
            number_reg_count: (second_word >> 22) & 0x1f,
            non_ptr_reg_count: (second_word >> 27) & 0x1f,
            frame_size: u32::from(frame_size),
            read_cache_size,
            write_cache_size: cache_sizes & 0x7f,
            private_name_cache_size: cache_sizes >> 7,
            flags,
        },
        overflowed: flags & 0b0010_0000 != 0,
    }
}

fn parse_large_function_header(bytes: &[u8]) -> HermesFunctionHeader {
    HermesFunctionHeader {
        offset: read_u32(bytes, 0),
        param_count: read_u32(bytes, 4),
        loop_depth: read_u32(bytes, 8),
        bytecode_size_in_bytes: read_u32(bytes, 12),
        function_name: read_u32(bytes, 16),
        number_reg_count: read_u32(bytes, 20),
        non_ptr_reg_count: read_u32(bytes, 24),
        frame_size: read_u32(bytes, 28),
        read_cache_size: bytes[32],
        write_cache_size: bytes[33],
        private_name_cache_size: bytes[34],
        flags: bytes[35],
    }
}

fn large_header_offset_from_small(header: HermesFunctionHeader) -> u32 {
    ((header.function_name & 0xff) << 24) | (header.offset & 0x00ff_ffff)
}

fn validate_function_headers(
    bytes: &[u8],
    header: HermesBytecodeHeader,
    function_headers: SectionView<'_>,
    function_bodies_offset: u32,
) -> Result<(), ParseError> {
    for function_id in 0..header.function_count {
        let function_header = function_header_at(bytes, header, function_headers, function_id)?;
        validate_function_body(function_id, function_header, header, function_bodies_offset)?;
    }
    Ok(())
}

fn validate_function_body(
    function_id: u32,
    function_header: HermesFunctionHeader,
    header: HermesBytecodeHeader,
    function_bodies_offset: u32,
) -> Result<(), ParseError> {
    let limit = function_body_limit(header);
    let body_end = function_header
        .offset
        .checked_add(function_header.bytecode_size_in_bytes);
    if function_header.offset < function_bodies_offset || body_end.is_none_or(|end| end > limit) {
        return Err(ParseError::FunctionBodyOutOfBounds {
            function_id,
            offset: function_header.offset,
            size: function_header.bytecode_size_in_bytes,
            function_bodies_offset,
            limit,
        });
    }
    Ok(())
}

fn function_body_limit(header: HermesBytecodeHeader) -> u32 {
    if header.debug_info_offset == 0 {
        header
            .file_length
            .saturating_sub(HERMES_BYTECODE_FOOTER_SIZE as u32)
    } else {
        header.debug_info_offset
    }
}

fn validate_cjs_module_table(bytes: &[u8], header: HermesBytecodeHeader) -> Result<(), ParseError> {
    for (index, entry) in bytes.chunks_exact(U32_PAIR_ENTRY_SIZE).enumerate() {
        let first = read_u32(entry, 0);
        let second = read_u32(entry, 4);
        let statically_resolved = header.options.cjs_modules_statically_resolved();
        let invalid = if statically_resolved {
            second >= header.function_count
        } else {
            first >= header.string_count || second >= header.function_count
        };
        if invalid {
            return Err(ParseError::InvalidCjsModuleEntry {
                entry_index: u32::try_from(index)
                    .expect("validated CJS module table index fits in u32"),
                first,
                second,
                statically_resolved,
            });
        }
    }
    Ok(())
}

fn validate_function_source_table(
    bytes: &[u8],
    header: HermesBytecodeHeader,
) -> Result<(), ParseError> {
    for (index, entry) in bytes.chunks_exact(U32_PAIR_ENTRY_SIZE).enumerate() {
        let function_id = read_u32(entry, 0);
        let string_id = read_u32(entry, 4);
        if function_id >= header.function_count || string_id >= header.string_count {
            return Err(ParseError::InvalidFunctionSourceEntry {
                entry_index: u32::try_from(index)
                    .expect("validated function source table index fits in u32"),
                function_id,
                string_id,
            });
        }
    }
    Ok(())
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

    fn append_aligned(bytes: &mut Vec<u8>) {
        while !bytes.len().is_multiple_of(HERMES_BYTECODE_ALIGNMENT) {
            bytes.push(0);
        }
    }

    fn append_bytes(bytes: &mut Vec<u8>, section: &[u8]) {
        append_aligned(bytes);
        bytes.extend_from_slice(section);
    }

    fn append_repeated(bytes: &mut Vec<u8>, len: usize, value: u8) {
        append_aligned(bytes);
        bytes.resize(bytes.len() + len, value);
    }

    fn small_string_entry(offset: u32, length: u32, is_utf16: bool) -> u32 {
        u32::from(is_utf16) | (offset << 1) | (length << 24)
    }

    #[derive(Debug, Clone, Copy)]
    struct SmallFunctionHeaderFixture {
        offset: u32,
        bytecode_size_in_bytes: u32,
        param_count: u32,
        loop_depth: u32,
        function_name: u32,
        number_reg_count: u32,
        non_ptr_reg_count: u32,
        frame_size: u8,
        read_cache_size: u8,
        write_cache_size: u8,
        private_name_cache_size: u8,
        flags: u8,
    }

    fn encode_small_function_header(
        header: SmallFunctionHeaderFixture,
    ) -> [u8; SMALL_FUNC_HEADER_SIZE] {
        let mut bytes = [0; SMALL_FUNC_HEADER_SIZE];
        let first_word = (header.offset & 0x01ff_ffff)
            | ((header.param_count & 0x1f) << 25)
            | ((header.loop_depth & 0x03) << 30);
        let second_word = (header.bytecode_size_in_bytes & 0x3fff)
            | ((header.function_name & 0xff) << 14)
            | ((header.number_reg_count & 0x1f) << 22)
            | ((header.non_ptr_reg_count & 0x1f) << 27);
        bytes[0..4].copy_from_slice(&first_word.to_le_bytes());
        bytes[4..8].copy_from_slice(&second_word.to_le_bytes());
        bytes[8] = header.frame_size;
        bytes[9] = header.read_cache_size;
        bytes[10] =
            (header.write_cache_size & 0x7f) | ((header.private_name_cache_size & 0x01) << 7);
        bytes[11] = header.flags;
        bytes
    }

    fn encode_overflow_function_header(large_header_offset: u32) -> [u8; SMALL_FUNC_HEADER_SIZE] {
        encode_small_function_header(SmallFunctionHeaderFixture {
            offset: large_header_offset & 0x00ff_ffff,
            bytecode_size_in_bytes: 0,
            param_count: 0,
            loop_depth: 0,
            function_name: (large_header_offset >> 24) & 0xff,
            number_reg_count: 0,
            non_ptr_reg_count: 0,
            frame_size: 0,
            read_cache_size: 0,
            write_cache_size: 0,
            private_name_cache_size: 0,
            flags: 0b0010_0000,
        })
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

    fn fixture_bytecode() -> Vec<u8> {
        let mut bytes = vec![0; HERMES_BYTECODE_HEADER_SIZE];
        bytes[MAGIC_OFFSET..MAGIC_OFFSET + size_of::<u64>()]
            .copy_from_slice(&HERMES_BYTECODE_MAGIC.to_le_bytes());
        write_u32(&mut bytes, VERSION_OFFSET, 98);
        bytes[SOURCE_HASH_OFFSET..SOURCE_HASH_OFFSET + HERMES_SOURCE_HASH_SIZE]
            .copy_from_slice(&[7; HERMES_SOURCE_HASH_SIZE]);
        write_u32(&mut bytes, GLOBAL_CODE_INDEX_OFFSET, 1);
        write_u32(&mut bytes, FUNCTION_COUNT_OFFSET, 2);
        write_u32(&mut bytes, STRING_KIND_COUNT_OFFSET, 2);
        write_u32(&mut bytes, IDENTIFIER_COUNT_OFFSET, 1);
        write_u32(&mut bytes, STRING_COUNT_OFFSET, 2);
        write_u32(&mut bytes, OVERFLOW_STRING_COUNT_OFFSET, 1);
        write_u32(&mut bytes, STRING_STORAGE_SIZE_OFFSET, 6);
        write_u32(&mut bytes, BIG_INT_COUNT_OFFSET, 1);
        write_u32(&mut bytes, BIG_INT_STORAGE_SIZE_OFFSET, 4);
        write_u32(&mut bytes, REG_EXP_COUNT_OFFSET, 1);
        write_u32(&mut bytes, REG_EXP_STORAGE_SIZE_OFFSET, 2);
        write_u32(&mut bytes, LITERAL_VALUE_BUFFER_SIZE_OFFSET, 3);
        write_u32(&mut bytes, OBJ_KEY_BUFFER_SIZE_OFFSET, 5);
        write_u32(&mut bytes, OBJ_SHAPE_TABLE_COUNT_OFFSET, 1);
        write_u32(&mut bytes, CJS_MODULE_COUNT_OFFSET, 2);
        write_u32(&mut bytes, FUNCTION_SOURCE_COUNT_OFFSET, 1);

        append_aligned(&mut bytes);
        let function_headers_offset = bytes.len();
        bytes.resize(bytes.len() + SMALL_FUNC_HEADER_SIZE * 2, 0);
        append_bytes(
            &mut bytes,
            &[1_u32.to_le_bytes(), 0x8000_0001_u32.to_le_bytes()].concat(),
        );
        append_bytes(&mut bytes, &0x1234_5678_u32.to_le_bytes());
        append_bytes(
            &mut bytes,
            &[
                small_string_entry(0, 2, false).to_le_bytes(),
                small_string_entry(0, 0xff, false).to_le_bytes(),
            ]
            .concat(),
        );
        append_bytes(
            &mut bytes,
            &[2_u32.to_le_bytes(), 4_u32.to_le_bytes()].concat(),
        );
        append_bytes(&mut bytes, b"abcdef");
        append_bytes(&mut bytes, &[1, 2, 3]);
        append_bytes(&mut bytes, &[4, 5, 6, 7, 8]);
        append_repeated(&mut bytes, SHAPE_TABLE_ENTRY_SIZE, 0x22);
        append_repeated(&mut bytes, BIG_INT_TABLE_ENTRY_SIZE, 0x33);
        append_bytes(&mut bytes, &[9, 10, 11, 12]);
        append_repeated(&mut bytes, REG_EXP_TABLE_ENTRY_SIZE, 0x44);
        append_bytes(&mut bytes, &[13, 14]);
        append_bytes(
            &mut bytes,
            &[
                0_u32.to_le_bytes(),
                0_u32.to_le_bytes(),
                1_u32.to_le_bytes(),
                1_u32.to_le_bytes(),
            ]
            .concat(),
        );
        append_bytes(
            &mut bytes,
            &[1_u32.to_le_bytes(), 1_u32.to_le_bytes()].concat(),
        );

        let function_bodies_offset = bytes.len();
        bytes.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd]);
        bytes[function_headers_offset..function_headers_offset + SMALL_FUNC_HEADER_SIZE]
            .copy_from_slice(&encode_small_function_header(SmallFunctionHeaderFixture {
                offset: u32::try_from(function_bodies_offset)
                    .expect("test fixture offset fits in u32"),
                bytecode_size_in_bytes: 2,
                param_count: 1,
                loop_depth: 0,
                function_name: 0,
                number_reg_count: 2,
                non_ptr_reg_count: 1,
                frame_size: 4,
                read_cache_size: 1,
                write_cache_size: 2,
                private_name_cache_size: 1,
                flags: 0b0000_0100,
            }));
        bytes[function_headers_offset + SMALL_FUNC_HEADER_SIZE
            ..function_headers_offset + SMALL_FUNC_HEADER_SIZE * 2]
            .copy_from_slice(&encode_small_function_header(SmallFunctionHeaderFixture {
                offset: u32::try_from(function_bodies_offset + 2)
                    .expect("test fixture offset fits in u32"),
                bytecode_size_in_bytes: 2,
                param_count: 2,
                loop_depth: 1,
                function_name: 1,
                number_reg_count: 3,
                non_ptr_reg_count: 2,
                frame_size: 5,
                read_cache_size: 3,
                write_cache_size: 4,
                private_name_cache_size: 0,
                flags: 0b0000_0100,
            }));

        append_aligned(&mut bytes);
        let debug_info_offset =
            u32::try_from(bytes.len()).expect("test fixture offset fits in u32");
        append_repeated(&mut bytes, 16, 0);
        bytes.extend_from_slice(&[0; HERMES_BYTECODE_FOOTER_SIZE]);

        let file_length = u32::try_from(bytes.len()).expect("test fixture length fits in u32");
        write_u32(&mut bytes, FILE_LENGTH_OFFSET, file_length);
        write_u32(&mut bytes, DEBUG_INFO_OFFSET_OFFSET, debug_info_offset);

        assert_eq!(function_bodies_offset % HERMES_BYTECODE_ALIGNMENT, 0);
        bytes
    }

    #[test]
    fn parses_header_without_copying_payload() {
        let bytes = fixture_bytecode();
        let bytecode = HermesBytecode::parse(&bytes).expect("valid Hermes bytecode header");
        let header = bytecode.header();

        assert!(std::ptr::eq(bytecode.bytes().as_ptr(), bytes.as_ptr()));
        assert_eq!(header.version, 98);
        assert_eq!(header.source_hash, [7; HERMES_SOURCE_HASH_SIZE]);
        assert_eq!(header.file_length, bytes.len() as u32);
        assert_eq!(header.global_code_index, 1);
        assert_eq!(header.function_count, 2);
        assert_eq!(header.string_kind_count, 2);
        assert_eq!(header.identifier_count, 1);
        assert_eq!(header.string_count, 2);
        assert_eq!(header.overflow_string_count, 1);
        assert_eq!(header.string_storage_size, 6);
        assert_eq!(header.big_int_count, 1);
        assert_eq!(header.big_int_storage_size, 4);
        assert_eq!(header.reg_exp_count, 1);
        assert_eq!(header.reg_exp_storage_size, 2);
        assert_eq!(header.literal_value_buffer_size, 3);
        assert_eq!(header.obj_key_buffer_size, 5);
        assert_eq!(header.obj_shape_table_count, 1);
        assert_eq!(header.cjs_module_count, 2);
        assert_eq!(header.function_source_count, 1);
        assert_eq!(header.options.flags(), 0);
    }

    #[test]
    fn exposes_metadata_for_cxx_hosts() {
        let bytes = fixture_bytecode();
        let metadata = parse_hbc_metadata(&bytes).expect("valid Hermes bytecode metadata");

        assert_eq!(metadata.version, 98);
        assert_eq!(metadata.file_length, bytes.len() as u32);
        assert_eq!(metadata.global_code_index, 1);
        assert_eq!(metadata.function_count, 2);
        assert_eq!(metadata.function_headers_offset, 128);
        assert_eq!(metadata.function_headers_size, 24);
        assert_eq!(metadata.string_count, 2);
        assert_eq!(metadata.string_storage_size, 6);
        assert_eq!(metadata.cjs_module_count, 2);
        assert_eq!(metadata.cjs_module_table_size, 16);
        assert_eq!(metadata.function_source_table_size, 8);
        assert!(metadata.function_bodies_offset > metadata.function_source_table_offset);
        assert_eq!(
            metadata.global_function_offset,
            metadata.function_bodies_offset + 2
        );
        assert_eq!(metadata.global_function_size, 2);
        assert_eq!(metadata.global_function_name, 1);
        assert_eq!(metadata.global_function_param_count, 2);
        assert_eq!(metadata.global_function_frame_size, 5);
        assert_eq!(metadata.options, 0);
    }

    #[test]
    fn parses_small_function_headers() {
        let bytes = fixture_bytecode();
        let bytecode = HermesBytecode::parse(&bytes).expect("valid Hermes bytecode");
        let sections = bytecode.sections();

        let first = bytecode
            .function_header(0)
            .expect("valid first function header");
        assert_eq!(first.offset, sections.function_bodies_offset());
        assert_eq!(first.bytecode_size_in_bytes, 2);
        assert_eq!(first.param_count, 1);
        assert_eq!(first.loop_depth, 0);
        assert_eq!(first.function_name, 0);
        assert_eq!(first.number_reg_count, 2);
        assert_eq!(first.non_ptr_reg_count, 1);
        assert_eq!(first.frame_size, 4);
        assert_eq!(first.read_cache_size, 1);
        assert_eq!(first.write_cache_size, 2);
        assert_eq!(first.private_name_cache_size, 1);
        assert!(first.strict_mode());

        let second = bytecode
            .global_function_header()
            .expect("valid global function header");
        assert_eq!(second.offset, sections.function_bodies_offset() + 2);
        assert_eq!(second.bytecode_size_in_bytes, 2);
        assert_eq!(second.param_count, 2);
        assert_eq!(second.loop_depth, 1);
        assert_eq!(second.function_name, 1);
        assert_eq!(second.number_reg_count, 3);
        assert_eq!(second.non_ptr_reg_count, 2);
        assert_eq!(second.frame_size, 5);
        assert_eq!(second.read_cache_size, 3);
        assert_eq!(second.write_cache_size, 4);
        assert_eq!(second.private_name_cache_size, 0);
        assert!(second.strict_mode());
    }

    #[test]
    fn exposes_zero_copy_section_views() {
        let bytes = fixture_bytecode();
        let bytecode = HermesBytecode::parse(&bytes).expect("valid Hermes bytecode");
        let sections = bytecode.sections();

        assert_eq!(sections.function_headers().offset(), 128);
        assert_eq!(sections.function_headers().len(), 24);
        assert_eq!(sections.string_kinds().len(), 8);
        assert_eq!(sections.identifier_hashes().len(), 4);
        assert_eq!(sections.small_string_table().len(), 8);
        assert_eq!(sections.overflow_string_table().len(), 8);
        assert_eq!(sections.string_storage().bytes(), b"abcdef");
        assert_eq!(sections.literal_value_buffer().bytes(), &[1, 2, 3]);
        assert_eq!(sections.obj_key_buffer().bytes(), &[4, 5, 6, 7, 8]);
        assert_eq!(sections.obj_shape_table().len(), 8);
        assert_eq!(sections.big_int_table().len(), 8);
        assert_eq!(sections.big_int_storage().bytes(), &[9, 10, 11, 12]);
        assert_eq!(sections.reg_exp_table().len(), 8);
        assert_eq!(sections.reg_exp_storage().bytes(), &[13, 14]);
        assert_eq!(sections.cjs_module_table().len(), 16);
        assert_eq!(sections.function_source_table().len(), 8);
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

    #[test]
    fn rejects_section_past_file_length() {
        let mut bytes = fixture_bytecode();
        write_u32(&mut bytes, FILE_LENGTH_OFFSET, 144);

        let error = HermesBytecode::parse(&bytes).expect_err("truncated section layout must fail");
        assert!(matches!(error, ParseError::SectionOutOfBounds { .. }));
    }

    #[test]
    fn rejects_string_kind_runs_that_do_not_cover_all_strings() {
        let mut bytes = fixture_bytecode();
        let string_kinds_offset = HermesBytecode::parse(&bytes)
            .expect("valid fixture")
            .sections()
            .string_kinds()
            .offset() as usize;
        write_u32(&mut bytes, string_kinds_offset, 2);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid string runs must fail");
        assert_eq!(
            error,
            ParseError::InvalidStringKindRunLength {
                string_count: 2,
                run_length: 3,
            },
        );
    }

    #[test]
    fn rejects_string_entries_outside_storage() {
        let mut bytes = fixture_bytecode();
        let small_string_offset = HermesBytecode::parse(&bytes)
            .expect("valid fixture")
            .sections()
            .small_string_table()
            .offset() as usize;
        write_u32(
            &mut bytes,
            small_string_offset,
            small_string_entry(5, 2, false),
        );

        let error = HermesBytecode::parse(&bytes).expect_err("invalid string entry must fail");
        assert_eq!(
            error,
            ParseError::StringTableEntryOutOfBounds {
                string_index: 0,
                offset: 5,
                length: 2,
                storage_size: 6,
            },
        );
    }

    #[test]
    fn rejects_function_body_outside_function_region() {
        let mut bytes = fixture_bytecode();
        let (function_headers_offset, function_bodies_offset, debug_info_offset) = {
            let bytecode = HermesBytecode::parse(&bytes).expect("valid fixture");
            (
                bytecode.sections().function_headers().offset() as usize,
                bytecode.sections().function_bodies_offset(),
                bytecode.header().debug_info_offset,
            )
        };
        bytes[function_headers_offset..function_headers_offset + SMALL_FUNC_HEADER_SIZE]
            .copy_from_slice(&encode_small_function_header(SmallFunctionHeaderFixture {
                offset: function_bodies_offset - 1,
                bytecode_size_in_bytes: 2,
                param_count: 1,
                loop_depth: 0,
                function_name: 0,
                number_reg_count: 2,
                non_ptr_reg_count: 1,
                frame_size: 4,
                read_cache_size: 1,
                write_cache_size: 2,
                private_name_cache_size: 1,
                flags: 0b0000_0100,
            }));

        let error = HermesBytecode::parse(&bytes).expect_err("invalid function body must fail");
        assert_eq!(
            error,
            ParseError::FunctionBodyOutOfBounds {
                function_id: 0,
                offset: function_bodies_offset - 1,
                size: 2,
                function_bodies_offset,
                limit: debug_info_offset,
            },
        );
    }

    #[test]
    fn rejects_large_function_header_outside_file() {
        let mut bytes = fixture_bytecode();
        let (function_headers_offset, invalid_large_header_offset, file_length) = {
            let bytecode = HermesBytecode::parse(&bytes).expect("valid fixture");
            (
                bytecode.sections().function_headers().offset() as usize,
                bytecode.header().file_length - 1,
                bytecode.header().file_length,
            )
        };
        bytes[function_headers_offset..function_headers_offset + SMALL_FUNC_HEADER_SIZE]
            .copy_from_slice(&encode_overflow_function_header(
                invalid_large_header_offset,
            ));

        let error =
            HermesBytecode::parse(&bytes).expect_err("invalid large function header must fail");
        assert_eq!(
            error,
            ParseError::LargeFunctionHeaderOutOfBounds {
                function_id: 0,
                offset: invalid_large_header_offset,
                file_length,
            },
        );
    }

    #[test]
    fn rejects_invalid_cjs_module_entry() {
        let mut bytes = fixture_bytecode();
        let cjs_module_table_offset = HermesBytecode::parse(&bytes)
            .expect("valid fixture")
            .sections()
            .cjs_module_table()
            .offset() as usize;
        write_u32(&mut bytes, cjs_module_table_offset + size_of::<u32>(), 99);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid CJS entry must fail");
        assert_eq!(
            error,
            ParseError::InvalidCjsModuleEntry {
                entry_index: 0,
                first: 0,
                second: 99,
                statically_resolved: false,
            },
        );
    }

    #[test]
    fn rejects_invalid_function_source_entry() {
        let mut bytes = fixture_bytecode();
        let function_source_table_offset = HermesBytecode::parse(&bytes)
            .expect("valid fixture")
            .sections()
            .function_source_table()
            .offset() as usize;
        write_u32(
            &mut bytes,
            function_source_table_offset + size_of::<u32>(),
            99,
        );

        let error = HermesBytecode::parse(&bytes).expect_err("invalid source entry must fail");
        assert_eq!(
            error,
            ParseError::InvalidFunctionSourceEntry {
                entry_index: 0,
                function_id: 1,
                string_id: 99,
            },
        );
    }
}
