//! QuickJS backend experiment boundary for Iris.
//!
//! The first Iris QuickJS work should prove adapter compatibility before it is
//! treated as a production runtime path.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use rquickjs::{Context, Function, Runtime};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cxx::bridge(namespace = "iris::qjs")]
mod ffi {
    extern "Rust" {
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

/// Backend identifier used in benchmarks.
pub const BACKEND_NAME: &str = "iris-qjs";

const BENCHMARK_SCHEMA_VERSION: &str = "iris.benchmark.v1";
const DEFAULT_WARMUP_ITERATIONS: usize = 3;
const DEFAULT_MEASURED_ITERATIONS: usize = 15;

const QUICKJS_BENCHMARK_SOURCE: &str = r#"
function runComputeCase() {
  let checksum = 0;

  for (let index = 0; index < 600_000; index += 1) {
    checksum += Math.sqrt((index % 1_000) + 1) * Math.sin(index);
  }

  return {
    checksum: Number(checksum.toFixed(3)),
    detail: "600k math operations",
  };
}

function runJsonCase() {
  const payload = Array.from({ length: 8_000 }, (_, index) => ({
    active: index % 3 === 0,
    id: index,
    meta: {
      lane: index % 7,
      label: `group-${index % 11}`,
    },
    name: `item-${index}`,
    points: [index, index * 2, index * 3],
  }));

  const encoded = JSON.stringify(payload);
  const decoded = JSON.parse(encoded);
  const checksum = decoded.reduce((total, item) => total + item.id + item.points[2], 0);

  return {
    checksum,
    detail: `${encoded.length} encoded bytes`,
  };
}

function runObjectTraversalCase() {
  const rows = Array.from({ length: 12_000 }, (_, index) => ({
    id: index,
    lane: index % 9,
    nested: {
      active: index % 5 === 0,
      score: (index * 17) % 1_024,
    },
  }));

  const checksum = rows.reduce((total, row) => {
    if (!row.nested.active) {
      return total + row.lane;
    }

    return total + row.id + row.nested.score;
  }, 0);

  return {
    checksum,
    detail: `${rows.length} objects traversed`,
  };
}

function runTypedArrayCopyCase() {
  const source = new Uint8Array(1_000_000);

  for (let index = 0; index < source.length; index += 1) {
    source[index] = index % 251;
  }

  const copy = new Uint8Array(source.length);
  copy.set(source);

  let checksum = 0;
  for (let index = 0; index < copy.length; index += 10_000) {
    checksum += copy[index];
  }

  return {
    checksum,
    detail: `${copy.byteLength} bytes copied`,
  };
}

globalThis.__irisBenchCases = {
  "js-compute": runComputeCase,
  "json-round-trip": runJsonCase,
  "object-traversal": runObjectTraversalCase,
  "typed-array-copy": runTypedArrayCopyCase,
};

globalThis.__irisRunCase = function irisRunCase(caseId) {
  const benchmarkCase = globalThis.__irisBenchCases[caseId];
  if (benchmarkCase == null) {
    throw new Error(`Unknown Iris QuickJS benchmark case: ${caseId}`);
  }
  return JSON.stringify(benchmarkCase());
};
"#;

#[derive(Clone, Copy)]
struct BenchmarkCaseDefinition {
    description: &'static str,
    id: &'static str,
    label: &'static str,
}

const BENCHMARK_CASES: &[BenchmarkCaseDefinition] = &[
    BenchmarkCaseDefinition {
        description: "CPU-bound JavaScript math loop for Hermes baseline timing.",
        id: "js-compute",
        label: "JS compute",
    },
    BenchmarkCaseDefinition {
        description: "Large object array stringify/parse round trip.",
        id: "json-round-trip",
        label: "JSON round trip",
    },
    BenchmarkCaseDefinition {
        description: "Nested object creation and traversal baseline.",
        id: "object-traversal",
        label: "Object traversal",
    },
    BenchmarkCaseDefinition {
        description: "Uint8Array allocation and copy baseline before JSI buffer work.",
        id: "typed-array-copy",
        label: "TypedArray copy",
    },
];

/// Benchmark runner options for the host-side QuickJS backend probe.
#[derive(Clone, Debug)]
pub struct BenchmarkOptions {
    /// Relative or absolute artifact path recorded in the report.
    pub artifact_path: String,
    /// Git commit recorded in the report.
    pub commit: String,
    /// Device label recorded in the report.
    pub device: String,
    /// Measured iterations per benchmark case.
    pub measured_iterations: usize,
    /// Build mode recorded in the report.
    pub mode: String,
    /// Short source label recorded in the report.
    pub source: String,
    /// Warmup iterations per benchmark case.
    pub warmup_iterations: usize,
}

impl Default for BenchmarkOptions {
    fn default() -> Self {
        Self {
            artifact_path: "artifacts/bench/qjs-baseline.json".to_owned(),
            commit: "unknown".to_owned(),
            device: "local-host".to_owned(),
            measured_iterations: DEFAULT_MEASURED_ITERATIONS,
            mode: "unknown".to_owned(),
            source: "git:unknown".to_owned(),
            warmup_iterations: DEFAULT_WARMUP_ITERATIONS,
        }
    }
}

/// Complete benchmark suite report using the shared Iris benchmark schema.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSuiteReport {
    artifact: BenchmarkArtifact,
    cases: Vec<BenchmarkCaseReport>,
    created_at: String,
    metadata: BenchmarkMetadata,
    schema_version: &'static str,
    suite: BenchmarkSuite,
    summary: BenchmarkSummary,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkArtifact {
    generated_by: &'static str,
    kind: &'static str,
    path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkCaseReport {
    checksum: Value,
    description: &'static str,
    detail: String,
    id: &'static str,
    label: &'static str,
    measured_iterations: usize,
    stats: BenchmarkStats,
    unit: &'static str,
    warmup_iterations: usize,
}

#[derive(Debug, Serialize)]
struct BenchmarkStats {
    max: f64,
    mean: f64,
    min: f64,
    p50: f64,
    p95: f64,
    samples: Vec<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkMetadata {
    app: BenchmarkAppMetadata,
    build: BenchmarkBuildMetadata,
    platform: BenchmarkPlatformMetadata,
    react_native: BenchmarkReactNativeMetadata,
    runtime: BenchmarkRuntimeMetadata,
}

#[derive(Debug, Serialize)]
struct BenchmarkAppMetadata {
    name: &'static str,
    version: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkBuildMetadata {
    commit: String,
    compiler: &'static str,
    mode: String,
    source: String,
    source_hash: String,
    transform: &'static str,
}

#[derive(Debug, Serialize)]
struct BenchmarkPlatformMetadata {
    device: String,
    os: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct BenchmarkReactNativeMetadata {
    version: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkRuntimeMetadata {
    fabric: bool,
    hermes: bool,
    hermes_version: &'static str,
    js_engine: &'static str,
    new_architecture: bool,
    runtime_backend: &'static str,
    turbo_module_proxy: bool,
}

#[derive(Debug, Serialize)]
struct BenchmarkSuite {
    id: &'static str,
    name: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkSummary {
    case_count: usize,
    measured_iterations: usize,
    total_elapsed_ms: f64,
}

#[derive(Debug, Deserialize)]
struct BenchmarkRunValue {
    checksum: Value,
    detail: String,
}

/// Returns a short backend description for benchmark reports.
#[must_use]
pub fn backend_description() -> String {
    format!("{BACKEND_NAME} for {}", iris_core::COMPATIBILITY_TARGET)
}

/// Runs the shared Iris JavaScript benchmark cases on a host QuickJS context.
///
/// This is a backend microbenchmark. It does not prove React Native strict
/// engine comparison until the same backend is connected through JSI/Fabric.
pub fn run_quickjs_benchmark(options: &BenchmarkOptions) -> Result<BenchmarkSuiteReport, String> {
    if options.measured_iterations == 0 {
        return Err("measured_iterations must be greater than zero".to_owned());
    }

    let runtime =
        Runtime::new().map_err(|error| format!("failed to create QuickJS runtime: {error}"))?;
    let context = Context::full(&runtime)
        .map_err(|error| format!("failed to create QuickJS context: {error}"))?;
    let started_at = Instant::now();

    let cases = context.with(|ctx| {
        ctx.eval::<(), _>(QUICKJS_BENCHMARK_SOURCE)
            .map_err(|error| format!("failed to load QuickJS benchmark cases: {error}"))?;
        let runner: Function = ctx
            .globals()
            .get("__irisRunCase")
            .map_err(|error| format!("failed to read QuickJS benchmark runner: {error}"))?;

        BENCHMARK_CASES
            .iter()
            .map(|case| run_case(&runner, *case, options))
            .collect::<Result<Vec<_>, _>>()
    })?;

    let measured_iterations = cases.iter().map(|case| case.measured_iterations).sum();

    Ok(BenchmarkSuiteReport {
        artifact: BenchmarkArtifact {
            generated_by: "crates/iris-qjs/src/bin/qjs-bench.rs",
            kind: "file",
            path: options.artifact_path.clone(),
        },
        cases,
        created_at: created_at(),
        metadata: BenchmarkMetadata {
            app: BenchmarkAppMetadata {
                name: "IrisBench",
                version: "0.0.1",
            },
            build: BenchmarkBuildMetadata {
                commit: options.commit.clone(),
                compiler: "rquickjs 0.12.0 / QuickJS-NG",
                mode: options.mode.clone(),
                source: options.source.clone(),
                source_hash: format!("fnv1a64:{:016x}", stable_source_hash()),
                transform: "none; host-side QuickJS backend microbenchmark",
            },
            platform: BenchmarkPlatformMetadata {
                device: options.device.clone(),
                os: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
                version: format!("rust-{}", env!("CARGO_PKG_VERSION")),
            },
            react_native: BenchmarkReactNativeMetadata { version: "0.85.0" },
            runtime: BenchmarkRuntimeMetadata {
                fabric: false,
                hermes: false,
                hermes_version: "not-applicable",
                js_engine: "iris-qjs",
                new_architecture: false,
                runtime_backend: BACKEND_NAME,
                turbo_module_proxy: false,
            },
        },
        schema_version: BENCHMARK_SCHEMA_VERSION,
        suite: BenchmarkSuite {
            id: "iris-qjs-backend-microbenchmark",
            name: "Iris QuickJS Backend Microbenchmark",
        },
        summary: BenchmarkSummary {
            case_count: BENCHMARK_CASES.len(),
            measured_iterations,
            total_elapsed_ms: round_milliseconds(started_at.elapsed().as_secs_f64() * 1_000.0),
        },
    })
}

/// Runs the QuickJS benchmark and returns a compact JSON artifact.
///
/// This is used by the Android FFI wrapper crate so Android links one Rust
/// static library instead of multiple `cxx` static libraries.
pub fn run_quickjs_benchmark_json(
    artifact_path: &str,
    commit: &str,
    device: &str,
    mode: &str,
    source: &str,
    warmup_iterations: u32,
    measured_iterations: u32,
) -> Result<String, String> {
    let report = run_quickjs_benchmark(&BenchmarkOptions {
        artifact_path: artifact_path.to_owned(),
        commit: commit.to_owned(),
        device: device.to_owned(),
        measured_iterations: usize::try_from(measured_iterations)
            .map_err(|error| format!("measured_iterations is out of range: {error}"))?,
        mode: mode.to_owned(),
        source: source.to_owned(),
        warmup_iterations: usize::try_from(warmup_iterations)
            .map_err(|error| format!("warmup_iterations is out of range: {error}"))?,
    })?;
    serde_json::to_string(&report)
        .map_err(|error| format!("failed to encode QuickJS benchmark report: {error}"))
}

fn run_case<'js>(
    runner: &Function<'js>,
    case: BenchmarkCaseDefinition,
    options: &BenchmarkOptions,
) -> Result<BenchmarkCaseReport, String> {
    for _ in 0..options.warmup_iterations {
        run_case_once(runner, case.id)?;
    }

    let mut samples = Vec::with_capacity(options.measured_iterations);
    let mut last_run = None;

    for _ in 0..options.measured_iterations {
        let started_at = Instant::now();
        let run_value = run_case_once(runner, case.id)?;
        samples.push(started_at.elapsed().as_secs_f64() * 1_000.0);
        last_run = Some(run_value);
    }

    let last_run = last_run.ok_or_else(|| format!("{} did not run", case.id))?;

    Ok(BenchmarkCaseReport {
        checksum: last_run.checksum,
        description: case.description,
        detail: last_run.detail,
        id: case.id,
        label: case.label,
        measured_iterations: options.measured_iterations,
        stats: summarize_samples(&samples),
        unit: "ms",
        warmup_iterations: options.warmup_iterations,
    })
}

fn run_case_once<'js>(
    runner: &Function<'js>,
    case_id: &'static str,
) -> Result<BenchmarkRunValue, String> {
    let json: String = runner
        .call((case_id,))
        .map_err(|error| format!("failed to run QuickJS case {case_id}: {error}"))?;
    serde_json::from_str(&json)
        .map_err(|error| format!("failed to decode QuickJS case {case_id} result: {error}"))
}

fn summarize_samples(samples: &[f64]) -> BenchmarkStats {
    let mut sorted_samples = samples.to_vec();
    sorted_samples.sort_by(f64::total_cmp);
    let total = sorted_samples.iter().sum::<f64>();

    BenchmarkStats {
        max: round_milliseconds(*sorted_samples.last().unwrap_or(&0.0)),
        mean: round_milliseconds(total / samples.len().max(1) as f64),
        min: round_milliseconds(*sorted_samples.first().unwrap_or(&0.0)),
        p50: round_milliseconds(percentile(&sorted_samples, 50.0)),
        p95: round_milliseconds(percentile(&sorted_samples, 95.0)),
        samples: samples
            .iter()
            .map(|sample| round_milliseconds(*sample))
            .collect(),
    }
}

fn percentile(sorted_samples: &[f64], percentile_value: f64) -> f64 {
    if sorted_samples.is_empty() {
        return 0.0;
    }

    let rank = ((percentile_value / 100.0) * sorted_samples.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted_samples.len() - 1);
    sorted_samples[index]
}

fn round_milliseconds(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

fn created_at() -> String {
    let milliseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    format!("unix-ms:{milliseconds}")
}

fn stable_source_hash() -> u64 {
    QUICKJS_BENCHMARK_SOURCE
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_description_names_quickjs_experiment() {
        assert!(backend_description().starts_with(BACKEND_NAME));
    }

    #[test]
    fn quickjs_benchmark_smoke_matches_expected_checksums() {
        let report = run_quickjs_benchmark(&BenchmarkOptions {
            measured_iterations: 1,
            warmup_iterations: 0,
            ..BenchmarkOptions::default()
        })
        .expect("quickjs benchmark should run");

        let checksums = report
            .cases
            .iter()
            .map(|case| (case.id, case.checksum.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();

        assert_eq!(checksums["js-compute"], Value::from(7.307));
        assert_eq!(checksums["json-round-trip"], Value::from(127_984_000));
        assert_eq!(checksums["object-traversal"], Value::from(15_661_082));
        assert_eq!(checksums["typed-array-copy"], Value::from(12_338));
    }
}
