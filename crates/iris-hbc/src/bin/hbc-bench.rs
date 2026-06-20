//! Benchmark Iris' current scalar executor against a Hermes bytecode bundle.

use std::env;
use std::fs;
use std::process::ExitCode;

use iris_hbc::ScalarValue;

fn main() -> ExitCode {
    match run_from_args(env::args().collect()) {
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

fn run_from_args(args: Vec<String>) -> Result<String, String> {
    let program = args.first().map_or("hbc-bench", String::as_str);
    let mut path = None;
    let mut warmup_iterations = 3;
    let mut measured_iterations = 20;
    let mut sample_inner_iterations = 1;
    let mut fast_paths_enabled = true;

    for arg in args.iter().skip(1) {
        if let Some(value) = arg.strip_prefix("--warmup=") {
            warmup_iterations = parse_positive_u32("--warmup", value, true)?;
        } else if let Some(value) = arg.strip_prefix("--iterations=") {
            measured_iterations = parse_positive_u32("--iterations", value, false)?;
        } else if let Some(value) = arg.strip_prefix("--sample-inner-iterations=") {
            sample_inner_iterations =
                parse_positive_u32("--sample-inner-iterations", value, false)?;
        } else if arg == "--disable-fast-paths" {
            fast_paths_enabled = false;
        } else if arg.starts_with("--") {
            return Err(format!("unknown option: {arg}\n{}", usage(program)));
        } else if path.replace(arg.clone()).is_some() {
            return Err(format!("multiple HBC paths provided\n{}", usage(program)));
        }
    }

    let Some(path) = path else {
        return Err(usage(program));
    };

    let bytes = fs::read(&path)
        .map_err(|error| format!("failed to read Hermes bytecode bundle: {error}"))?;
    let report = iris_hbc::benchmark_global_scalar_function_with_inner_iterations_and_fast_paths(
        &bytes,
        warmup_iterations,
        measured_iterations,
        sample_inner_iterations,
        fast_paths_enabled,
    )
    .map_err(|error| format!("failed to benchmark Hermes bytecode scalar subset: {error}"))?;

    Ok(format!(
        "{{\"engine\":\"iris\",\"casePath\":\"{}\",\"value\":{},\"declaredGlobals\":{},\"fastPathsEnabled\":{},\"warmupIterations\":{},\"measuredIterations\":{},\"sampleInnerIterations\":{},\"samplesMs\":[{}]}}",
        json_escape(&path),
        scalar_value_json(report.value),
        report.declared_global_count,
        fast_paths_enabled,
        report.warmup_iterations,
        report.measured_iterations,
        report.sample_inner_iterations,
        report
            .samples_ms
            .iter()
            .map(|sample| format!("{sample:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    ))
}

fn usage(program: &str) -> String {
    format!(
        "usage: {program} [--warmup=N] [--iterations=N] [--sample-inner-iterations=N] [--disable-fast-paths] <bundle.hbc>"
    )
}

fn parse_positive_u32(name: &str, value: &str, allow_zero: bool) -> Result<u32, String> {
    let parsed = value
        .parse::<u32>()
        .map_err(|error| format!("{name} must be an integer: {error}"))?;
    if !allow_zero && parsed == 0 {
        return Err(format!("{name} must be greater than zero"));
    }
    Ok(parsed)
}

fn scalar_value_json(value: ScalarValue) -> String {
    match value {
        ScalarValue::Boolean(value) => value.to_string(),
        ScalarValue::Empty => "\"<empty>\"".to_owned(),
        ScalarValue::Environment(value) => format!("\"Environment({})\"", value.environment_id),
        ScalarValue::Function(value) => format!("\"{}\"", json_escape(&format!("{value:?}"))),
        ScalarValue::Null => "null".to_owned(),
        ScalarValue::DynamicString(value) => format!("\"DynamicString({})\"", value.dynamic_id),
        ScalarValue::Number(value) if value.is_finite() => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        }
        ScalarValue::Number(value) => format!("\"{value}\""),
        ScalarValue::Object(value) => format!("\"{value:?}\""),
        ScalarValue::String(value) => format!("\"String({})\"", value.string_id),
        ScalarValue::Symbol(value) => format!("\"Symbol({})\"", value.string_id),
        ScalarValue::Undefined => "\"undefined\"".to_owned(),
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character => escaped.push(character),
        }
    }
    escaped
}
