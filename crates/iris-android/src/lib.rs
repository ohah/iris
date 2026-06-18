//! Android CXX bridge for Iris runtime components.
//!
//! This crate links Iris Rust components into one static library so Android
//! does not link multiple `cxx` runtime symbol copies.

#[cxx::bridge(namespace = "iris::android")]
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
        fn describe_hbc_execution_gap(bytes: &[u8]) -> Result<String>;
        fn describe_hbc_scalar_execution(bytes: &[u8]) -> Result<String>;
        fn describe_hbc_strict_scalar_execution(bytes: &[u8]) -> Result<String>;
        fn run_quickjs_benchmark_json(
            artifact_path: &str,
            commit: &str,
            device: &str,
            mode: &str,
            source: &str,
            warmup_iterations: u32,
            measured_iterations: u32,
        ) -> Result<String>;
    }
}

fn parse_hbc_metadata(bytes: &[u8]) -> Result<ffi::HbcMetadata, String> {
    let bytecode = iris_hbc::HermesBytecode::parse(bytes).map_err(|error| error.to_string())?;
    let header = bytecode.header();
    let sections = bytecode.sections();
    let global_function = bytecode
        .global_function_header()
        .map_err(|error| error.to_string())?;
    let global_instruction_count = bytecode
        .global_instruction_count()
        .map_err(|error| error.to_string())?;

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

fn describe_hbc_execution_gap(bytes: &[u8]) -> Result<String, String> {
    iris_hbc::describe_hbc_execution_gap(bytes).map_err(|error| error.to_string())
}

fn describe_hbc_scalar_execution(bytes: &[u8]) -> Result<String, String> {
    iris_hbc::describe_hbc_scalar_execution(bytes).map_err(|error| error.to_string())
}

fn describe_hbc_strict_scalar_execution(bytes: &[u8]) -> Result<String, String> {
    iris_hbc::describe_hbc_strict_scalar_execution(bytes).map_err(|error| error.to_string())
}

fn run_quickjs_benchmark_json(
    artifact_path: &str,
    commit: &str,
    device: &str,
    mode: &str,
    source: &str,
    warmup_iterations: u32,
    measured_iterations: u32,
) -> Result<String, String> {
    iris_qjs::run_quickjs_benchmark_json(
        artifact_path,
        commit,
        device,
        mode,
        source,
        warmup_iterations,
        measured_iterations,
    )
}
