//! Print a Hermes bytecode instruction window for Iris scalar executor work.

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "hbc-window".to_owned());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <index.android.bundle> <function-id> <offset> [context]");
        return ExitCode::from(2);
    };
    let Some(function_id) = args.next().and_then(|arg| arg.parse::<u32>().ok()) else {
        eprintln!("usage: {program} <index.android.bundle> <function-id> <offset> [context]");
        return ExitCode::from(2);
    };
    let Some(offset) = args.next().and_then(|arg| arg.parse::<u32>().ok()) else {
        eprintln!("usage: {program} <index.android.bundle> <function-id> <offset> [context]");
        return ExitCode::from(2);
    };
    let context = args
        .next()
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(8);

    if args.next().is_some() {
        eprintln!("usage: {program} <index.android.bundle> <function-id> <offset> [context]");
        return ExitCode::from(2);
    }

    match run(&path, function_id, offset, context) {
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

fn run(path: &str, function_id: u32, offset: u32, context: usize) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read Hermes bytecode bundle: {error}"))?;
    iris_hbc::describe_hbc_instruction_window(&bytes, function_id, offset, context)
        .map_err(|error| format!("failed to describe Hermes bytecode instruction window: {error}"))
}
