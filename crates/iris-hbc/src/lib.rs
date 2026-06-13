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
        pub global_instruction_count: u32,
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

/// Number of opcodes in the Hermes bytecode table mirrored by Iris.
pub const HERMES_OPCODE_COUNT: usize = 220;

const HERMES_SOURCE_HASH_SIZE: usize = 20;
const SMALL_FUNC_HEADER_SIZE: usize = 12;
const LARGE_FUNC_HEADER_SIZE: usize = 36;
const STRING_TABLE_ENTRY_SIZE: usize = 8;
const STRING_KIND_ENTRY_SIZE: usize = 4;
const IDENTIFIER_HASH_SIZE: usize = 4;
const SMALL_STRING_TABLE_ENTRY_SIZE: usize = 4;
const OVERFLOW_STRING_TABLE_ENTRY_SIZE: usize = 8;
const SHAPE_TABLE_ENTRY_SIZE: usize = 8;
const BIG_INT_TABLE_ENTRY_SIZE: usize = 8;
const REG_EXP_TABLE_ENTRY_SIZE: usize = 8;
const U32_PAIR_ENTRY_SIZE: usize = 8;
const EXCEPTION_HANDLER_TABLE_HEADER_SIZE: usize = 4;
const EXCEPTION_HANDLER_ENTRY_SIZE: usize = 12;
const DEBUG_INFO_HEADER_SIZE: usize = 16;
const DEBUG_FILE_REGION_SIZE: usize = 12;
const DEBUG_OFFSETS_SIZE: usize = 4;
const DEBUG_OFFSETS_NO_OFFSET: u32 = u32::MAX;
const DEBUG_SOURCE_MAPPING_URL_INVALID: u32 = 0;
const DEBUG_DATA_TRUNCATED_LEB: &str = "truncated signed LEB128";
const DEBUG_DATA_LEB_OVERFLOW: &str = "signed LEB128 exceeds i64";
const DEBUG_DATA_FUNCTION_MISMATCH: &str = "function id mismatch";
const DEBUG_DATA_ADDRESS_OUT_OF_BOUNDS: &str = "address outside function body";
const DEBUG_DATA_ADDRESS_NOT_BOUNDARY: &str = "address is not an instruction boundary";
const DEBUG_DATA_SOURCE_VALUE_OUT_OF_BOUNDS: &str = "source location value outside u32";
const DEBUG_DATA_LOCATION_NOT_ONE_BASED: &str = "source location line or column is not one-based";
const STRING_TABLE_ENTRY_UTF16_MASK: u32 = 1 << 31;
const FUNCTION_INFO_ALIGNMENT: u32 = 4;
const SWITCH_TABLE_CASE_SIZE: usize = 8;
const SWITCH_TABLE_ALIGNMENT: u32 = 4;
const SERIALIZED_LITERAL_TAG_LONG_SEQUENCE: u8 = 0x80;
const SERIALIZED_LITERAL_TAG_MASK: u8 = 0x70;
const SERIALIZED_LITERAL_TAG_LENGTH_MASK: u8 = 0x0f;
const SERIALIZED_LITERAL_TAG_TRUE: u8 = 1 << 4;
const SERIALIZED_LITERAL_TAG_FALSE: u8 = 2 << 4;
const SERIALIZED_LITERAL_TAG_NUMBER: u8 = 3 << 4;
const SERIALIZED_LITERAL_TAG_LONG_STRING: u8 = 4 << 4;
const SERIALIZED_LITERAL_TAG_SHORT_STRING: u8 = 5 << 4;
const SERIALIZED_LITERAL_TAG_UNDEFINED: u8 = 6 << 4;
const SERIALIZED_LITERAL_TAG_INTEGER: u8 = 7 << 4;
const HERMES_OPCODE_WIDTHS: [u8; HERMES_OPCODE_COUNT] = [
    1, 6, 10, 11, 12, 2, 3, 8, 10, 4, 5, 3, 4, 4, 3, 3, 3, 9, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 4, 4, 5, 5, 3, 4, 3, 4, 5, 4, 5, 4, 5, 2,
    2, 3, 3, 6, 7, 5, 5, 6, 8, 9, 6, 8, 6, 6, 8, 8, 6, 6, 8, 8, 4, 7, 4, 7, 6, 8, 4, 7, 4, 5, 5, 4,
    4, 5, 4, 4, 6, 5, 4, 5, 5, 6, 5, 6, 4, 4, 4, 5, 5, 6, 7, 5, 7, 4, 7, 3, 2, 2, 4, 2, 3, 3, 2, 1,
    1, 3, 6, 8, 7, 9, 5, 7, 4, 5, 4, 3, 6, 3, 6, 10, 4, 6, 4, 6, 2, 2, 2, 2, 2, 2, 3, 2, 3, 3, 3,
    3, 3, 6, 4, 4, 3, 2, 2, 3, 14, 18, 18, 5, 7, 3, 4, 3, 3, 2, 5, 3, 6, 3, 6, 3, 6, 8, 4, 7, 4, 7,
    4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7, 4, 7,
];

const BYTECODE_TABLE_OPERANDS: [BytecodeTableOperand; 35] = [
    BytecodeTableOperand::new(1, 2, 2, BytecodeTable::ObjectShape),
    BytecodeTableOperand::new(2, 2, 4, BytecodeTable::ObjectShape),
    BytecodeTableOperand::new(3, 3, 4, BytecodeTable::ObjectShape),
    BytecodeTableOperand::new(4, 3, 4, BytecodeTable::ObjectShape),
    BytecodeTableOperand::new(67, 1, 4, BytecodeTable::String),
    BytecodeTableOperand::new(68, 4, 1, BytecodeTable::String),
    BytecodeTableOperand::new(69, 4, 2, BytecodeTable::String),
    BytecodeTableOperand::new(70, 4, 4, BytecodeTable::String),
    BytecodeTableOperand::new(71, 5, 4, BytecodeTable::String),
    BytecodeTableOperand::new(72, 4, 2, BytecodeTable::String),
    BytecodeTableOperand::new(73, 4, 4, BytecodeTable::String),
    BytecodeTableOperand::new(74, 4, 2, BytecodeTable::String),
    BytecodeTableOperand::new(75, 4, 2, BytecodeTable::String),
    BytecodeTableOperand::new(76, 4, 4, BytecodeTable::String),
    BytecodeTableOperand::new(77, 4, 4, BytecodeTable::String),
    BytecodeTableOperand::new(78, 4, 2, BytecodeTable::String),
    BytecodeTableOperand::new(79, 4, 2, BytecodeTable::String),
    BytecodeTableOperand::new(80, 4, 4, BytecodeTable::String),
    BytecodeTableOperand::new(81, 4, 4, BytecodeTable::String),
    BytecodeTableOperand::new(87, 4, 4, BytecodeTable::String),
    BytecodeTableOperand::new(128, 4, 2, BytecodeTable::Function),
    BytecodeTableOperand::new(129, 4, 4, BytecodeTable::Function),
    BytecodeTableOperand::new(130, 5, 2, BytecodeTable::Function),
    BytecodeTableOperand::new(131, 5, 4, BytecodeTable::Function),
    BytecodeTableOperand::new(132, 3, 2, BytecodeTable::Function),
    BytecodeTableOperand::new(133, 3, 4, BytecodeTable::Function),
    BytecodeTableOperand::new(142, 2, 2, BytecodeTable::BigInt),
    BytecodeTableOperand::new(143, 2, 4, BytecodeTable::BigInt),
    BytecodeTableOperand::new(144, 2, 2, BytecodeTable::String),
    BytecodeTableOperand::new(145, 2, 4, BytecodeTable::String),
    BytecodeTableOperand::new(159, 2, 4, BytecodeTable::String),
    BytecodeTableOperand::new(166, 2, 4, BytecodeTable::String),
    BytecodeTableOperand::new(166, 6, 4, BytecodeTable::String),
    BytecodeTableOperand::new(169, 3, 2, BytecodeTable::Function),
    BytecodeTableOperand::new(170, 3, 4, BytecodeTable::Function),
];

const OBJECT_LITERAL_OPERANDS: [ObjectLiteralOperand; 4] = [
    ObjectLiteralOperand::new(1, 2, 2, 4, 2),
    ObjectLiteralOperand::new(2, 2, 4, 6, 4),
    ObjectLiteralOperand::new(3, 3, 4, 7, 4),
    ObjectLiteralOperand::new(4, 3, 4, 7, 4),
];

const ARRAY_LITERAL_OPERANDS: [ArrayLiteralOperand; 2] = [
    ArrayLiteralOperand::new(7, 4, 2, 6, 2),
    ArrayLiteralOperand::new(8, 4, 2, 6, 4),
];

const JUMP_OPERANDS: [JumpOperand; 45] = [
    JumpOperand::new(175, 1, 1),
    JumpOperand::new(176, 1, 4),
    JumpOperand::new(177, 1, 1),
    JumpOperand::new(178, 1, 4),
    JumpOperand::new(179, 1, 1),
    JumpOperand::new(180, 1, 4),
    JumpOperand::new(181, 1, 1),
    JumpOperand::new(182, 1, 4),
    JumpOperand::new(183, 1, 4),
    JumpOperand::new(184, 1, 1),
    JumpOperand::new(185, 1, 4),
    JumpOperand::new(186, 1, 1),
    JumpOperand::new(187, 1, 4),
    JumpOperand::new(188, 1, 1),
    JumpOperand::new(189, 1, 4),
    JumpOperand::new(190, 1, 1),
    JumpOperand::new(191, 1, 4),
    JumpOperand::new(192, 1, 1),
    JumpOperand::new(193, 1, 4),
    JumpOperand::new(194, 1, 1),
    JumpOperand::new(195, 1, 4),
    JumpOperand::new(196, 1, 1),
    JumpOperand::new(197, 1, 4),
    JumpOperand::new(198, 1, 1),
    JumpOperand::new(199, 1, 4),
    JumpOperand::new(200, 1, 1),
    JumpOperand::new(201, 1, 4),
    JumpOperand::new(202, 1, 1),
    JumpOperand::new(203, 1, 4),
    JumpOperand::new(204, 1, 1),
    JumpOperand::new(205, 1, 4),
    JumpOperand::new(206, 1, 1),
    JumpOperand::new(207, 1, 4),
    JumpOperand::new(208, 1, 1),
    JumpOperand::new(209, 1, 4),
    JumpOperand::new(210, 1, 1),
    JumpOperand::new(211, 1, 4),
    JumpOperand::new(212, 1, 1),
    JumpOperand::new(213, 1, 4),
    JumpOperand::new(214, 1, 1),
    JumpOperand::new(215, 1, 4),
    JumpOperand::new(216, 1, 1),
    JumpOperand::new(217, 1, 4),
    JumpOperand::new(218, 1, 1),
    JumpOperand::new(219, 1, 4),
];

const UINT_SWITCH_IMM_OPCODE: u8 = 167;
const STRING_SWITCH_IMM_OPCODE: u8 = 168;

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

    /// Returns the bytecode body for one function.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if `function_id` is invalid or the function
    /// header points outside the bytecode file.
    pub fn function_body(self, function_id: u32) -> Result<SectionView<'a>, ParseError> {
        let function_header = self.function_header(function_id)?;
        function_body(self.bytes, function_id, function_header)
    }

    /// Returns the global function bytecode body.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if the global function header cannot be read.
    pub fn global_function_body(self) -> Result<SectionView<'a>, ParseError> {
        self.function_body(self.header.global_code_index)
    }

    /// Returns an iterator over one function's Hermes instructions.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if `function_id` is invalid or the function
    /// body is not a valid instruction stream.
    pub fn function_instructions(
        self,
        function_id: u32,
    ) -> Result<HermesInstructionStream<'a>, ParseError> {
        let body = self.function_body(function_id)?;
        Ok(HermesInstructionStream::new(function_id, body))
    }

    /// Counts decoded Hermes instructions in one function.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if `function_id` is invalid or the function
    /// body is not a valid instruction stream.
    pub fn function_instruction_count(self, function_id: u32) -> Result<u32, ParseError> {
        count_function_instructions(function_id, self.function_body(function_id)?)
    }

    /// Counts decoded Hermes instructions in the global function.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if the global function body cannot be decoded.
    pub fn global_instruction_count(self) -> Result<u32, ParseError> {
        self.function_instruction_count(self.header.global_code_index)
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
        validate_object_shape_table(obj_shape_table.bytes, obj_key_buffer.bytes, header)?;
        validate_storage_table(
            "big_int_table",
            big_int_table.bytes,
            header.big_int_storage_size,
        )?;
        validate_storage_table(
            "reg_exp_table",
            reg_exp_table.bytes,
            header.reg_exp_storage_size,
        )?;
        let debug_info = parse_debug_info(bytes, header)?;
        validate_function_headers(
            bytes,
            header,
            function_headers,
            function_bodies_offset,
            literal_value_buffer,
            obj_shape_table,
            debug_info,
        )?;
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

/// One decoded Hermes bytecode instruction boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HermesInstruction {
    /// File offset of the instruction opcode byte.
    pub offset: u32,
    /// Raw Hermes opcode value.
    pub opcode: u8,
    /// Instruction size including the opcode byte and all operands.
    pub width: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BytecodeTable {
    String,
    Function,
    BigInt,
    ObjectShape,
}

impl BytecodeTable {
    const fn name(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Function => "function",
            Self::BigInt => "bigint",
            Self::ObjectShape => "object_shape",
        }
    }

    const fn limit(self, header: HermesBytecodeHeader) -> u32 {
        match self {
            Self::String => header.string_count,
            Self::Function => header.function_count,
            Self::BigInt => header.big_int_count,
            Self::ObjectShape => header.obj_shape_table_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BytecodeTableOperand {
    opcode: u8,
    offset: u8,
    width: u8,
    table: BytecodeTable,
}

impl BytecodeTableOperand {
    const fn new(opcode: u8, offset: u8, width: u8, table: BytecodeTable) -> Self {
        Self {
            opcode,
            offset,
            width,
            table,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectLiteralOperand {
    opcode: u8,
    shape_offset: u8,
    shape_width: u8,
    value_offset: u8,
    value_width: u8,
}

impl ObjectLiteralOperand {
    const fn new(
        opcode: u8,
        shape_offset: u8,
        shape_width: u8,
        value_offset: u8,
        value_width: u8,
    ) -> Self {
        Self {
            opcode,
            shape_offset,
            shape_width,
            value_offset,
            value_width,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArrayLiteralOperand {
    opcode: u8,
    element_count_offset: u8,
    element_count_width: u8,
    value_offset: u8,
    value_width: u8,
}

impl ArrayLiteralOperand {
    const fn new(
        opcode: u8,
        element_count_offset: u8,
        element_count_width: u8,
        value_offset: u8,
        value_width: u8,
    ) -> Self {
        Self {
            opcode,
            element_count_offset,
            element_count_width,
            value_offset,
            value_width,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JumpOperand {
    opcode: u8,
    offset: u8,
    width: u8,
}

impl JumpOperand {
    const fn new(opcode: u8, offset: u8, width: u8) -> Self {
        Self {
            opcode,
            offset,
            width,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ObjectShapeEntry {
    key_buffer_offset: u32,
    num_props: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InstructionValidationContext<'a> {
    header: HermesBytecodeHeader,
    function_region_limit: u32,
    literal_value_buffer: &'a [u8],
    obj_shape_table: &'a [u8],
}

/// Iterator over a Hermes function bytecode body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HermesInstructionStream<'a> {
    function_id: u32,
    body: SectionView<'a>,
    cursor: usize,
    finished: bool,
}

impl<'a> HermesInstructionStream<'a> {
    fn new(function_id: u32, body: SectionView<'a>) -> Self {
        Self {
            function_id,
            body,
            cursor: 0,
            finished: false,
        }
    }
}

impl Iterator for HermesInstructionStream<'_> {
    type Item = Result<HermesInstruction, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.cursor >= self.body.bytes.len() {
            return None;
        }

        let offset = self.body.offset
            + u32::try_from(self.cursor).expect(
                "validated Hermes instruction cursor fits in u32 because file length is u32",
            );
        let opcode = self.body.bytes[self.cursor];
        let width = match hermes_opcode_width(opcode) {
            Some(width) => width,
            None => {
                self.finished = true;
                return Some(Err(ParseError::InvalidOpcode {
                    function_id: self.function_id,
                    offset,
                    opcode,
                }));
            }
        };
        let next_cursor = self.cursor + usize::from(width);
        if next_cursor > self.body.bytes.len() {
            self.finished = true;
            return Some(Err(ParseError::InstructionOutOfBounds {
                function_id: self.function_id,
                offset,
                opcode,
                width,
                body_end: self.body.offset + self.body.len(),
            }));
        }

        self.cursor = next_cursor;
        Some(Ok(HermesInstruction {
            offset,
            opcode,
            width,
        }))
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
    /// A storage-backed table entry points outside its byte storage section.
    StorageTableEntryOutOfBounds {
        /// Table section name.
        section: &'static str,
        /// Table entry index.
        entry_index: u32,
        /// Byte offset into the storage section.
        offset: u32,
        /// Entry byte length.
        length: u32,
        /// Declared storage section byte length.
        storage_size: u32,
    },
    /// A serialized literal buffer reference cannot contain the requested items.
    InvalidSerializedLiteralBuffer {
        /// Buffer section name.
        buffer: &'static str,
        /// Byte offset into the buffer.
        offset: u32,
        /// Number of logical literal entries requested.
        element_count: u32,
        /// Buffer byte length.
        buffer_size: u32,
    },
    /// A serialized literal buffer uses a tag that is invalid for the buffer kind.
    InvalidSerializedLiteralTag {
        /// Buffer section name.
        buffer: &'static str,
        /// Byte offset of the serialized tag.
        offset: u32,
        /// Raw tag byte.
        tag: u8,
    },
    /// A serialized literal buffer references an invalid string id.
    InvalidSerializedLiteralStringReference {
        /// Buffer section name.
        buffer: &'static str,
        /// Byte offset of the serialized string id.
        offset: u32,
        /// Referenced string id.
        string_id: u32,
        /// Declared string table count.
        string_count: u32,
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
        /// Exclusive function body region end offset.
        limit: u32,
    },
    /// A function declares extra metadata but has no function info region.
    MissingFunctionInfo {
        /// Function id whose info region is missing.
        function_id: u32,
    },
    /// A function info table extends outside its allowed region.
    FunctionInfoOutOfBounds {
        /// Function id whose info table is invalid.
        function_id: u32,
        /// Table byte offset.
        offset: u32,
        /// Table byte size.
        size: u32,
        /// Exclusive function info region end offset.
        limit: u32,
    },
    /// The global debug info section extends outside the declared file body.
    DebugInfoOutOfBounds {
        /// Debug info byte offset.
        offset: u32,
        /// Debug info byte size.
        size: u32,
        /// Exclusive debug info section limit before the footer.
        limit: u32,
    },
    /// A debug filename table entry points outside filename storage.
    DebugFilenameOutOfBounds {
        /// Filename table entry index.
        filename_index: u32,
        /// Filename byte offset in debug filename storage.
        offset: u32,
        /// Filename byte length.
        length: u32,
        /// Declared debug filename storage byte length.
        storage_size: u32,
    },
    /// A debug file region references invalid debug data or filename ids.
    InvalidDebugFileRegion {
        /// Debug file region entry index.
        region_index: u32,
        /// Offset into the debug data payload where this region starts.
        from_address: u32,
        /// Referenced debug filename table id.
        filename_id: u32,
        /// Referenced source map URL debug filename table id.
        source_mapping_url_id: u32,
        /// Declared debug filename table entry count.
        filename_count: u32,
        /// Declared debug data byte length.
        debug_data_size: u32,
    },
    /// Debug file regions are not sorted by debug data address.
    InvalidDebugFileRegionOrder {
        /// Debug file region entry index.
        region_index: u32,
        /// Previous file region debug data offset.
        previous_from_address: u32,
        /// Current file region debug data offset.
        from_address: u32,
    },
    /// A function points at debug data outside the global debug data payload.
    InvalidDebugOffset {
        /// Function id whose debug offsets are invalid.
        function_id: u32,
        /// Offset into the global debug data payload.
        source_locations: u32,
        /// Declared global debug data byte length.
        debug_data_size: u32,
    },
    /// A function debug source-location stream is not decodable by Hermes rules.
    InvalidDebugData {
        /// Function id whose debug data is invalid.
        function_id: u32,
        /// Offset into the global debug data payload where validation failed.
        offset: u32,
        /// Static validation failure reason.
        reason: &'static str,
    },
    /// An exception handler entry points outside the function body boundaries.
    InvalidExceptionHandler {
        /// Function id whose exception table is invalid.
        function_id: u32,
        /// Exception table entry index.
        entry_index: u32,
        /// Try range start offset relative to the function bytecode.
        start: u32,
        /// Try range end offset relative to the function bytecode.
        end: u32,
        /// Handler target offset relative to the function bytecode.
        target: u32,
        /// Function bytecode body size.
        body_size: u32,
    },
    /// A function bytecode body contains an opcode not known to this Hermes table.
    InvalidOpcode {
        /// Function id whose instruction stream is invalid.
        function_id: u32,
        /// File offset of the invalid opcode.
        offset: u32,
        /// Raw opcode byte.
        opcode: u8,
    },
    /// An instruction's declared operand width exceeds the function body.
    InstructionOutOfBounds {
        /// Function id whose instruction stream is invalid.
        function_id: u32,
        /// File offset of the opcode.
        offset: u32,
        /// Raw opcode byte.
        opcode: u8,
        /// Instruction width in bytes.
        width: u8,
        /// Exclusive end offset of the function body.
        body_end: u32,
    },
    /// An instruction references an invalid bytecode table entry.
    InvalidInstructionTableReference {
        /// Function id whose instruction stream is invalid.
        function_id: u32,
        /// File offset of the opcode.
        offset: u32,
        /// Raw opcode byte.
        opcode: u8,
        /// Referenced table name.
        table: &'static str,
        /// Referenced table index.
        index: u32,
        /// Declared table entry count.
        limit: u32,
    },
    /// A branch target is outside the function body or not on an instruction boundary.
    InvalidJumpTarget {
        /// Function id whose instruction stream is invalid.
        function_id: u32,
        /// File offset of the opcode.
        offset: u32,
        /// Raw opcode byte.
        opcode: u8,
        /// Computed absolute target offset.
        target: i64,
        /// Function body start offset.
        body_start: u32,
        /// Exclusive function body end offset.
        body_end: u32,
    },
    /// A switch instruction has an invalid table range.
    InvalidSwitchTableRange {
        /// Function id whose instruction stream is invalid.
        function_id: u32,
        /// File offset of the opcode.
        offset: u32,
        /// Raw opcode byte.
        opcode: u8,
        /// Switch minimum value.
        minimum: u32,
        /// Switch maximum value.
        maximum: u32,
    },
    /// A switch instruction references a table outside the function payload.
    SwitchTableOutOfBounds {
        /// Function id whose instruction stream is invalid.
        function_id: u32,
        /// File offset of the opcode.
        offset: u32,
        /// Raw opcode byte.
        opcode: u8,
        /// Absolute switch table start offset.
        table_start: u32,
        /// Switch table size in bytes.
        table_size: u32,
        /// Exclusive function payload end offset.
        region_limit: u32,
    },
    /// A switch table entry references an invalid branch target.
    InvalidSwitchTableTarget {
        /// Function id whose instruction stream is invalid.
        function_id: u32,
        /// File offset of the opcode.
        offset: u32,
        /// Raw opcode byte.
        opcode: u8,
        /// Switch table entry index.
        entry_index: u32,
        /// Computed absolute target offset.
        target: i64,
        /// Function body start offset.
        body_start: u32,
        /// Exclusive function body end offset.
        body_end: u32,
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
            Self::StorageTableEntryOutOfBounds {
                section,
                entry_index,
                offset,
                length,
                storage_size,
            } => write!(
                formatter,
                "Hermes bytecode {section} entry {entry_index} at offset {offset} with length {length} exceeds storage size {storage_size}",
            ),
            Self::InvalidSerializedLiteralBuffer {
                buffer,
                offset,
                element_count,
                buffer_size,
            } => write!(
                formatter,
                "Hermes bytecode {buffer} at offset {offset} cannot provide {element_count} serialized literal entries within buffer size {buffer_size}",
            ),
            Self::InvalidSerializedLiteralTag {
                buffer,
                offset,
                tag,
            } => write!(
                formatter,
                "Hermes bytecode {buffer} has invalid serialized literal tag {tag:#04x} at offset {offset}",
            ),
            Self::InvalidSerializedLiteralStringReference {
                buffer,
                offset,
                string_id,
                string_count,
            } => write!(
                formatter,
                "Hermes bytecode {buffer} at offset {offset} references string id {string_id}, but the string table count is {string_count}",
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
            Self::MissingFunctionInfo { function_id } => write!(
                formatter,
                "Hermes bytecode function {function_id} declares function info but has no function info offset",
            ),
            Self::FunctionInfoOutOfBounds {
                function_id,
                offset,
                size,
                limit,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} info table at {offset} with size {size} exceeds function info limit {limit}",
            ),
            Self::DebugInfoOutOfBounds {
                offset,
                size,
                limit,
            } => write!(
                formatter,
                "Hermes bytecode debug info at {offset} with size {size} exceeds debug info limit {limit}",
            ),
            Self::DebugFilenameOutOfBounds {
                filename_index,
                offset,
                length,
                storage_size,
            } => write!(
                formatter,
                "Hermes bytecode debug filename {filename_index} at offset {offset} with length {length} exceeds filename storage size {storage_size}",
            ),
            Self::InvalidDebugFileRegion {
                region_index,
                from_address,
                filename_id,
                source_mapping_url_id,
                filename_count,
                debug_data_size,
            } => write!(
                formatter,
                "Hermes bytecode debug file region {region_index} at debug data offset {from_address} references filename {filename_id} and source map URL {source_mapping_url_id}; filename count is {filename_count}, debug data size is {debug_data_size}",
            ),
            Self::InvalidDebugFileRegionOrder {
                region_index,
                previous_from_address,
                from_address,
            } => write!(
                formatter,
                "Hermes bytecode debug file region {region_index} starts at {from_address}, not after previous region start {previous_from_address}",
            ),
            Self::InvalidDebugOffset {
                function_id,
                source_locations,
                debug_data_size,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} debug source locations offset {source_locations} exceeds debug data size {debug_data_size}",
            ),
            Self::InvalidDebugData {
                function_id,
                offset,
                reason,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} debug data at offset {offset} is invalid: {reason}",
            ),
            Self::InvalidExceptionHandler {
                function_id,
                entry_index,
                start,
                end,
                target,
                body_size,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} exception handler {entry_index} has range {start}..{end} and target {target}, outside function body size {body_size}",
            ),
            Self::InvalidOpcode {
                function_id,
                offset,
                opcode,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} has invalid opcode {opcode} at {offset}",
            ),
            Self::InstructionOutOfBounds {
                function_id,
                offset,
                opcode,
                width,
                body_end,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} instruction opcode {opcode} at {offset} has width {width}, exceeding function body end {body_end}",
            ),
            Self::InvalidInstructionTableReference {
                function_id,
                offset,
                opcode,
                table,
                index,
                limit,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} instruction opcode {opcode} at {offset} references {table} table index {index}, but the table count is {limit}",
            ),
            Self::InvalidJumpTarget {
                function_id,
                offset,
                opcode,
                target,
                body_start,
                body_end,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} instruction opcode {opcode} at {offset} jumps to {target}, outside instruction boundaries {body_start}..{body_end}",
            ),
            Self::InvalidSwitchTableRange {
                function_id,
                offset,
                opcode,
                minimum,
                maximum,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} switch opcode {opcode} at {offset} has invalid range {minimum}..={maximum}",
            ),
            Self::SwitchTableOutOfBounds {
                function_id,
                offset,
                opcode,
                table_start,
                table_size,
                region_limit,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} switch opcode {opcode} at {offset} has table {table_start}+{table_size}, exceeding function payload end {region_limit}",
            ),
            Self::InvalidSwitchTableTarget {
                function_id,
                offset,
                opcode,
                entry_index,
                target,
                body_start,
                body_end,
            } => write!(
                formatter,
                "Hermes bytecode function {function_id} switch opcode {opcode} at {offset} table entry {entry_index} jumps to {target}, outside instruction boundaries {body_start}..{body_end}",
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
    let global_instruction_count = bytecode.global_instruction_count()?;
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
        global_instruction_count,
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

fn validate_storage_table(
    section: &'static str,
    entries: &[u8],
    storage_size: u32,
) -> Result<(), ParseError> {
    for (index, entry) in entries.chunks_exact(U32_PAIR_ENTRY_SIZE).enumerate() {
        let offset = read_u32(entry, 0);
        let length = read_u32(entry, 4);
        if offset
            .checked_add(length)
            .is_none_or(|end| end > storage_size)
        {
            return Err(ParseError::StorageTableEntryOutOfBounds {
                section,
                entry_index: u32::try_from(index)
                    .expect("validated storage table index fits in u32"),
                offset,
                length,
                storage_size,
            });
        }
    }
    Ok(())
}

fn validate_object_shape_table(
    entries: &[u8],
    obj_key_buffer: &[u8],
    header: HermesBytecodeHeader,
) -> Result<(), ParseError> {
    for entry in entries.chunks_exact(SHAPE_TABLE_ENTRY_SIZE) {
        let shape = object_shape_entry(entry);
        validate_serialized_literal_buffer(
            "obj_key_buffer",
            obj_key_buffer,
            shape.key_buffer_offset,
            shape.num_props,
            true,
            header,
        )?;
    }
    Ok(())
}

fn object_shape_entry(bytes: &[u8]) -> ObjectShapeEntry {
    ObjectShapeEntry {
        key_buffer_offset: read_u32(bytes, 0),
        num_props: read_u32(bytes, 4),
    }
}

fn object_shape_entry_at(entries: &[u8], shape_index: u32) -> Option<ObjectShapeEntry> {
    let start = usize::try_from(shape_index)
        .expect("u32 always fits in usize on supported targets")
        .checked_mul(SHAPE_TABLE_ENTRY_SIZE)?;
    let end = start.checked_add(SHAPE_TABLE_ENTRY_SIZE)?;
    entries.get(start..end).map(object_shape_entry)
}

fn validate_serialized_literal_buffer(
    buffer_name: &'static str,
    buffer: &[u8],
    offset: u32,
    element_count: u32,
    is_key_buffer: bool,
    header: HermesBytecodeHeader,
) -> Result<(), ParseError> {
    let buffer_size =
        u32::try_from(buffer.len()).expect("validated HBC section length fits in u32");
    if offset > buffer_size {
        return Err(ParseError::InvalidSerializedLiteralBuffer {
            buffer: buffer_name,
            offset,
            element_count,
            buffer_size,
        });
    }

    let mut remaining = element_count;
    let mut cursor =
        usize::try_from(offset).expect("u32 always fits in usize on supported targets");
    while remaining > 0 {
        let tag_offset = cursor;
        let Some(&tag) = buffer.get(cursor) else {
            return Err(ParseError::InvalidSerializedLiteralBuffer {
                buffer: buffer_name,
                offset,
                element_count,
                buffer_size,
            });
        };
        cursor += 1;

        let sequence_len = if tag & SERIALIZED_LITERAL_TAG_LONG_SEQUENCE != 0 {
            let Some(&low_len) = buffer.get(cursor) else {
                return Err(ParseError::InvalidSerializedLiteralBuffer {
                    buffer: buffer_name,
                    offset,
                    element_count,
                    buffer_size,
                });
            };
            cursor += 1;
            (u32::from(tag & SERIALIZED_LITERAL_TAG_LENGTH_MASK) << 8) | u32::from(low_len)
        } else {
            u32::from(tag & SERIALIZED_LITERAL_TAG_LENGTH_MASK)
        };
        if sequence_len == 0 {
            return Err(ParseError::InvalidSerializedLiteralBuffer {
                buffer: buffer_name,
                offset,
                element_count,
                buffer_size,
            });
        }

        let tag_type = tag & SERIALIZED_LITERAL_TAG_MASK;
        if is_key_buffer
            && matches!(
                tag_type,
                SERIALIZED_LITERAL_TAG_TRUE
                    | SERIALIZED_LITERAL_TAG_FALSE
                    | SERIALIZED_LITERAL_TAG_UNDEFINED
            )
        {
            return Err(ParseError::InvalidSerializedLiteralTag {
                buffer: buffer_name,
                offset: u32::try_from(tag_offset)
                    .expect("validated HBC section offset fits in u32"),
                tag,
            });
        }

        let to_read = remaining.min(sequence_len);
        let data_size = serialized_literal_data_size(tag_type);
        let bytes_to_read =
            to_read
                .checked_mul(data_size)
                .ok_or(ParseError::InvalidSerializedLiteralBuffer {
                    buffer: buffer_name,
                    offset,
                    element_count,
                    buffer_size,
                })?;
        let end = cursor
            .checked_add(
                usize::try_from(bytes_to_read)
                    .expect("u32 always fits in usize on supported targets"),
            )
            .ok_or(ParseError::InvalidSerializedLiteralBuffer {
                buffer: buffer_name,
                offset,
                element_count,
                buffer_size,
            })?;
        if end > buffer.len() {
            return Err(ParseError::InvalidSerializedLiteralBuffer {
                buffer: buffer_name,
                offset,
                element_count,
                buffer_size,
            });
        }

        validate_serialized_literal_strings(
            buffer_name,
            buffer,
            cursor,
            to_read,
            tag_type,
            header.string_count,
        )?;
        cursor = end;
        remaining -= to_read;
    }
    Ok(())
}

fn serialized_literal_data_size(tag_type: u8) -> u32 {
    match tag_type {
        SERIALIZED_LITERAL_TAG_SHORT_STRING => 2,
        SERIALIZED_LITERAL_TAG_LONG_STRING
        | SERIALIZED_LITERAL_TAG_INTEGER
        | SERIALIZED_LITERAL_TAG_NUMBER => {
            if tag_type == SERIALIZED_LITERAL_TAG_NUMBER {
                8
            } else {
                4
            }
        }
        _ => 0,
    }
}

fn validate_serialized_literal_strings(
    buffer_name: &'static str,
    buffer: &[u8],
    start: usize,
    element_count: u32,
    tag_type: u8,
    string_count: u32,
) -> Result<(), ParseError> {
    match tag_type {
        SERIALIZED_LITERAL_TAG_SHORT_STRING => {
            for index in 0..element_count {
                let offset = start
                    + usize::try_from(index)
                        .expect("u32 always fits in usize on supported targets")
                        * 2;
                let string_id = u32::from(u16::from_le_bytes(
                    buffer[offset..offset + 2]
                        .try_into()
                        .expect("validated serialized literal string id has u16 width"),
                ));
                validate_serialized_literal_string(buffer_name, offset, string_id, string_count)?;
            }
        }
        SERIALIZED_LITERAL_TAG_LONG_STRING => {
            for index in 0..element_count {
                let offset = start
                    + usize::try_from(index)
                        .expect("u32 always fits in usize on supported targets")
                        * 4;
                validate_serialized_literal_string(
                    buffer_name,
                    offset,
                    read_u32(buffer, offset),
                    string_count,
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_serialized_literal_string(
    buffer_name: &'static str,
    offset: usize,
    string_id: u32,
    string_count: u32,
) -> Result<(), ParseError> {
    if string_id >= string_count {
        return Err(ParseError::InvalidSerializedLiteralStringReference {
            buffer: buffer_name,
            offset: u32::try_from(offset).expect("validated HBC section offset fits in u32"),
            string_id,
            string_count,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DecodedSmallFunctionHeader {
    header: HermesFunctionHeader,
    overflowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DecodedFunctionHeader {
    header: HermesFunctionHeader,
    large_header_offset: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DebugInfoBounds {
    debug_data_offset: u32,
    debug_data_size: u32,
}

fn function_header_at(
    bytes: &[u8],
    header: HermesBytecodeHeader,
    function_headers: SectionView<'_>,
    function_id: u32,
) -> Result<HermesFunctionHeader, ParseError> {
    decoded_function_header_at(bytes, header, function_headers, function_id)
        .map(|decoded| decoded.header)
}

fn decoded_function_header_at(
    bytes: &[u8],
    header: HermesBytecodeHeader,
    function_headers: SectionView<'_>,
    function_id: u32,
) -> Result<DecodedFunctionHeader, ParseError> {
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
        return Ok(DecodedFunctionHeader {
            header: decoded.header,
            large_header_offset: None,
        });
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

    Ok(DecodedFunctionHeader {
        header: parse_large_function_header(&bytes[large_header_start..large_header_end]),
        large_header_offset: Some(large_header_offset),
    })
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

fn parse_debug_info(
    bytes: &[u8],
    header: HermesBytecodeHeader,
) -> Result<Option<DebugInfoBounds>, ParseError> {
    if header.debug_info_offset == 0 {
        return Ok(None);
    }

    let limit = header
        .file_length
        .saturating_sub(HERMES_BYTECODE_FOOTER_SIZE as u32);
    let debug_header = debug_info_bytes(
        bytes,
        header.debug_info_offset,
        DEBUG_INFO_HEADER_SIZE as u32,
        limit,
    )?;
    let filename_count = read_u32(debug_header, 0);
    let filename_storage_size = read_u32(debug_header, 4);
    let file_region_count = read_u32(debug_header, 8);
    let debug_data_size = read_u32(debug_header, 12);

    let filename_table_size = filename_count
        .checked_mul(STRING_TABLE_ENTRY_SIZE as u32)
        .ok_or(ParseError::DebugInfoOutOfBounds {
            offset: header
                .debug_info_offset
                .saturating_add(DEBUG_INFO_HEADER_SIZE as u32),
            size: u32::MAX,
            limit,
        })?;
    let file_region_table_size = file_region_count
        .checked_mul(DEBUG_FILE_REGION_SIZE as u32)
        .ok_or(ParseError::DebugInfoOutOfBounds {
            offset: header
                .debug_info_offset
                .saturating_add(DEBUG_INFO_HEADER_SIZE as u32),
            size: u32::MAX,
            limit,
        })?;

    let mut cursor = header
        .debug_info_offset
        .checked_add(DEBUG_INFO_HEADER_SIZE as u32)
        .ok_or(ParseError::DebugInfoOutOfBounds {
            offset: header.debug_info_offset,
            size: DEBUG_INFO_HEADER_SIZE as u32,
            limit,
        })?;
    let filename_table_offset = cursor;
    cursor = checked_debug_info_end(cursor, filename_table_size, limit)?;
    let filename_storage_offset = cursor;
    cursor = checked_debug_info_end(cursor, filename_storage_size, limit)?;
    let file_region_table_offset = cursor;
    cursor = checked_debug_info_end(cursor, file_region_table_size, limit)?;
    let debug_data_offset = cursor;
    checked_debug_info_end(cursor, debug_data_size, limit)?;

    let filename_table =
        debug_info_bytes(bytes, filename_table_offset, filename_table_size, limit)?;
    validate_debug_filename_table(filename_table, filename_storage_size)?;
    debug_info_bytes(bytes, filename_storage_offset, filename_storage_size, limit)?;
    let file_region_table = debug_info_bytes(
        bytes,
        file_region_table_offset,
        file_region_table_size,
        limit,
    )?;
    validate_debug_file_regions(file_region_table, filename_count, debug_data_size)?;

    Ok(Some(DebugInfoBounds {
        debug_data_offset,
        debug_data_size,
    }))
}

fn validate_debug_filename_table(bytes: &[u8], storage_size: u32) -> Result<(), ParseError> {
    for (index, entry) in bytes.chunks_exact(STRING_TABLE_ENTRY_SIZE).enumerate() {
        let offset = read_u32(entry, 0);
        let raw_length = read_u32(entry, 4);
        let length = raw_length & !STRING_TABLE_ENTRY_UTF16_MASK;
        let byte_length = if raw_length & STRING_TABLE_ENTRY_UTF16_MASK == 0 {
            length
        } else {
            length
                .checked_mul(2)
                .ok_or(ParseError::DebugFilenameOutOfBounds {
                    filename_index: u32::try_from(index)
                        .expect("validated debug filename index fits in u32"),
                    offset,
                    length: u32::MAX,
                    storage_size,
                })?
        };
        if offset
            .checked_add(byte_length)
            .is_none_or(|end| end > storage_size)
        {
            return Err(ParseError::DebugFilenameOutOfBounds {
                filename_index: u32::try_from(index)
                    .expect("validated debug filename index fits in u32"),
                offset,
                length: byte_length,
                storage_size,
            });
        }
    }
    Ok(())
}

fn validate_debug_file_regions(
    bytes: &[u8],
    filename_count: u32,
    debug_data_size: u32,
) -> Result<(), ParseError> {
    let mut previous_from_address = None;
    for (index, region) in bytes.chunks_exact(DEBUG_FILE_REGION_SIZE).enumerate() {
        let from_address = read_u32(region, 0);
        let filename_id = read_u32(region, 4);
        let source_mapping_url_id = read_u32(region, 8);
        if let Some(previous) = previous_from_address {
            if from_address <= previous {
                return Err(ParseError::InvalidDebugFileRegionOrder {
                    region_index: u32::try_from(index)
                        .expect("validated debug file region index fits in u32"),
                    previous_from_address: previous,
                    from_address,
                });
            }
        }
        let valid_source_mapping_url = source_mapping_url_id == DEBUG_SOURCE_MAPPING_URL_INVALID
            || source_mapping_url_id < filename_count;
        if from_address >= debug_data_size
            || filename_id >= filename_count
            || !valid_source_mapping_url
        {
            return Err(ParseError::InvalidDebugFileRegion {
                region_index: u32::try_from(index)
                    .expect("validated debug file region index fits in u32"),
                from_address,
                filename_id,
                source_mapping_url_id,
                filename_count,
                debug_data_size,
            });
        }
        previous_from_address = Some(from_address);
    }
    Ok(())
}

fn checked_debug_info_end(offset: u32, size: u32, limit: u32) -> Result<u32, ParseError> {
    let end = offset
        .checked_add(size)
        .ok_or(ParseError::DebugInfoOutOfBounds {
            offset,
            size,
            limit,
        })?;
    if end > limit {
        return Err(ParseError::DebugInfoOutOfBounds {
            offset,
            size,
            limit,
        });
    }
    Ok(end)
}

fn debug_info_bytes(bytes: &[u8], offset: u32, size: u32, limit: u32) -> Result<&[u8], ParseError> {
    let end = checked_debug_info_end(offset, size, limit)?;
    let start = usize::try_from(offset).expect("u32 always fits in usize on supported targets");
    let end = usize::try_from(end).expect("u32 always fits in usize on supported targets");
    bytes
        .get(start..end)
        .ok_or(ParseError::DebugInfoOutOfBounds {
            offset,
            size,
            limit,
        })
}

fn validate_function_headers(
    bytes: &[u8],
    header: HermesBytecodeHeader,
    function_headers: SectionView<'_>,
    function_bodies_offset: u32,
    literal_value_buffer: SectionView<'_>,
    obj_shape_table: SectionView<'_>,
    debug_info: Option<DebugInfoBounds>,
) -> Result<(), ParseError> {
    let body_limit = function_body_limit(bytes, header, function_headers)?;
    let mut decoded_headers = Vec::with_capacity(
        usize::try_from(header.function_count)
            .expect("u32 always fits in usize on supported targets"),
    );
    for function_id in 0..header.function_count {
        decoded_headers.push(decoded_function_header_at(
            bytes,
            header,
            function_headers,
            function_id,
        )?);
    }

    for function_id in 0..header.function_count {
        let decoded = decoded_headers
            [usize::try_from(function_id).expect("u32 always fits in usize on supported targets")];
        let function_header = decoded.header;
        let function_region_limit = function_region_limit(function_header, &decoded_headers)
            .unwrap_or(body_limit)
            .min(body_limit);
        validate_function_body(
            bytes,
            function_id,
            function_header,
            function_bodies_offset,
            function_region_limit,
            InstructionValidationContext {
                header,
                function_region_limit,
                literal_value_buffer: literal_value_buffer.bytes,
                obj_shape_table: obj_shape_table.bytes,
            },
        )?;
        validate_function_info(
            bytes,
            function_id,
            decoded,
            function_info_region_limit(header, decoded.large_header_offset, &decoded_headers),
            debug_info,
        )?;
    }
    Ok(())
}

fn validate_function_body(
    bytes: &[u8],
    function_id: u32,
    function_header: HermesFunctionHeader,
    function_bodies_offset: u32,
    limit: u32,
    context: InstructionValidationContext<'_>,
) -> Result<(), ParseError> {
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
    validate_instruction_stream(
        bytes,
        function_id,
        function_body(bytes, function_id, function_header)?,
        context,
    )
}

fn validate_function_info(
    bytes: &[u8],
    function_id: u32,
    decoded: DecodedFunctionHeader,
    limit: u32,
    debug_info: Option<DebugInfoBounds>,
) -> Result<(), ParseError> {
    if !decoded.header.has_exception_handler() && !decoded.header.has_debug_info() {
        return Ok(());
    }
    let Some(info_offset) = decoded.large_header_offset else {
        return Err(ParseError::MissingFunctionInfo { function_id });
    };

    let mut cursor = info_offset
        .checked_add(LARGE_FUNC_HEADER_SIZE as u32)
        .and_then(|offset| align_u32(offset, FUNCTION_INFO_ALIGNMENT))
        .ok_or(ParseError::FunctionInfoOutOfBounds {
            function_id,
            offset: u32::MAX,
            size: EXCEPTION_HANDLER_TABLE_HEADER_SIZE as u32,
            limit,
        })?;

    if decoded.header.has_exception_handler() {
        let table_header = function_info_bytes(
            bytes,
            function_id,
            cursor,
            EXCEPTION_HANDLER_TABLE_HEADER_SIZE as u32,
            limit,
        )?;
        let entry_count = read_u32(table_header, 0);
        let table_offset = cursor
            .checked_add(EXCEPTION_HANDLER_TABLE_HEADER_SIZE as u32)
            .ok_or(ParseError::FunctionInfoOutOfBounds {
                function_id,
                offset: cursor,
                size: EXCEPTION_HANDLER_TABLE_HEADER_SIZE as u32,
                limit,
            })?;
        let table_size = entry_count
            .checked_mul(EXCEPTION_HANDLER_ENTRY_SIZE as u32)
            .ok_or(ParseError::FunctionInfoOutOfBounds {
                function_id,
                offset: table_offset,
                size: u32::MAX,
                limit,
            })?;
        let table_end =
            table_offset
                .checked_add(table_size)
                .ok_or(ParseError::FunctionInfoOutOfBounds {
                    function_id,
                    offset: table_offset,
                    size: table_size,
                    limit,
                })?;
        let table = function_info_bytes(bytes, function_id, table_offset, table_size, limit)?;
        let body = function_body(bytes, function_id, decoded.header)?;
        let boundaries = instruction_boundaries(function_id, body)?;

        for (entry_index, entry) in table.chunks_exact(EXCEPTION_HANDLER_ENTRY_SIZE).enumerate() {
            validate_exception_handler(
                function_id,
                body,
                &boundaries,
                u32::try_from(entry_index).expect("validated exception handler index fits in u32"),
                read_u32(entry, 0),
                read_u32(entry, 4),
                read_u32(entry, 8),
            )?;
        }
        cursor = table_end;
    }

    if decoded.header.has_debug_info() {
        let debug_offsets_offset = align_u32(cursor, FUNCTION_INFO_ALIGNMENT).ok_or(
            ParseError::FunctionInfoOutOfBounds {
                function_id,
                offset: cursor,
                size: DEBUG_OFFSETS_SIZE as u32,
                limit,
            },
        )?;
        let debug_offsets = function_info_bytes(
            bytes,
            function_id,
            debug_offsets_offset,
            DEBUG_OFFSETS_SIZE as u32,
            limit,
        )?;
        let body = function_body(bytes, function_id, decoded.header)?;
        let boundaries = instruction_boundaries(function_id, body)?;
        validate_debug_offsets(
            bytes,
            function_id,
            body,
            &boundaries,
            read_u32(debug_offsets, 0),
            debug_info,
        )?;
    }
    Ok(())
}

fn validate_debug_offsets(
    bytes: &[u8],
    function_id: u32,
    body: SectionView<'_>,
    boundaries: &[u32],
    source_locations: u32,
    debug_info: Option<DebugInfoBounds>,
) -> Result<(), ParseError> {
    if source_locations == DEBUG_OFFSETS_NO_OFFSET {
        return Ok(());
    }
    let Some(debug_info) = debug_info else {
        return Err(ParseError::InvalidDebugOffset {
            function_id,
            source_locations,
            debug_data_size: 0,
        });
    };
    let debug_data_size = debug_info.debug_data_size;
    if source_locations >= debug_data_size {
        return Err(ParseError::InvalidDebugOffset {
            function_id,
            source_locations,
            debug_data_size,
        });
    }

    let stream_offset = debug_info
        .debug_data_offset
        .checked_add(source_locations)
        .ok_or(ParseError::InvalidDebugData {
            function_id,
            offset: source_locations,
            reason: DEBUG_DATA_ADDRESS_OUT_OF_BOUNDS,
        })?;
    let stream_size =
        debug_data_size
            .checked_sub(source_locations)
            .ok_or(ParseError::InvalidDebugOffset {
                function_id,
                source_locations,
                debug_data_size,
            })?;
    let stream = debug_info_bytes(bytes, stream_offset, stream_size, u32::MAX).map_err(|_| {
        ParseError::InvalidDebugData {
            function_id,
            offset: source_locations,
            reason: DEBUG_DATA_TRUNCATED_LEB,
        }
    })?;
    validate_debug_source_locations(function_id, body, boundaries, source_locations, stream)
}

fn validate_debug_source_locations(
    function_id: u32,
    body: SectionView<'_>,
    boundaries: &[u32],
    source_locations: u32,
    data: &[u8],
) -> Result<(), ParseError> {
    let mut cursor = 0_usize;
    let stream_function_id = read_signed_leb128(data, &mut cursor, function_id, source_locations)?;
    if stream_function_id < 0 || u32::try_from(stream_function_id).ok() != Some(function_id) {
        return Err(ParseError::InvalidDebugData {
            function_id,
            offset: source_locations,
            reason: DEBUG_DATA_FUNCTION_MISMATCH,
        });
    }

    let mut current_line = read_debug_u32(data, &mut cursor, function_id, source_locations)?;
    let mut current_column = read_debug_u32(data, &mut cursor, function_id, source_locations)?;
    let mut current_statement = 0_u32;
    let mut current_env_idx = read_debug_u32(data, &mut cursor, function_id, source_locations)?;
    validate_debug_location_line_column(
        function_id,
        source_locations,
        current_line,
        current_column,
        true,
    )?;

    let mut current_address = 0_i64;
    loop {
        let address_delta_offset = debug_data_offset(source_locations, cursor);
        let address_delta = read_signed_leb128(data, &mut cursor, function_id, source_locations)?;
        if address_delta == -1 {
            return Ok(());
        }
        current_address =
            current_address
                .checked_add(address_delta)
                .ok_or(ParseError::InvalidDebugData {
                    function_id,
                    offset: address_delta_offset,
                    reason: DEBUG_DATA_ADDRESS_OUT_OF_BOUNDS,
                })?;
        if current_address < 0 || current_address >= i64::from(body.len()) {
            return Err(ParseError::InvalidDebugData {
                function_id,
                offset: address_delta_offset,
                reason: DEBUG_DATA_ADDRESS_OUT_OF_BOUNDS,
            });
        }
        let relative_address =
            u32::try_from(current_address).expect("validated non-negative address fits in u32");
        if !relative_instruction_boundary(body, relative_address, boundaries, false) {
            return Err(ParseError::InvalidDebugData {
                function_id,
                offset: address_delta_offset,
                reason: DEBUG_DATA_ADDRESS_NOT_BOUNDARY,
            });
        }

        let line_delta_offset = debug_data_offset(source_locations, cursor);
        let line_delta = read_signed_leb128(data, &mut cursor, function_id, source_locations)?;
        if line_delta & 1 == 0 {
            continue;
        }
        let column_delta_offset = debug_data_offset(source_locations, cursor);
        let column_delta = read_signed_leb128(data, &mut cursor, function_id, source_locations)?;
        let mut statement_delta = 0_i64;
        if line_delta & 2 != 0 {
            statement_delta = read_signed_leb128(data, &mut cursor, function_id, source_locations)?;
        }
        let mut env_idx_delta = 0_i64;
        if line_delta & 4 != 0 {
            env_idx_delta = read_signed_leb128(data, &mut cursor, function_id, source_locations)?;
        }
        current_line = add_debug_u32_delta(
            current_line,
            line_delta >> 3,
            function_id,
            line_delta_offset,
        )?;
        current_column = add_debug_u32_delta(
            current_column,
            column_delta,
            function_id,
            column_delta_offset,
        )?;
        current_statement = add_debug_u32_delta(
            current_statement,
            statement_delta,
            function_id,
            line_delta_offset,
        )?;
        current_env_idx = add_debug_u32_delta(
            current_env_idx,
            env_idx_delta,
            function_id,
            line_delta_offset,
        )?;
        validate_debug_location_line_column(
            function_id,
            line_delta_offset,
            current_line,
            current_column,
            false,
        )?;
    }
}

fn read_debug_u32(
    data: &[u8],
    cursor: &mut usize,
    function_id: u32,
    source_locations: u32,
) -> Result<u32, ParseError> {
    let start = *cursor;
    let value = read_signed_leb128(data, cursor, function_id, source_locations)?;
    u32::try_from(value).map_err(|_| ParseError::InvalidDebugData {
        function_id,
        offset: debug_data_offset(source_locations, start),
        reason: DEBUG_DATA_SOURCE_VALUE_OUT_OF_BOUNDS,
    })
}

fn add_debug_u32_delta(
    value: u32,
    delta: i64,
    function_id: u32,
    offset: u32,
) -> Result<u32, ParseError> {
    let value = i128::from(value) + i128::from(delta);
    u32::try_from(value).map_err(|_| ParseError::InvalidDebugData {
        function_id,
        offset,
        reason: DEBUG_DATA_SOURCE_VALUE_OUT_OF_BOUNDS,
    })
}

fn validate_debug_location_line_column(
    function_id: u32,
    offset: u32,
    line: u32,
    column: u32,
    allow_missing_location: bool,
) -> Result<(), ParseError> {
    if allow_missing_location && line == 0 && column == 0 {
        return Ok(());
    }
    if line == 0 || column == 0 {
        return Err(ParseError::InvalidDebugData {
            function_id,
            offset,
            reason: DEBUG_DATA_LOCATION_NOT_ONE_BASED,
        });
    }
    Ok(())
}

fn read_signed_leb128(
    data: &[u8],
    cursor: &mut usize,
    function_id: u32,
    source_locations: u32,
) -> Result<i64, ParseError> {
    let start = *cursor;
    let mut result = 0_i128;
    let mut shift = 0_u32;
    for _ in 0..10 {
        let Some(byte) = data.get(*cursor).copied() else {
            return Err(ParseError::InvalidDebugData {
                function_id,
                offset: debug_data_offset(source_locations, start),
                reason: DEBUG_DATA_TRUNCATED_LEB,
            });
        };
        *cursor += 1;
        result |= i128::from(byte & 0x7f) << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            if shift < 64 && byte & 0x40 != 0 {
                result |= (!0_i128) << shift;
            }
            return i64::try_from(result).map_err(|_| ParseError::InvalidDebugData {
                function_id,
                offset: debug_data_offset(source_locations, start),
                reason: DEBUG_DATA_LEB_OVERFLOW,
            });
        }
    }

    Err(ParseError::InvalidDebugData {
        function_id,
        offset: debug_data_offset(source_locations, start),
        reason: DEBUG_DATA_LEB_OVERFLOW,
    })
}

fn debug_data_offset(source_locations: u32, relative_offset: usize) -> u32 {
    source_locations.saturating_add(u32::try_from(relative_offset).unwrap_or(u32::MAX))
}

fn function_info_bytes(
    bytes: &[u8],
    function_id: u32,
    offset: u32,
    size: u32,
    limit: u32,
) -> Result<&[u8], ParseError> {
    let end = offset
        .checked_add(size)
        .ok_or(ParseError::FunctionInfoOutOfBounds {
            function_id,
            offset,
            size,
            limit,
        })?;
    if end > limit {
        return Err(ParseError::FunctionInfoOutOfBounds {
            function_id,
            offset,
            size,
            limit,
        });
    }

    let start = usize::try_from(offset).expect("u32 always fits in usize on supported targets");
    let end = usize::try_from(end).expect("u32 always fits in usize on supported targets");
    bytes
        .get(start..end)
        .ok_or(ParseError::FunctionInfoOutOfBounds {
            function_id,
            offset,
            size,
            limit,
        })
}

fn instruction_boundaries(function_id: u32, body: SectionView<'_>) -> Result<Vec<u32>, ParseError> {
    let mut boundaries = Vec::new();
    for instruction in HermesInstructionStream::new(function_id, body) {
        boundaries.push(instruction?.offset);
    }
    Ok(boundaries)
}

fn validate_exception_handler(
    function_id: u32,
    body: SectionView<'_>,
    boundaries: &[u32],
    entry_index: u32,
    start: u32,
    end: u32,
    target: u32,
) -> Result<(), ParseError> {
    let body_size = body.len();
    let valid = start <= end
        && relative_instruction_boundary(body, start, boundaries, false)
        && relative_instruction_boundary(body, end, boundaries, true)
        && relative_instruction_boundary(body, target, boundaries, false);
    if !valid {
        return Err(ParseError::InvalidExceptionHandler {
            function_id,
            entry_index,
            start,
            end,
            target,
            body_size,
        });
    }
    Ok(())
}

fn relative_instruction_boundary(
    body: SectionView<'_>,
    relative_offset: u32,
    boundaries: &[u32],
    allow_body_end: bool,
) -> bool {
    if allow_body_end && relative_offset == body.len() {
        return true;
    }
    let Some(target) = body.offset.checked_add(relative_offset) else {
        return false;
    };
    is_instruction_boundary(body, i64::from(target), boundaries)
}

fn function_region_limit(
    function_header: HermesFunctionHeader,
    function_headers: &[DecodedFunctionHeader],
) -> Option<u32> {
    function_headers
        .iter()
        .map(|decoded| decoded.header.offset)
        .filter(|offset| *offset > function_header.offset)
        .min()
}

fn function_info_region_limit(
    header: HermesBytecodeHeader,
    large_header_offset: Option<u32>,
    function_headers: &[DecodedFunctionHeader],
) -> u32 {
    let fallback = if header.debug_info_offset == 0 {
        header
            .file_length
            .saturating_sub(HERMES_BYTECODE_FOOTER_SIZE as u32)
    } else {
        header.debug_info_offset
    };
    let Some(large_header_offset) = large_header_offset else {
        return fallback;
    };
    function_headers
        .iter()
        .filter_map(|decoded| decoded.large_header_offset)
        .filter(|offset| *offset > large_header_offset)
        .min()
        .unwrap_or(fallback)
}

fn function_body_limit(
    bytes: &[u8],
    header: HermesBytecodeHeader,
    function_headers: SectionView<'_>,
) -> Result<u32, ParseError> {
    let fallback = if header.debug_info_offset == 0 {
        header
            .file_length
            .saturating_sub(HERMES_BYTECODE_FOOTER_SIZE as u32)
    } else {
        header.debug_info_offset
    };

    let mut limit = fallback;
    for function_id in 0..header.function_count {
        let decoded = decoded_function_header_at(bytes, header, function_headers, function_id)?;
        if let Some(large_header_offset) = decoded.large_header_offset {
            limit = limit.min(large_header_offset);
        }
    }
    Ok(limit)
}

fn function_body<'a>(
    bytes: &'a [u8],
    function_id: u32,
    function_header: HermesFunctionHeader,
) -> Result<SectionView<'a>, ParseError> {
    let start = usize::try_from(function_header.offset)
        .expect("u32 always fits in usize on supported targets");
    let size = usize::try_from(function_header.bytecode_size_in_bytes)
        .expect("u32 always fits in usize on supported targets");
    let end = start
        .checked_add(size)
        .ok_or(ParseError::FunctionBodyOutOfBounds {
            function_id,
            offset: function_header.offset,
            size: function_header.bytecode_size_in_bytes,
            function_bodies_offset: function_header.offset,
            limit: u32::MAX,
        })?;
    let body = bytes
        .get(start..end)
        .ok_or(ParseError::FunctionBodyOutOfBounds {
            function_id,
            offset: function_header.offset,
            size: function_header.bytecode_size_in_bytes,
            function_bodies_offset: 0,
            limit: u32::try_from(bytes.len()).unwrap_or(u32::MAX),
        })?;
    Ok(SectionView {
        offset: function_header.offset,
        bytes: body,
    })
}

fn validate_instruction_stream(
    bytes: &[u8],
    function_id: u32,
    body: SectionView<'_>,
    context: InstructionValidationContext<'_>,
) -> Result<(), ParseError> {
    let mut boundaries = Vec::new();
    let mut instructions = Vec::new();
    for instruction in HermesInstructionStream::new(function_id, body) {
        let instruction = instruction?;
        boundaries.push(instruction.offset);
        instructions.push(instruction);
    }

    for instruction in instructions {
        validate_instruction_table_operands(function_id, body, instruction, context.header)?;
        validate_instruction_literal_operands(function_id, body, instruction, context)?;
        validate_instruction_jump_operands(function_id, body, instruction, &boundaries)?;
        validate_switch_instruction(bytes, function_id, body, instruction, context, &boundaries)?;
    }
    Ok(())
}

fn validate_instruction_table_operands(
    function_id: u32,
    body: SectionView<'_>,
    instruction: HermesInstruction,
    header: HermesBytecodeHeader,
) -> Result<(), ParseError> {
    let instruction_bytes = instruction_bytes(body, instruction);
    for operand in BYTECODE_TABLE_OPERANDS
        .iter()
        .filter(|operand| operand.opcode == instruction.opcode)
    {
        let index = read_unsigned_operand(
            instruction_bytes,
            usize::from(operand.offset),
            usize::from(operand.width),
        );
        let limit = operand.table.limit(header);
        if index >= limit {
            return Err(ParseError::InvalidInstructionTableReference {
                function_id,
                offset: instruction.offset,
                opcode: instruction.opcode,
                table: operand.table.name(),
                index,
                limit,
            });
        }
    }
    Ok(())
}

fn validate_instruction_literal_operands(
    _function_id: u32,
    body: SectionView<'_>,
    instruction: HermesInstruction,
    context: InstructionValidationContext<'_>,
) -> Result<(), ParseError> {
    let instruction_bytes = instruction_bytes(body, instruction);
    for operand in OBJECT_LITERAL_OPERANDS
        .iter()
        .filter(|operand| operand.opcode == instruction.opcode)
    {
        let shape_index = read_unsigned_operand(
            instruction_bytes,
            usize::from(operand.shape_offset),
            usize::from(operand.shape_width),
        );
        let shape = object_shape_entry_at(context.obj_shape_table, shape_index)
            .expect("object shape table operand was validated before literal buffer parsing");
        let value_offset = read_unsigned_operand(
            instruction_bytes,
            usize::from(operand.value_offset),
            usize::from(operand.value_width),
        );
        validate_serialized_literal_buffer(
            "literal_value_buffer",
            context.literal_value_buffer,
            value_offset,
            shape.num_props,
            false,
            context.header,
        )?;
    }

    for operand in ARRAY_LITERAL_OPERANDS
        .iter()
        .filter(|operand| operand.opcode == instruction.opcode)
    {
        let element_count = read_unsigned_operand(
            instruction_bytes,
            usize::from(operand.element_count_offset),
            usize::from(operand.element_count_width),
        );
        let value_offset = read_unsigned_operand(
            instruction_bytes,
            usize::from(operand.value_offset),
            usize::from(operand.value_width),
        );
        validate_serialized_literal_buffer(
            "literal_value_buffer",
            context.literal_value_buffer,
            value_offset,
            element_count,
            false,
            context.header,
        )?;
    }
    Ok(())
}

fn validate_instruction_jump_operands(
    function_id: u32,
    body: SectionView<'_>,
    instruction: HermesInstruction,
    boundaries: &[u32],
) -> Result<(), ParseError> {
    let instruction_bytes = instruction_bytes(body, instruction);
    for operand in JUMP_OPERANDS
        .iter()
        .filter(|operand| operand.opcode == instruction.opcode)
    {
        let delta = read_signed_operand(
            instruction_bytes,
            usize::from(operand.offset),
            usize::from(operand.width),
        );
        validate_jump_target(function_id, body, instruction, delta, boundaries)?;
    }
    Ok(())
}

fn validate_switch_instruction(
    bytes: &[u8],
    function_id: u32,
    body: SectionView<'_>,
    instruction: HermesInstruction,
    context: InstructionValidationContext<'_>,
    boundaries: &[u32],
) -> Result<(), ParseError> {
    match instruction.opcode {
        UINT_SWITCH_IMM_OPCODE => validate_uint_switch_instruction(
            bytes,
            function_id,
            body,
            instruction,
            context,
            boundaries,
        ),
        STRING_SWITCH_IMM_OPCODE => validate_string_switch_instruction(
            bytes,
            function_id,
            body,
            instruction,
            context,
            boundaries,
        ),
        _ => Ok(()),
    }
}

fn validate_uint_switch_instruction(
    bytes: &[u8],
    function_id: u32,
    body: SectionView<'_>,
    instruction: HermesInstruction,
    context: InstructionValidationContext<'_>,
    boundaries: &[u32],
) -> Result<(), ParseError> {
    let instruction_bytes = instruction_bytes(body, instruction);
    let table_offset = read_unsigned_operand(instruction_bytes, 2, 4);
    let default_delta = read_signed_operand(instruction_bytes, 6, 4);
    let minimum = read_unsigned_operand(instruction_bytes, 10, 4);
    let maximum = read_unsigned_operand(instruction_bytes, 14, 4);

    validate_jump_target(function_id, body, instruction, default_delta, boundaries)?;
    if maximum < minimum {
        return Err(ParseError::InvalidSwitchTableRange {
            function_id,
            offset: instruction.offset,
            opcode: instruction.opcode,
            minimum,
            maximum,
        });
    }

    let entry_count = maximum
        .checked_sub(minimum)
        .and_then(|span| span.checked_add(1))
        .ok_or(ParseError::InvalidSwitchTableRange {
            function_id,
            offset: instruction.offset,
            opcode: instruction.opcode,
            minimum,
            maximum,
        })?;
    let table_bytes = checked_table_size(entry_count, size_of::<u32>() as u32);
    let table = switch_table_bytes(
        bytes,
        function_id,
        body,
        instruction,
        table_offset,
        table_bytes,
        context.function_region_limit,
    )?;

    for (entry_index, entry) in table.chunks_exact(size_of::<u32>()).enumerate() {
        let delta = i64::from(i32::from_le_bytes(
            entry
                .try_into()
                .expect("switch table entry is exactly u32 sized"),
        ));
        validate_switch_table_target(
            function_id,
            body,
            instruction,
            u32::try_from(entry_index).expect("switch table entry index fits in u32"),
            delta,
            boundaries,
        )?;
    }
    Ok(())
}

fn validate_string_switch_instruction(
    bytes: &[u8],
    function_id: u32,
    body: SectionView<'_>,
    instruction: HermesInstruction,
    context: InstructionValidationContext<'_>,
    boundaries: &[u32],
) -> Result<(), ParseError> {
    let instruction_bytes = instruction_bytes(body, instruction);
    let switch_index = read_unsigned_operand(instruction_bytes, 2, 4);
    let table_offset = read_unsigned_operand(instruction_bytes, 6, 4);
    let default_delta = read_signed_operand(instruction_bytes, 10, 4);
    let entry_count = read_unsigned_operand(instruction_bytes, 14, 4);

    if switch_index >= context.header.num_string_switch_imms {
        return Err(ParseError::InvalidInstructionTableReference {
            function_id,
            offset: instruction.offset,
            opcode: instruction.opcode,
            table: "string_switch",
            index: switch_index,
            limit: context.header.num_string_switch_imms,
        });
    }

    validate_jump_target(function_id, body, instruction, default_delta, boundaries)?;
    let table_bytes = checked_table_size(entry_count, SWITCH_TABLE_CASE_SIZE as u32);
    let table = switch_table_bytes(
        bytes,
        function_id,
        body,
        instruction,
        table_offset,
        table_bytes,
        context.function_region_limit,
    )?;

    for (entry_index, entry) in table.chunks_exact(SWITCH_TABLE_CASE_SIZE).enumerate() {
        let string_id = read_u32(entry, 0);
        if string_id >= context.header.string_count {
            return Err(ParseError::InvalidInstructionTableReference {
                function_id,
                offset: instruction.offset,
                opcode: instruction.opcode,
                table: "string",
                index: string_id,
                limit: context.header.string_count,
            });
        }
        let delta = i64::from(i32::from_le_bytes(
            entry[4..8]
                .try_into()
                .expect("string switch target entry is exactly i32 sized"),
        ));
        validate_switch_table_target(
            function_id,
            body,
            instruction,
            u32::try_from(entry_index).expect("switch table entry index fits in u32"),
            delta,
            boundaries,
        )?;
    }
    Ok(())
}

fn switch_table_bytes<'a>(
    bytes: &'a [u8],
    function_id: u32,
    body: SectionView<'_>,
    instruction: HermesInstruction,
    table_offset: u32,
    table_size: u32,
    region_limit: u32,
) -> Result<&'a [u8], ParseError> {
    let table_start = instruction
        .offset
        .checked_add(table_offset)
        .and_then(|offset| align_u32(offset, SWITCH_TABLE_ALIGNMENT))
        .ok_or(ParseError::SwitchTableOutOfBounds {
            function_id,
            offset: instruction.offset,
            opcode: instruction.opcode,
            table_start: u32::MAX,
            table_size,
            region_limit,
        })?;
    let table_end =
        table_start
            .checked_add(table_size)
            .ok_or(ParseError::SwitchTableOutOfBounds {
                function_id,
                offset: instruction.offset,
                opcode: instruction.opcode,
                table_start,
                table_size,
                region_limit,
            })?;

    if table_start < body.offset + body.len() || table_end > region_limit {
        return Err(ParseError::SwitchTableOutOfBounds {
            function_id,
            offset: instruction.offset,
            opcode: instruction.opcode,
            table_start,
            table_size,
            region_limit,
        });
    }

    let start =
        usize::try_from(table_start).expect("u32 always fits in usize on supported targets");
    let end = usize::try_from(table_end).expect("u32 always fits in usize on supported targets");
    bytes
        .get(start..end)
        .ok_or(ParseError::SwitchTableOutOfBounds {
            function_id,
            offset: instruction.offset,
            opcode: instruction.opcode,
            table_start,
            table_size,
            region_limit,
        })
}

fn validate_jump_target(
    function_id: u32,
    body: SectionView<'_>,
    instruction: HermesInstruction,
    delta: i64,
    boundaries: &[u32],
) -> Result<(), ParseError> {
    let target = i64::from(instruction.offset) + delta;
    if !is_instruction_boundary(body, target, boundaries) {
        return Err(ParseError::InvalidJumpTarget {
            function_id,
            offset: instruction.offset,
            opcode: instruction.opcode,
            target,
            body_start: body.offset,
            body_end: body.offset + body.len(),
        });
    }
    Ok(())
}

fn validate_switch_table_target(
    function_id: u32,
    body: SectionView<'_>,
    instruction: HermesInstruction,
    entry_index: u32,
    delta: i64,
    boundaries: &[u32],
) -> Result<(), ParseError> {
    let target = i64::from(instruction.offset) + delta;
    if !is_instruction_boundary(body, target, boundaries) {
        return Err(ParseError::InvalidSwitchTableTarget {
            function_id,
            offset: instruction.offset,
            opcode: instruction.opcode,
            entry_index,
            target,
            body_start: body.offset,
            body_end: body.offset + body.len(),
        });
    }
    Ok(())
}

fn is_instruction_boundary(body: SectionView<'_>, target: i64, boundaries: &[u32]) -> bool {
    if target < i64::from(body.offset) || target >= i64::from(body.offset + body.len()) {
        return false;
    }
    u32::try_from(target)
        .ok()
        .is_some_and(|target| boundaries.binary_search(&target).is_ok())
}

fn instruction_bytes(body: SectionView<'_>, instruction: HermesInstruction) -> &[u8] {
    let start = usize::try_from(instruction.offset - body.offset)
        .expect("instruction offset was validated inside the function body");
    let end = start + usize::from(instruction.width);
    &body.bytes[start..end]
}

fn read_unsigned_operand(bytes: &[u8], offset: usize, width: usize) -> u32 {
    match width {
        1 => u32::from(bytes[offset]),
        2 => u32::from(u16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .expect("validated instruction operand has u16 width"),
        )),
        4 => read_u32(bytes, offset),
        _ => unreachable!("Hermes integer operand width is 1, 2, or 4 bytes"),
    }
}

fn read_signed_operand(bytes: &[u8], offset: usize, width: usize) -> i64 {
    match width {
        1 => i64::from(i8::from_le_bytes([bytes[offset]])),
        4 => i64::from(i32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("validated instruction operand has i32 width"),
        )),
        _ => unreachable!("Hermes jump operand width is 1 or 4 bytes"),
    }
}

fn checked_table_size(entry_count: u32, entry_size: u32) -> u32 {
    entry_count.checked_mul(entry_size).unwrap_or(u32::MAX)
}

fn align_u32(offset: u32, alignment: u32) -> Option<u32> {
    offset
        .checked_add(alignment - 1)
        .map(|adjusted| adjusted / alignment * alignment)
}

fn count_function_instructions(function_id: u32, body: SectionView<'_>) -> Result<u32, ParseError> {
    let mut count = 0_u32;
    for instruction in HermesInstructionStream::new(function_id, body) {
        instruction?;
        count = count
            .checked_add(1)
            .expect("instruction count cannot exceed u32 because HBC file length is u32");
    }
    Ok(count)
}

fn hermes_opcode_width(opcode: u8) -> Option<u8> {
    HERMES_OPCODE_WIDTHS.get(usize::from(opcode)).copied()
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

    const NEW_OBJECT_WITH_BUFFER_OPCODE: u8 = 1;
    const NEW_ARRAY_WITH_BUFFER_OPCODE: u8 = 7;
    const RET_OPCODE: u8 = 118;
    const LOAD_CONST_DOUBLE_OPCODE: u8 = 141;
    const LOAD_CONST_BIGINT_OPCODE: u8 = 142;
    const LOAD_CONST_STRING_OPCODE: u8 = 144;
    const CREATE_CLOSURE_OPCODE: u8 = 132;
    const JMP_OPCODE: u8 = 175;

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

    fn encode_large_function_header(
        header: SmallFunctionHeaderFixture,
    ) -> [u8; LARGE_FUNC_HEADER_SIZE] {
        let mut bytes = [0; LARGE_FUNC_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&header.offset.to_le_bytes());
        bytes[4..8].copy_from_slice(&header.param_count.to_le_bytes());
        bytes[8..12].copy_from_slice(&header.loop_depth.to_le_bytes());
        bytes[12..16].copy_from_slice(&header.bytecode_size_in_bytes.to_le_bytes());
        bytes[16..20].copy_from_slice(&header.function_name.to_le_bytes());
        bytes[20..24].copy_from_slice(&header.number_reg_count.to_le_bytes());
        bytes[24..28].copy_from_slice(&header.non_ptr_reg_count.to_le_bytes());
        bytes[28..32].copy_from_slice(&u32::from(header.frame_size).to_le_bytes());
        bytes[32] = header.read_cache_size;
        bytes[33] = header.write_cache_size;
        bytes[34] = header.private_name_cache_size;
        bytes[35] = header.flags;
        bytes
    }

    fn fixture_header() -> Vec<u8> {
        let mut bytes = vec![0; HERMES_BYTECODE_HEADER_SIZE];
        bytes[MAGIC_OFFSET..MAGIC_OFFSET + size_of::<u64>()]
            .copy_from_slice(&HERMES_BYTECODE_MAGIC.to_le_bytes());
        write_u32(&mut bytes, VERSION_OFFSET, 99);
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
        write_u32(&mut bytes, VERSION_OFFSET, 99);
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
        append_bytes(
            &mut bytes,
            &[0_u32.to_le_bytes(), 1_u32.to_le_bytes()].concat(),
        );
        append_bytes(
            &mut bytes,
            &[0_u32.to_le_bytes(), 4_u32.to_le_bytes()].concat(),
        );
        append_bytes(&mut bytes, &[9, 10, 11, 12]);
        append_bytes(
            &mut bytes,
            &[0_u32.to_le_bytes(), 2_u32.to_le_bytes()].concat(),
        );
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
        bytes.extend_from_slice(&[RET_OPCODE, 0, RET_OPCODE, 0]);
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

    fn replace_global_body(bytes: &mut Vec<u8>, body: &[u8]) -> u32 {
        replace_global_body_with_payload(bytes, body, &[]).0
    }

    fn replace_global_body_with_payload(
        bytes: &mut Vec<u8>,
        body: &[u8],
        payload: &[u8],
    ) -> (u32, u32) {
        let bytecode = HermesBytecode::parse(bytes).expect("valid fixture");
        let header = bytecode.header();
        let sections = bytecode.sections();
        let global = bytecode
            .global_function_header()
            .expect("valid global function");
        let body_offset =
            usize::try_from(global.offset).expect("test fixture offset fits in usize");
        let old_debug_offset =
            usize::try_from(header.debug_info_offset).expect("test fixture offset fits in usize");
        let function_header_offset = sections.function_headers().offset() as usize
            + usize::try_from(header.global_code_index)
                .expect("test fixture global index fits in usize")
                * SMALL_FUNC_HEADER_SIZE;

        let mut replacement = Vec::with_capacity(body.len() + payload.len() + 4);
        replacement.extend_from_slice(body);
        replacement.extend_from_slice(payload);
        while !(body_offset + replacement.len()).is_multiple_of(HERMES_BYTECODE_ALIGNMENT) {
            replacement.push(0);
        }

        bytes.splice(body_offset..old_debug_offset, replacement);
        let new_debug_offset = u32::try_from(body_offset + body.len() + payload.len())
            .expect("test fixture offset fits in u32");
        let new_debug_offset_aligned =
            align_u32(new_debug_offset, HERMES_BYTECODE_ALIGNMENT as u32)
                .expect("test fixture aligned offset fits in u32");
        let new_file_length = u32::try_from(bytes.len()).expect("test fixture length fits in u32");
        write_u32(bytes, DEBUG_INFO_OFFSET_OFFSET, new_debug_offset_aligned);
        write_u32(bytes, FILE_LENGTH_OFFSET, new_file_length);
        bytes[function_header_offset..function_header_offset + SMALL_FUNC_HEADER_SIZE]
            .copy_from_slice(&encode_small_function_header(SmallFunctionHeaderFixture {
                offset: global.offset,
                bytecode_size_in_bytes: u32::try_from(body.len())
                    .expect("test fixture body length fits in u32"),
                param_count: global.param_count,
                loop_depth: global.loop_depth,
                function_name: global.function_name,
                number_reg_count: global.number_reg_count,
                non_ptr_reg_count: global.non_ptr_reg_count,
                frame_size: u8::try_from(global.frame_size)
                    .expect("test fixture frame size fits in u8"),
                read_cache_size: global.read_cache_size,
                write_cache_size: global.write_cache_size,
                private_name_cache_size: global.private_name_cache_size,
                flags: global.flags,
            }));

        (
            u32::try_from(body_offset).expect("test fixture offset fits in u32"),
            new_debug_offset_aligned,
        )
    }

    fn add_global_exception_info(
        bytes: &mut Vec<u8>,
        declared_count: u32,
        entries: &[(u32, u32, u32)],
    ) -> u32 {
        add_global_function_info(bytes, Some((declared_count, entries)), None)
    }

    fn add_global_debug_offsets(bytes: &mut Vec<u8>, source_locations: u32) -> u32 {
        add_global_function_info(bytes, None, Some(source_locations))
    }

    fn set_debug_data(bytes: &mut Vec<u8>, debug_data: &[u8]) {
        set_debug_info(bytes, &[], &[], &[], debug_data);
    }

    fn set_debug_info(
        bytes: &mut Vec<u8>,
        filename_entries: &[(u32, u32, bool)],
        filename_storage: &[u8],
        file_regions: &[(u32, u32, u32)],
        debug_data: &[u8],
    ) {
        let (debug_info_offset, file_length) = {
            let bytecode = HermesBytecode::parse(bytes).expect("valid fixture");
            (
                bytecode.header().debug_info_offset,
                bytecode.header().file_length,
            )
        };
        let debug_info_offset =
            usize::try_from(debug_info_offset).expect("test fixture offset fits in usize");
        let footer_offset = usize::try_from(file_length)
            .expect("test fixture length fits in usize")
            - HERMES_BYTECODE_FOOTER_SIZE;

        let mut debug_info = Vec::new();
        debug_info.extend_from_slice(
            &u32::try_from(filename_entries.len())
                .expect("test fixture filename count fits in u32")
                .to_le_bytes(),
        );
        debug_info.extend_from_slice(
            &u32::try_from(filename_storage.len())
                .expect("test fixture filename storage length fits in u32")
                .to_le_bytes(),
        );
        debug_info.extend_from_slice(
            &u32::try_from(file_regions.len())
                .expect("test fixture file region count fits in u32")
                .to_le_bytes(),
        );
        debug_info.extend_from_slice(
            &u32::try_from(debug_data.len())
                .expect("test fixture debug data length fits in u32")
                .to_le_bytes(),
        );
        for (offset, length, is_utf16) in filename_entries {
            debug_info.extend_from_slice(&offset.to_le_bytes());
            let raw_length = if *is_utf16 {
                length | STRING_TABLE_ENTRY_UTF16_MASK
            } else {
                *length
            };
            debug_info.extend_from_slice(&raw_length.to_le_bytes());
        }
        debug_info.extend_from_slice(filename_storage);
        for (from_address, filename_id, source_mapping_url_id) in file_regions {
            debug_info.extend_from_slice(&from_address.to_le_bytes());
            debug_info.extend_from_slice(&filename_id.to_le_bytes());
            debug_info.extend_from_slice(&source_mapping_url_id.to_le_bytes());
        }
        debug_info.extend_from_slice(debug_data);

        bytes.splice(debug_info_offset..footer_offset, debug_info);
        let new_file_length = u32::try_from(bytes.len()).expect("test fixture length fits in u32");
        write_u32(bytes, FILE_LENGTH_OFFSET, new_file_length);
    }

    fn debug_source_stream(function_id: u32, address_delta: i64) -> Vec<u8> {
        let mut data = Vec::new();
        append_signed_leb128(&mut data, i64::from(function_id));
        append_signed_leb128(&mut data, 1);
        append_signed_leb128(&mut data, 1);
        append_signed_leb128(&mut data, 0);
        append_signed_leb128(&mut data, address_delta);
        append_signed_leb128(&mut data, 0);
        append_signed_leb128(&mut data, -1);
        data
    }

    fn debug_source_stream_with_location_delta(
        function_id: u32,
        address_delta: i64,
        line_delta: i64,
        column_delta: i64,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        append_signed_leb128(&mut data, i64::from(function_id));
        append_signed_leb128(&mut data, 1);
        append_signed_leb128(&mut data, 1);
        append_signed_leb128(&mut data, 0);
        append_signed_leb128(&mut data, address_delta);
        append_signed_leb128(&mut data, (line_delta << 3) | 1);
        append_signed_leb128(&mut data, column_delta);
        append_signed_leb128(&mut data, -1);
        data
    }

    fn debug_source_stream_without_sentinel(function_id: u32) -> Vec<u8> {
        let mut data = Vec::new();
        append_signed_leb128(&mut data, i64::from(function_id));
        append_signed_leb128(&mut data, 1);
        append_signed_leb128(&mut data, 1);
        append_signed_leb128(&mut data, 0);
        data
    }

    fn append_signed_leb128(bytes: &mut Vec<u8>, mut value: i64) {
        loop {
            let byte = u8::try_from(value & 0x7f).expect("SLEB low bits fit in u8");
            value >>= 7;
            let done = (value == 0 && byte & 0x40 == 0) || (value == -1 && byte & 0x40 != 0);
            if done {
                bytes.push(byte);
                return;
            }
            bytes.push(byte | 0x80);
        }
    }

    fn add_global_function_info(
        bytes: &mut Vec<u8>,
        exception_table: Option<(u32, &[(u32, u32, u32)])>,
        debug_source_locations: Option<u32>,
    ) -> u32 {
        let (header, function_headers_offset, global) = {
            let bytecode = HermesBytecode::parse(bytes).expect("valid fixture");
            (
                bytecode.header(),
                bytecode.sections().function_headers().offset() as usize,
                bytecode
                    .global_function_header()
                    .expect("valid global function"),
            )
        };
        let info_offset =
            usize::try_from(header.debug_info_offset).expect("test fixture offset fits in usize");
        let function_header_offset = function_headers_offset
            + usize::try_from(header.global_code_index)
                .expect("test fixture global index fits in usize")
                * SMALL_FUNC_HEADER_SIZE;

        let mut info = Vec::new();
        let mut flags = global.flags;
        if exception_table.is_some() {
            flags |= 0b0000_1000;
        }
        if debug_source_locations.is_some() {
            flags |= 0b0001_0000;
        }
        info.extend_from_slice(&encode_large_function_header(SmallFunctionHeaderFixture {
            offset: global.offset,
            bytecode_size_in_bytes: global.bytecode_size_in_bytes,
            param_count: global.param_count,
            loop_depth: global.loop_depth,
            function_name: global.function_name,
            number_reg_count: global.number_reg_count,
            non_ptr_reg_count: global.non_ptr_reg_count,
            frame_size: u8::try_from(global.frame_size)
                .expect("test fixture frame size fits in u8"),
            read_cache_size: global.read_cache_size,
            write_cache_size: global.write_cache_size,
            private_name_cache_size: global.private_name_cache_size,
            flags,
        }));
        if let Some((declared_count, entries)) = exception_table {
            while !(info_offset + info.len()).is_multiple_of(
                usize::try_from(FUNCTION_INFO_ALIGNMENT)
                    .expect("test fixture alignment fits in usize"),
            ) {
                info.push(0);
            }
            info.extend_from_slice(&declared_count.to_le_bytes());
            for (start, end, target) in entries {
                info.extend_from_slice(&start.to_le_bytes());
                info.extend_from_slice(&end.to_le_bytes());
                info.extend_from_slice(&target.to_le_bytes());
            }
        }
        if let Some(source_locations) = debug_source_locations {
            while !(info_offset + info.len()).is_multiple_of(
                usize::try_from(FUNCTION_INFO_ALIGNMENT)
                    .expect("test fixture alignment fits in usize"),
            ) {
                info.push(0);
            }
            info.extend_from_slice(&source_locations.to_le_bytes());
        }

        let info_size = u32::try_from(info.len()).expect("test fixture info length fits in u32");
        bytes.splice(info_offset..info_offset, info);
        let info_offset_u32 = u32::try_from(info_offset).expect("test fixture offset fits in u32");
        let new_debug_offset = header.debug_info_offset + info_size;
        let new_file_length = u32::try_from(bytes.len()).expect("test fixture length fits in u32");
        write_u32(bytes, DEBUG_INFO_OFFSET_OFFSET, new_debug_offset);
        write_u32(bytes, FILE_LENGTH_OFFSET, new_file_length);
        bytes[function_header_offset..function_header_offset + SMALL_FUNC_HEADER_SIZE]
            .copy_from_slice(&encode_overflow_function_header(info_offset_u32));
        info_offset_u32
    }

    #[test]
    fn parses_header_without_copying_payload() {
        let bytes = fixture_bytecode();
        let bytecode = HermesBytecode::parse(&bytes).expect("valid Hermes bytecode header");
        let header = bytecode.header();

        assert!(std::ptr::eq(bytecode.bytes().as_ptr(), bytes.as_ptr()));
        assert_eq!(header.version, 99);
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

        assert_eq!(metadata.version, 99);
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
        assert_eq!(metadata.global_instruction_count, 1);
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
    fn iterates_function_instructions() {
        let bytes = fixture_bytecode();
        let bytecode = HermesBytecode::parse(&bytes).expect("valid Hermes bytecode");
        let global_body = bytecode
            .global_function_body()
            .expect("valid global function body");

        assert_eq!(global_body.bytes(), &[RET_OPCODE, 0]);
        assert_eq!(bytecode.global_instruction_count(), Ok(1));

        let instructions = bytecode
            .function_instructions(1)
            .expect("valid instruction stream")
            .collect::<Result<Vec<_>, _>>()
            .expect("valid instructions");
        assert_eq!(
            instructions,
            vec![HermesInstruction {
                offset: bytecode
                    .global_function_header()
                    .expect("global header")
                    .offset,
                opcode: RET_OPCODE,
                width: 2,
            }],
        );
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
    fn rejects_object_shape_entries_outside_key_buffer() {
        let mut bytes = fixture_bytecode();
        let shape_offset = HermesBytecode::parse(&bytes)
            .expect("valid fixture")
            .sections()
            .obj_shape_table()
            .offset() as usize;
        write_u32(&mut bytes, shape_offset, 99);
        write_u32(&mut bytes, shape_offset + 4, 1);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid shape entry must fail");
        assert_eq!(
            error,
            ParseError::InvalidSerializedLiteralBuffer {
                buffer: "obj_key_buffer",
                offset: 99,
                element_count: 1,
                buffer_size: 5,
            },
        );
    }

    #[test]
    fn rejects_invalid_object_literal_shape_reference() {
        let mut bytes = fixture_bytecode();
        let function_body_offset =
            replace_global_body(&mut bytes, &[NEW_OBJECT_WITH_BUFFER_OPCODE, 0, 99, 0, 0, 0]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid object shape must fail");
        assert_eq!(
            error,
            ParseError::InvalidInstructionTableReference {
                function_id: 1,
                offset: function_body_offset,
                opcode: NEW_OBJECT_WITH_BUFFER_OPCODE,
                table: "object_shape",
                index: 99,
                limit: 1,
            },
        );
    }

    #[test]
    fn rejects_array_literal_value_buffer_outside_storage() {
        let mut bytes = fixture_bytecode();
        replace_global_body(
            &mut bytes,
            &[NEW_ARRAY_WITH_BUFFER_OPCODE, 0, 0, 0, 1, 0, 99, 0],
        );

        let error = HermesBytecode::parse(&bytes).expect_err("invalid array buffer must fail");
        assert_eq!(
            error,
            ParseError::InvalidSerializedLiteralBuffer {
                buffer: "literal_value_buffer",
                offset: 99,
                element_count: 1,
                buffer_size: 3,
            },
        );
    }

    #[test]
    fn accepts_valid_object_literal_buffer_reference() {
        let mut bytes = fixture_bytecode();
        replace_global_body(&mut bytes, &[NEW_OBJECT_WITH_BUFFER_OPCODE, 0, 0, 0, 0, 0]);

        let bytecode = HermesBytecode::parse(&bytes).expect("valid object literal buffer");
        assert_eq!(bytecode.global_instruction_count(), Ok(1));
    }

    #[test]
    fn rejects_bigint_entries_outside_storage() {
        let mut bytes = fixture_bytecode();
        let big_int_offset = HermesBytecode::parse(&bytes)
            .expect("valid fixture")
            .sections()
            .big_int_table()
            .offset() as usize;
        write_u32(&mut bytes, big_int_offset, 2);
        write_u32(&mut bytes, big_int_offset + 4, 3);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid bigint entry must fail");
        assert_eq!(
            error,
            ParseError::StorageTableEntryOutOfBounds {
                section: "big_int_table",
                entry_index: 0,
                offset: 2,
                length: 3,
                storage_size: 4,
            },
        );
    }

    #[test]
    fn rejects_regexp_entries_outside_storage() {
        let mut bytes = fixture_bytecode();
        let reg_exp_offset = HermesBytecode::parse(&bytes)
            .expect("valid fixture")
            .sections()
            .reg_exp_table()
            .offset() as usize;
        write_u32(&mut bytes, reg_exp_offset, 1);
        write_u32(&mut bytes, reg_exp_offset + 4, 2);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid regexp entry must fail");
        assert_eq!(
            error,
            ParseError::StorageTableEntryOutOfBounds {
                section: "reg_exp_table",
                entry_index: 0,
                offset: 1,
                length: 2,
                storage_size: 2,
            },
        );
    }

    #[test]
    fn rejects_function_body_outside_function_region() {
        let mut bytes = fixture_bytecode();
        let (function_headers_offset, function_bodies_offset) = {
            let bytecode = HermesBytecode::parse(&bytes).expect("valid fixture");
            (
                bytecode.sections().function_headers().offset() as usize,
                bytecode.sections().function_bodies_offset(),
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
                limit: function_bodies_offset + 2,
            },
        );
    }

    #[test]
    fn rejects_function_body_that_overlaps_function_info() {
        let mut bytes = fixture_bytecode();
        let (function_headers_offset, function_bodies_offset, debug_info_offset) = {
            let bytecode = HermesBytecode::parse(&bytes).expect("valid fixture");
            (
                bytecode.sections().function_headers().offset() as usize,
                bytecode.sections().function_bodies_offset(),
                bytecode.header().debug_info_offset,
            )
        };
        let large_header_offset = debug_info_offset;
        let insert_offset =
            usize::try_from(large_header_offset).expect("test fixture offset fits in usize");
        bytes.splice(insert_offset..insert_offset, [0; LARGE_FUNC_HEADER_SIZE]);
        let new_debug_info_offset = debug_info_offset + LARGE_FUNC_HEADER_SIZE as u32;
        let new_file_length = u32::try_from(bytes.len()).expect("test fixture length fits in u32");
        write_u32(&mut bytes, DEBUG_INFO_OFFSET_OFFSET, new_debug_info_offset);
        write_u32(&mut bytes, FILE_LENGTH_OFFSET, new_file_length);
        bytes[insert_offset..insert_offset + LARGE_FUNC_HEADER_SIZE].copy_from_slice(
            &encode_large_function_header(SmallFunctionHeaderFixture {
                offset: function_bodies_offset,
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
            }),
        );
        bytes[function_headers_offset..function_headers_offset + SMALL_FUNC_HEADER_SIZE]
            .copy_from_slice(&encode_overflow_function_header(large_header_offset));
        bytes[function_headers_offset + SMALL_FUNC_HEADER_SIZE
            ..function_headers_offset + SMALL_FUNC_HEADER_SIZE * 2]
            .copy_from_slice(&encode_small_function_header(SmallFunctionHeaderFixture {
                offset: function_bodies_offset + 2,
                bytecode_size_in_bytes: LARGE_FUNC_HEADER_SIZE as u32,
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

        let error = HermesBytecode::parse(&bytes)
            .expect_err("function body overlapping function info must fail");
        assert_eq!(
            error,
            ParseError::FunctionBodyOutOfBounds {
                function_id: 1,
                offset: function_bodies_offset + 2,
                size: LARGE_FUNC_HEADER_SIZE as u32,
                function_bodies_offset,
                limit: large_header_offset,
            },
        );
    }

    #[test]
    fn accepts_valid_exception_handler_table() {
        let mut bytes = fixture_bytecode();
        add_global_exception_info(&mut bytes, 1, &[(0, 2, 0)]);

        let bytecode = HermesBytecode::parse(&bytes).expect("valid exception table");
        assert!(
            bytecode
                .global_function_header()
                .expect("valid global function")
                .has_exception_handler()
        );
    }

    #[test]
    fn rejects_exception_handler_target_between_instructions() {
        let mut bytes = fixture_bytecode();
        add_global_exception_info(&mut bytes, 1, &[(0, 2, 1)]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid exception target must fail");
        assert_eq!(
            error,
            ParseError::InvalidExceptionHandler {
                function_id: 1,
                entry_index: 0,
                start: 0,
                end: 2,
                target: 1,
                body_size: 2,
            },
        );
    }

    #[test]
    fn rejects_exception_table_outside_function_info() {
        let mut bytes = fixture_bytecode();
        let info_offset = add_global_exception_info(&mut bytes, 2, &[(0, 2, 0)]);

        let error = HermesBytecode::parse(&bytes).expect_err("short exception table must fail");
        assert_eq!(
            error,
            ParseError::FunctionInfoOutOfBounds {
                function_id: 1,
                offset: info_offset
                    + LARGE_FUNC_HEADER_SIZE as u32
                    + EXCEPTION_HANDLER_TABLE_HEADER_SIZE as u32,
                size: 2 * EXCEPTION_HANDLER_ENTRY_SIZE as u32,
                limit: info_offset
                    + LARGE_FUNC_HEADER_SIZE as u32
                    + EXCEPTION_HANDLER_TABLE_HEADER_SIZE as u32
                    + EXCEPTION_HANDLER_ENTRY_SIZE as u32,
            },
        );
    }

    #[test]
    fn accepts_debug_offsets_without_source_locations() {
        let mut bytes = fixture_bytecode();
        add_global_debug_offsets(&mut bytes, DEBUG_OFFSETS_NO_OFFSET);

        let bytecode = HermesBytecode::parse(&bytes).expect("valid debug offsets");
        assert!(
            bytecode
                .global_function_header()
                .expect("valid global function")
                .has_debug_info()
        );
    }

    #[test]
    fn accepts_valid_debug_source_locations() {
        let mut bytes = fixture_bytecode();
        set_debug_data(&mut bytes, &debug_source_stream(1, 0));
        add_global_debug_offsets(&mut bytes, 0);

        let bytecode = HermesBytecode::parse(&bytes).expect("valid debug source locations");
        assert!(
            bytecode
                .global_function_header()
                .expect("valid global function")
                .has_debug_info()
        );
    }

    #[test]
    fn accepts_debug_source_location_delta() {
        let mut bytes = fixture_bytecode();
        set_debug_data(
            &mut bytes,
            &debug_source_stream_with_location_delta(1, 0, 1, 1),
        );
        add_global_debug_offsets(&mut bytes, 0);

        HermesBytecode::parse(&bytes).expect("valid debug source location delta");
    }

    #[test]
    fn accepts_debug_filename_table_and_file_region() {
        let mut bytes = fixture_bytecode();
        set_debug_info(
            &mut bytes,
            &[(0, 8, false)],
            b"file.js\0",
            &[(0, 0, DEBUG_SOURCE_MAPPING_URL_INVALID)],
            &debug_source_stream(1, 0),
        );
        add_global_debug_offsets(&mut bytes, 0);

        let bytecode = HermesBytecode::parse(&bytes).expect("valid debug file regions");
        assert!(
            bytecode
                .global_function_header()
                .expect("valid global function")
                .has_debug_info()
        );
    }

    #[test]
    fn accepts_utf16_debug_filename_entry() {
        let mut bytes = fixture_bytecode();
        set_debug_info(&mut bytes, &[(0, 1, true)], &[0, 0], &[], &[]);

        HermesBytecode::parse(&bytes).expect("valid UTF-16 debug filename entry");
    }

    #[test]
    fn accepts_debug_file_region_source_mapping_url() {
        let mut bytes = fixture_bytecode();
        set_debug_info(
            &mut bytes,
            &[(0, 1, false), (1, 1, false)],
            b"am",
            &[(0, 0, 1)],
            &[0],
        );

        HermesBytecode::parse(&bytes).expect("valid source map URL file region");
    }

    #[test]
    fn accepts_ordered_debug_file_regions() {
        let mut bytes = fixture_bytecode();
        set_debug_info(
            &mut bytes,
            &[(0, 1, false), (1, 1, false)],
            b"ab",
            &[(0, 0, 0), (1, 1, 0)],
            &[0, 0],
        );

        HermesBytecode::parse(&bytes).expect("ordered debug file regions");
    }

    #[test]
    fn accepts_exception_table_with_debug_offsets() {
        let mut bytes = fixture_bytecode();
        add_global_function_info(
            &mut bytes,
            Some((1, &[(0, 2, 0)])),
            Some(DEBUG_OFFSETS_NO_OFFSET),
        );

        let bytecode = HermesBytecode::parse(&bytes).expect("valid function info");
        let global = bytecode
            .global_function_header()
            .expect("valid global function");
        assert!(global.has_exception_handler());
        assert!(global.has_debug_info());
    }

    #[test]
    fn rejects_debug_filename_outside_storage() {
        let mut bytes = fixture_bytecode();
        set_debug_info(&mut bytes, &[(1, 3, false)], b"abc", &[], &[]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid debug filename must fail");
        assert_eq!(
            error,
            ParseError::DebugFilenameOutOfBounds {
                filename_index: 0,
                offset: 1,
                length: 3,
                storage_size: 3,
            },
        );
    }

    #[test]
    fn rejects_debug_file_region_filename_outside_table() {
        let mut bytes = fixture_bytecode();
        set_debug_info(&mut bytes, &[(0, 1, false)], b"a", &[(0, 1, 0)], &[0]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid filename id must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugFileRegion {
                region_index: 0,
                from_address: 0,
                filename_id: 1,
                source_mapping_url_id: 0,
                filename_count: 1,
                debug_data_size: 1,
            },
        );
    }

    #[test]
    fn rejects_debug_file_region_source_mapping_url_outside_table() {
        let mut bytes = fixture_bytecode();
        set_debug_info(&mut bytes, &[(0, 1, false)], b"a", &[(0, 0, 1)], &[0]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid source map URL id must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugFileRegion {
                region_index: 0,
                from_address: 0,
                filename_id: 0,
                source_mapping_url_id: 1,
                filename_count: 1,
                debug_data_size: 1,
            },
        );
    }

    #[test]
    fn rejects_debug_file_region_outside_debug_data() {
        let mut bytes = fixture_bytecode();
        set_debug_info(&mut bytes, &[(0, 1, false)], b"a", &[(1, 0, 0)], &[0]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid file region must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugFileRegion {
                region_index: 0,
                from_address: 1,
                filename_id: 0,
                source_mapping_url_id: 0,
                filename_count: 1,
                debug_data_size: 1,
            },
        );
    }

    #[test]
    fn rejects_unsorted_debug_file_regions() {
        let mut bytes = fixture_bytecode();
        set_debug_info(
            &mut bytes,
            &[(0, 1, false), (1, 1, false)],
            b"ab",
            &[(1, 1, 0), (0, 0, 0)],
            &[0, 0],
        );

        let error = HermesBytecode::parse(&bytes).expect_err("unsorted file regions must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugFileRegionOrder {
                region_index: 1,
                previous_from_address: 1,
                from_address: 0,
            },
        );
    }

    #[test]
    fn rejects_debug_source_locations_for_wrong_function() {
        let mut bytes = fixture_bytecode();
        set_debug_data(&mut bytes, &debug_source_stream(0, 0));
        add_global_debug_offsets(&mut bytes, 0);

        let error = HermesBytecode::parse(&bytes).expect_err("wrong debug function id must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugData {
                function_id: 1,
                offset: 0,
                reason: DEBUG_DATA_FUNCTION_MISMATCH,
            },
        );
    }

    #[test]
    fn rejects_unterminated_debug_source_locations() {
        let mut bytes = fixture_bytecode();
        set_debug_data(&mut bytes, &debug_source_stream_without_sentinel(1));
        add_global_debug_offsets(&mut bytes, 0);

        let error = HermesBytecode::parse(&bytes).expect_err("unterminated debug source must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugData {
                function_id: 1,
                offset: 4,
                reason: DEBUG_DATA_TRUNCATED_LEB,
            },
        );
    }

    #[test]
    fn rejects_debug_source_location_zero_column() {
        let mut bytes = fixture_bytecode();
        set_debug_data(
            &mut bytes,
            &debug_source_stream_with_location_delta(1, 0, 0, -1),
        );
        add_global_debug_offsets(&mut bytes, 0);

        let error = HermesBytecode::parse(&bytes).expect_err("zero column location must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugData {
                function_id: 1,
                offset: 5,
                reason: DEBUG_DATA_LOCATION_NOT_ONE_BASED,
            },
        );
    }

    #[test]
    fn rejects_debug_source_address_outside_body() {
        let mut bytes = fixture_bytecode();
        set_debug_data(&mut bytes, &debug_source_stream(1, 3));
        add_global_debug_offsets(&mut bytes, 0);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid debug address must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugData {
                function_id: 1,
                offset: 4,
                reason: DEBUG_DATA_ADDRESS_OUT_OF_BOUNDS,
            },
        );
    }

    #[test]
    fn rejects_debug_source_address_between_instructions() {
        let mut bytes = fixture_bytecode();
        set_debug_data(&mut bytes, &debug_source_stream(1, 1));
        add_global_debug_offsets(&mut bytes, 0);

        let error =
            HermesBytecode::parse(&bytes).expect_err("non-boundary debug address must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugData {
                function_id: 1,
                offset: 4,
                reason: DEBUG_DATA_ADDRESS_NOT_BOUNDARY,
            },
        );
    }

    #[test]
    fn rejects_debug_offset_outside_debug_data() {
        let mut bytes = fixture_bytecode();
        add_global_debug_offsets(&mut bytes, 0);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid debug offset must fail");
        assert_eq!(
            error,
            ParseError::InvalidDebugOffset {
                function_id: 1,
                source_locations: 0,
                debug_data_size: 0,
            },
        );
    }

    #[test]
    fn rejects_debug_info_section_outside_footer() {
        let mut bytes = fixture_bytecode();
        let (debug_info_offset, file_length) = {
            let bytecode = HermesBytecode::parse(&bytes).expect("valid fixture");
            (
                bytecode.header().debug_info_offset,
                bytecode.header().file_length,
            )
        };
        write_u32(&mut bytes, DEBUG_INFO_OFFSET_OFFSET, debug_info_offset + 8);

        let error = HermesBytecode::parse(&bytes).expect_err("short debug info must fail");
        assert_eq!(
            error,
            ParseError::DebugInfoOutOfBounds {
                offset: debug_info_offset + 8,
                size: DEBUG_INFO_HEADER_SIZE as u32,
                limit: file_length - HERMES_BYTECODE_FOOTER_SIZE as u32,
            },
        );
    }

    #[test]
    fn rejects_missing_function_info_for_exception_flag() {
        let mut bytes = fixture_bytecode();
        let (function_headers_offset, function_body_offset) = {
            let bytecode = HermesBytecode::parse(&bytes).expect("valid fixture");
            (
                bytecode.sections().function_headers().offset() as usize,
                bytecode
                    .function_body(0)
                    .expect("valid function body")
                    .offset(),
            )
        };
        bytes[function_headers_offset..function_headers_offset + SMALL_FUNC_HEADER_SIZE]
            .copy_from_slice(&encode_small_function_header(SmallFunctionHeaderFixture {
                offset: function_body_offset,
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
                flags: 0b0000_1100,
            }));

        let error = HermesBytecode::parse(&bytes).expect_err("missing function info must fail");
        assert_eq!(error, ParseError::MissingFunctionInfo { function_id: 0 });
    }

    #[test]
    fn rejects_invalid_opcode_in_function_body() {
        let mut bytes = fixture_bytecode();
        let function_body_offset = HermesBytecode::parse(&bytes)
            .expect("valid fixture")
            .function_body(0)
            .expect("valid function body")
            .offset() as usize;
        bytes[function_body_offset] = 0xff;

        let error = HermesBytecode::parse(&bytes).expect_err("invalid opcode must fail");
        assert_eq!(
            error,
            ParseError::InvalidOpcode {
                function_id: 0,
                offset: function_body_offset as u32,
                opcode: 0xff,
            },
        );
    }

    #[test]
    fn rejects_invalid_string_operand_reference() {
        let mut bytes = fixture_bytecode();
        let function_body_offset =
            replace_global_body(&mut bytes, &[LOAD_CONST_STRING_OPCODE, 0, 99, 0]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid string operand must fail");
        assert_eq!(
            error,
            ParseError::InvalidInstructionTableReference {
                function_id: 1,
                offset: function_body_offset,
                opcode: LOAD_CONST_STRING_OPCODE,
                table: "string",
                index: 99,
                limit: 2,
            },
        );
    }

    #[test]
    fn rejects_invalid_function_operand_reference() {
        let mut bytes = fixture_bytecode();
        let function_body_offset =
            replace_global_body(&mut bytes, &[CREATE_CLOSURE_OPCODE, 0, 0, 99, 0]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid function operand must fail");
        assert_eq!(
            error,
            ParseError::InvalidInstructionTableReference {
                function_id: 1,
                offset: function_body_offset,
                opcode: CREATE_CLOSURE_OPCODE,
                table: "function",
                index: 99,
                limit: 2,
            },
        );
    }

    #[test]
    fn rejects_invalid_bigint_operand_reference() {
        let mut bytes = fixture_bytecode();
        let function_body_offset =
            replace_global_body(&mut bytes, &[LOAD_CONST_BIGINT_OPCODE, 0, 99, 0]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid bigint operand must fail");
        assert_eq!(
            error,
            ParseError::InvalidInstructionTableReference {
                function_id: 1,
                offset: function_body_offset,
                opcode: LOAD_CONST_BIGINT_OPCODE,
                table: "bigint",
                index: 99,
                limit: 1,
            },
        );
    }

    #[test]
    fn rejects_jump_target_between_instructions() {
        let mut bytes = fixture_bytecode();
        let function_body_offset = replace_global_body(&mut bytes, &[JMP_OPCODE, 1]);

        let error = HermesBytecode::parse(&bytes).expect_err("invalid jump target must fail");
        assert_eq!(
            error,
            ParseError::InvalidJumpTarget {
                function_id: 1,
                offset: function_body_offset,
                opcode: JMP_OPCODE,
                target: i64::from(function_body_offset + 1),
                body_start: function_body_offset,
                body_end: function_body_offset + 2,
            },
        );
    }

    #[test]
    fn accepts_valid_uint_switch_table_target() {
        let mut bytes = fixture_bytecode();
        let mut body = [0; 18];
        body[0] = UINT_SWITCH_IMM_OPCODE;
        body[2..6].copy_from_slice(&18_u32.to_le_bytes());
        body[6..10].copy_from_slice(&0_i32.to_le_bytes());
        body[10..14].copy_from_slice(&0_u32.to_le_bytes());
        body[14..18].copy_from_slice(&0_u32.to_le_bytes());

        replace_global_body_with_payload(&mut bytes, &body, &0_i32.to_le_bytes());

        let bytecode = HermesBytecode::parse(&bytes).expect("valid switch table target");
        assert_eq!(bytecode.global_instruction_count(), Ok(1));
    }

    #[test]
    fn rejects_switch_table_target_between_instructions() {
        let mut bytes = fixture_bytecode();
        let mut body = [0; 18];
        body[0] = UINT_SWITCH_IMM_OPCODE;
        body[2..6].copy_from_slice(&18_u32.to_le_bytes());
        body[6..10].copy_from_slice(&0_i32.to_le_bytes());
        body[10..14].copy_from_slice(&0_u32.to_le_bytes());
        body[14..18].copy_from_slice(&0_u32.to_le_bytes());
        let function_body_offset =
            replace_global_body_with_payload(&mut bytes, &body, &1_i32.to_le_bytes()).0;

        let error = HermesBytecode::parse(&bytes).expect_err("invalid switch target must fail");
        assert_eq!(
            error,
            ParseError::InvalidSwitchTableTarget {
                function_id: 1,
                offset: function_body_offset,
                opcode: UINT_SWITCH_IMM_OPCODE,
                entry_index: 0,
                target: i64::from(function_body_offset + 1),
                body_start: function_body_offset,
                body_end: function_body_offset + 18,
            },
        );
    }

    #[test]
    fn rejects_string_switch_table_string_reference() {
        let mut bytes = fixture_bytecode();
        write_u32(&mut bytes, NUM_STRING_SWITCH_IMMS_OFFSET, 1);
        let mut body = [0; 18];
        body[0] = STRING_SWITCH_IMM_OPCODE;
        body[2..6].copy_from_slice(&0_u32.to_le_bytes());
        body[6..10].copy_from_slice(&18_u32.to_le_bytes());
        body[10..14].copy_from_slice(&0_i32.to_le_bytes());
        body[14..18].copy_from_slice(&1_u32.to_le_bytes());
        let function_body_offset =
            replace_global_body_with_payload(&mut bytes, &body, &[99, 0, 0, 0, 0, 0, 0, 0]).0;

        let error =
            HermesBytecode::parse(&bytes).expect_err("invalid string switch label must fail");
        assert_eq!(
            error,
            ParseError::InvalidInstructionTableReference {
                function_id: 1,
                offset: function_body_offset,
                opcode: STRING_SWITCH_IMM_OPCODE,
                table: "string",
                index: 99,
                limit: 2,
            },
        );
    }

    #[test]
    fn rejects_instruction_that_exceeds_function_body() {
        let mut bytes = fixture_bytecode();
        let (function_body_offset, function_body_end) = {
            let body = HermesBytecode::parse(&bytes)
                .expect("valid fixture")
                .function_body(0)
                .expect("valid function body");
            (body.offset() as usize, body.offset() + body.len())
        };
        bytes[function_body_offset] = LOAD_CONST_DOUBLE_OPCODE;

        let error = HermesBytecode::parse(&bytes).expect_err("wide instruction must fail");
        assert_eq!(
            error,
            ParseError::InstructionOutOfBounds {
                function_id: 0,
                offset: function_body_offset as u32,
                opcode: LOAD_CONST_DOUBLE_OPCODE,
                width: 10,
                body_end: function_body_end,
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
