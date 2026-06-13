import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import type { BenchmarkSuiteReport } from "../../apps/rn-bench/src/benchmarks/types";

const marker = "IRIS_BENCHMARK_ARTIFACT";
const root = resolve(import.meta.dir, "../..");

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

function hasArg(name: string) {
  return process.argv.includes(name);
}

function parseReport(logText: string): BenchmarkSuiteReport {
  const candidates = logText
    .split(/\r?\n/)
    .filter((line) => line.includes(marker))
    .map((line) => line.slice(line.indexOf(marker) + marker.length).trim())
    .filter((line) => line.startsWith("{"));

  if (candidates.length === 0) {
    throw new Error(`No ${marker} JSON payload was found.`);
  }

  const latest = candidates[candidates.length - 1];
  return JSON.parse(latest) as BenchmarkSuiteReport;
}

function assertNumber(value: unknown, fieldName: string) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`${fieldName} must be a finite number.`);
  }
}

function validateReport(report: BenchmarkSuiteReport, allowNonHermes: boolean) {
  if (report.schemaVersion !== "iris.benchmark.v1") {
    throw new Error(`Unexpected benchmark schema: ${report.schemaVersion}`);
  }

  if (report.suite.id !== "rn-hermes-js-baseline") {
    throw new Error(`Unexpected benchmark suite: ${report.suite.id}`);
  }

  if (!allowNonHermes && !report.metadata.runtime.hermes) {
    throw new Error("Report is not a Hermes runtime report.");
  }

  if (report.cases.length === 0) {
    throw new Error("Report must include at least one benchmark case.");
  }

  for (const benchmarkCase of report.cases) {
    if (benchmarkCase.unit !== "ms") {
      throw new Error(`${benchmarkCase.id} has unexpected unit: ${benchmarkCase.unit}`);
    }

    if (benchmarkCase.stats.samples.length !== benchmarkCase.measuredIterations) {
      throw new Error(`${benchmarkCase.id} sample count does not match measuredIterations.`);
    }

    assertNumber(benchmarkCase.stats.min, `${benchmarkCase.id}.stats.min`);
    assertNumber(benchmarkCase.stats.max, `${benchmarkCase.id}.stats.max`);
    assertNumber(benchmarkCase.stats.mean, `${benchmarkCase.id}.stats.mean`);
    assertNumber(benchmarkCase.stats.p50, `${benchmarkCase.id}.stats.p50`);
    assertNumber(benchmarkCase.stats.p95, `${benchmarkCase.id}.stats.p95`);
  }
}

const inputPath = resolve(root, readArg("--input") ?? "artifacts/bench/metro-hermes.log");
const outputPath = resolve(root, readArg("--output") ?? "artifacts/bench/hermes-baseline.json");
const allowNonHermes = hasArg("--allow-non-hermes");
const report = parseReport(readFileSync(inputPath, "utf8"));

validateReport(report, allowNonHermes);

const outputReport: BenchmarkSuiteReport = {
  ...report,
  artifact: {
    generatedBy: "tools/bench/extract-hermes-report.ts",
    kind: "file",
    path: outputPath.replace(`${root}/`, ""),
  },
};

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(outputReport, null, 2)}\n`);

console.log(`Hermes benchmark artifact: ${outputPath}`);
for (const result of outputReport.cases) {
  console.log(
    `${result.id}: p50=${result.stats.p50}${result.unit} p95=${result.stats.p95}${result.unit}`,
  );
}
