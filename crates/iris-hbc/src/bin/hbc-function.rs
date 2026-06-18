//! Print one Hermes function header for Iris scalar executor work.

use std::env;
use std::fs;
use std::process::ExitCode;

use iris_hbc::HermesBytecode;

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "hbc-function".to_owned());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <index.android.bundle> <function-id>");
        return ExitCode::from(2);
    };
    let Some(function_id) = args.next().and_then(|arg| arg.parse::<u32>().ok()) else {
        eprintln!("usage: {program} <index.android.bundle> <function-id>");
        return ExitCode::from(2);
    };

    if args.next().is_some() {
        eprintln!("usage: {program} <index.android.bundle> <function-id>");
        return ExitCode::from(2);
    }

    match run(&path, function_id) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(path: &str, function_id: u32) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read Hermes bytecode bundle: {error}"))?;
    let bytecode = HermesBytecode::parse(&bytes)
        .map_err(|error| format!("failed to parse Hermes bytecode bundle: {error}"))?;
    let header = bytecode
        .function_header(function_id)
        .map_err(|error| format!("failed to read function {function_id} header: {error}"))?;

    Ok(format!(
        "function={function_id}, offset={}, size={}, params={}, frameSize={}, readCache={}, writeCache={}, flags={:#010b}",
        header.offset,
        header.bytecode_size_in_bytes,
        header.param_count,
        header.frame_size,
        header.read_cache_size,
        header.write_cache_size,
        header.flags,
    ))
}
