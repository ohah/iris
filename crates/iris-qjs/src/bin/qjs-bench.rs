//! Run the host-side Iris QuickJS backend benchmark.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use iris_qjs::{BenchmarkOptions, run_quickjs_benchmark};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut smoke = false;
    let mut output_path = None;
    let mut warmup_iterations = None;
    let mut measured_iterations = None;

    for arg in env::args().skip(1) {
        if arg == "--smoke" {
            smoke = true;
        } else if let Some(value) = arg.strip_prefix("--output=") {
            output_path = Some(PathBuf::from(value));
        } else if let Some(value) = arg.strip_prefix("--warmup=") {
            warmup_iterations = Some(parse_iterations("--warmup", value, true)?);
        } else if let Some(value) = arg.strip_prefix("--iterations=") {
            measured_iterations = Some(parse_iterations("--iterations", value, false)?);
        } else {
            return Err(format!("unknown option: {arg}\n{}", usage()));
        }
    }

    let output_path = output_path.unwrap_or_else(|| {
        PathBuf::from(if smoke {
            "artifacts/bench/qjs-baseline-smoke.json"
        } else {
            "artifacts/bench/qjs-baseline.json"
        })
    });
    let measured_iterations = measured_iterations.unwrap_or(if smoke { 3 } else { 15 });
    let warmup_iterations = warmup_iterations.unwrap_or(if smoke { 1 } else { 3 });
    let commit = git_value(&["rev-parse", "HEAD"]);
    let short_commit = git_value(&["rev-parse", "--short", "HEAD"]);

    let options = BenchmarkOptions {
        artifact_path: output_path.to_string_lossy().into_owned(),
        commit,
        device: "local-host".to_owned(),
        measured_iterations,
        mode: if cfg!(debug_assertions) {
            "development"
        } else {
            "release"
        }
        .to_owned(),
        source: format!("git:{short_commit}"),
        warmup_iterations,
    };
    let report = run_quickjs_benchmark(&options)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("failed to encode QuickJS benchmark report: {error}"))?;
    fs::write(&output_path, format!("{json}\n"))
        .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;

    println!("benchmark artifact: {}", output_path.display());
    for case in report_cases(&json)? {
        println!(
            "{}: p50={}ms p95={}ms checksum={}",
            case.id, case.p50, case.p95, case.checksum
        );
    }

    Ok(())
}

fn usage() -> &'static str {
    "usage: qjs-bench [--smoke] [--warmup=N] [--iterations=N] [--output=PATH]"
}

fn parse_iterations(name: &str, value: &str, allow_zero: bool) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|error| format!("{name} must be an integer: {error}"))?;
    if !allow_zero && parsed == 0 {
        return Err(format!("{name} must be greater than zero"));
    }
    Ok(parsed)
}

fn git_value(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

#[derive(serde::Deserialize)]
struct PrintableReport {
    cases: Vec<PrintableCase>,
}

#[derive(serde::Deserialize)]
struct PrintableCase {
    checksum: serde_json::Value,
    id: String,
    stats: PrintableStats,
}

#[derive(serde::Deserialize)]
struct PrintableStats {
    p50: f64,
    p95: f64,
}

struct PrintableCaseSummary {
    checksum: serde_json::Value,
    id: String,
    p50: f64,
    p95: f64,
}

fn report_cases(json: &str) -> Result<Vec<PrintableCaseSummary>, String> {
    let report: PrintableReport = serde_json::from_str(json)
        .map_err(|error| format!("failed to read report for console summary: {error}"))?;
    Ok(report
        .cases
        .into_iter()
        .map(|case| PrintableCaseSummary {
            checksum: case.checksum,
            id: case.id,
            p50: case.stats.p50,
            p95: case.stats.p95,
        })
        .collect())
}
