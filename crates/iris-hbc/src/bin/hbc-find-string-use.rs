//! Finds Hermes instructions that reference strings matching a substring.

use std::env;
use std::fs;
use std::process::ExitCode;

use iris_hbc::HermesBytecode;

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "hbc-find-string-use".to_owned());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <index.android.bundle> <string-substring>");
        return ExitCode::from(2);
    };
    let Some(pattern) = args.next() else {
        eprintln!("usage: {program} <index.android.bundle> <string-substring>");
        return ExitCode::from(2);
    };

    if args.next().is_some() {
        eprintln!("usage: {program} <index.android.bundle> <string-substring>");
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
    for function_id in 0..bytecode.header().function_count {
        let body = bytecode
            .function_body(function_id)
            .map_err(|error| format!("failed to read function {function_id} body: {error}"))?;
        let instructions = bytecode
            .function_instructions(function_id)
            .map_err(|error| {
                format!("failed to read function {function_id} instructions: {error}")
            })?;
        for instruction in instructions {
            let instruction = instruction
                .map_err(|error| format!("failed to decode function {function_id}: {error}"))?;
            let bytes = instruction_bytes(
                body.offset(),
                body.bytes(),
                instruction.offset,
                instruction.width,
            )?;
            for string_id in instruction_string_operands(instruction.opcode, bytes) {
                let Ok(string) = bytecode.string(string_id) else {
                    continue;
                };
                if string.is_utf16() {
                    continue;
                }
                let string = String::from_utf8_lossy(string.bytes());
                if string.contains(pattern) {
                    lines.push(format!(
                        "function={function_id} offset={} opcode={} string={}#{string_id}",
                        instruction.offset, instruction.opcode, string
                    ));
                }
            }
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

fn instruction_bytes(
    body_start: u32,
    body_bytes: &[u8],
    instruction_offset: u32,
    instruction_width: u8,
) -> Result<&[u8], String> {
    let instruction_offset =
        usize::try_from(instruction_offset).map_err(|_| "instruction offset exceeds usize")?;
    let body_start = usize::try_from(body_start).map_err(|_| "body offset exceeds usize")?;
    let instruction_width = usize::from(instruction_width);
    let body_offset = instruction_offset
        .checked_sub(body_start)
        .ok_or_else(|| format!("instruction {instruction_offset} is before function body"))?;

    body_bytes
        .get(body_offset..body_offset + instruction_width)
        .ok_or_else(|| format!("instruction {instruction_offset} exceeds function body"))
}

fn instruction_string_operands(opcode: u8, bytes: &[u8]) -> Vec<u32> {
    match opcode {
        68 => vec![read_unsigned_operand(bytes, 4, 1)],
        69 | 72 | 74 | 75 => vec![read_unsigned_operand(bytes, 4, 2)],
        86 => vec![read_unsigned_operand(bytes, 4, 2)],
        87 => vec![read_unsigned_operand(bytes, 4, 4)],
        144 => vec![read_unsigned_operand(bytes, 2, 2)],
        _ => Vec::new(),
    }
}

fn read_unsigned_operand(bytes: &[u8], offset: usize, width: usize) -> u32 {
    bytes[offset..offset + width]
        .iter()
        .enumerate()
        .fold(0_u32, |value, (index, byte)| {
            value | (u32::from(*byte) << (index * 8))
        })
}
