//! Trace Iris' current scalar executor against a Hermes bytecode bundle.

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "hbc-trace".to_owned());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <index.android.bundle>");
        return ExitCode::from(2);
    };

    if args.next().is_some() {
        eprintln!("usage: {program} <index.android.bundle>");
        return ExitCode::from(2);
    }

    match run(&path) {
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

fn run(path: &str) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read Hermes bytecode bundle: {error}"))?;
    iris_hbc::describe_hbc_scalar_frontier_trace(&bytes)
        .map_err(|error| format!("failed to trace Hermes bytecode scalar subset: {error}"))
}
