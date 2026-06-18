//! Find Hermes bytecode closure creation sites for one target function.

use std::env;
use std::fs;
use std::process::ExitCode;

use iris_hbc::HermesBytecode;

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "hbc-find-closure".to_owned());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <index.android.bundle> <target-function-id>");
        return ExitCode::from(2);
    };
    let Some(target_function_id) = args.next().and_then(|arg| arg.parse::<u32>().ok()) else {
        eprintln!("usage: {program} <index.android.bundle> <target-function-id>");
        return ExitCode::from(2);
    };

    if args.next().is_some() {
        eprintln!("usage: {program} <index.android.bundle> <target-function-id>");
        return ExitCode::from(2);
    }

    match run(&path, target_function_id) {
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

fn run(path: &str, target_function_id: u32) -> Result<String, String> {
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
            let created_function_id = match instruction.opcode {
                132 => read_unsigned_operand(bytes, 3, 2),
                133 => read_unsigned_operand(bytes, 3, 4),
                _ => continue,
            };

            if created_function_id == target_function_id {
                let destination = read_unsigned_operand(bytes, 1, 1);
                let environment_register = read_unsigned_operand(bytes, 2, 1);
                lines.push(format!(
                    "function={function_id} offset={} opcode={} dst=r{destination} env=r{environment_register}",
                    instruction.offset, instruction.opcode
                ));
            }
        }
    }

    if lines.is_empty() {
        Ok(format!("targetFunction={target_function_id}, matches=0"))
    } else {
        Ok(format!(
            "targetFunction={target_function_id}, matches={}\n{}",
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
        usize::try_from(instruction_offset).expect("Hermes instruction offset fits in usize");
    let body_start = usize::try_from(body_start).expect("Hermes body offset fits in usize");
    let instruction_width = usize::from(instruction_width);
    let body_offset = instruction_offset
        .checked_sub(body_start)
        .ok_or_else(|| format!("instruction {instruction_offset} is before function body"))?;

    body_bytes
        .get(body_offset..body_offset + instruction_width)
        .ok_or_else(|| format!("instruction {instruction_offset} exceeds function body"))
}

fn read_unsigned_operand(bytes: &[u8], offset: usize, width: usize) -> u32 {
    let mut value = 0_u32;
    for index in 0..width {
        value |= u32::from(bytes[offset + index]) << (index * 8);
    }
    value
}
