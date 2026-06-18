//! Finds Hermes function source-table entries by source-string substring.

use std::env;
use std::fs;
use std::process::ExitCode;

use iris_hbc::HermesBytecode;

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "hbc-find-source".to_owned());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <index.android.bundle> <source-substring>");
        return ExitCode::from(2);
    };
    let Some(pattern) = args.next() else {
        eprintln!("usage: {program} <index.android.bundle> <source-substring>");
        return ExitCode::from(2);
    };

    if args.next().is_some() {
        eprintln!("usage: {program} <index.android.bundle> <source-substring>");
        return ExitCode::from(2);
    }

    match run(&path, &pattern) {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(path: &str, pattern: &str) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read Hermes bytecode bundle: {error}"))?;
    let bytecode = HermesBytecode::parse(&bytes)
        .map_err(|error| format!("failed to parse Hermes bytecode bundle: {error}"))?;
    let mut lines = Vec::new();
    let table = bytecode.sections().function_source_table().bytes();
    for (entry_index, entry) in table.chunks_exact(8).enumerate() {
        let function_id = read_u32(entry, 0)?;
        let string_id = read_u32(entry, 4)?;
        let Ok(source) = bytecode.string(string_id) else {
            continue;
        };
        if source.is_utf16() {
            continue;
        }
        let source = String::from_utf8_lossy(source.bytes());
        if source.contains(pattern) {
            lines.push(format!(
                "entry={entry_index} function={function_id} string={string_id} source={source}"
            ));
        }
    }

    if lines.is_empty() {
        Ok(format!("matches=0 pattern={pattern}\n"))
    } else {
        Ok(format!(
            "matches={} pattern={pattern}\n{}\n",
            lines.len(),
            lines.join("\n")
        ))
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("source-table entry missing u32 at offset {offset}"))?;
    Ok(u32::from_le_bytes(bytes.try_into().map_err(|_| {
        format!("source-table entry has invalid u32 at offset {offset}")
    })?))
}
