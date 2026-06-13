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

function readArgs(name: string) {
  const prefix = `${name}=`;
  return process.argv
    .filter((arg) => arg.startsWith(prefix))
    .map((arg) => arg.slice(prefix.length));
}

function extractJsonPayload(line: string) {
  const markerIndex = line.indexOf(marker);
  if (markerIndex === -1) {
    return null;
  }

  const payload = line.slice(markerIndex + marker.length);
  const jsonStart = payload.indexOf("{");
  const jsonEnd = payload.lastIndexOf("}");

  if (jsonStart === -1 || jsonEnd === -1 || jsonEnd <= jsonStart) {
    return null;
  }

  return payload.slice(jsonStart, jsonEnd + 1);
}

function parseReport(logText: string): BenchmarkSuiteReport {
  const candidates = logText
    .split(/\r?\n/)
    .map((line) => extractJsonPayload(line))
    .filter((payload): payload is string => payload != null);

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

type ValidationOptions = {
  allowNonHermes: boolean;
  requiredCases: string[];
  requireNewArchitecture: boolean;
  requireRelease: boolean;
  requireTurboModuleProxy: boolean;
};

function validateReport(report: BenchmarkSuiteReport, options: ValidationOptions) {
  if (report.schemaVersion !== "iris.benchmark.v1") {
    throw new Error(`Unexpected benchmark schema: ${report.schemaVersion}`);
  }

  if (report.suite.id !== "rn-hermes-js-baseline") {
    throw new Error(`Unexpected benchmark suite: ${report.suite.id}`);
  }

  if (!options.allowNonHermes && !report.metadata.runtime.hermes) {
    throw new Error("Report is not a Hermes runtime report.");
  }

  if (options.requireRelease && report.metadata.build.mode !== "release") {
    throw new Error(`Report is not a release build report: ${report.metadata.build.mode}`);
  }

  if (options.requireNewArchitecture && !report.metadata.runtime.newArchitecture) {
    throw new Error("Report does not have New Architecture enabled.");
  }

  if (options.requireTurboModuleProxy && !report.metadata.runtime.turboModuleProxy) {
    throw new Error("Report does not have the TurboModule proxy enabled.");
  }

  if (report.cases.length === 0) {
    throw new Error("Report must include at least one benchmark case.");
  }

  const caseIds = new Set(report.cases.map((benchmarkCase) => benchmarkCase.id));
  for (const requiredCase of options.requiredCases) {
    if (!caseIds.has(requiredCase)) {
      throw new Error(`Report is missing required benchmark case: ${requiredCase}`);
    }
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
const validationOptions: ValidationOptions = {
  allowNonHermes: hasArg("--allow-non-hermes"),
  requiredCases: readArgs("--require-case"),
  requireNewArchitecture: hasArg("--require-new-architecture"),
  requireRelease: hasArg("--require-release"),
  requireTurboModuleProxy: hasArg("--require-turbo-module-proxy"),
};
const report = parseReport(readFileSync(inputPath, "utf8"));

validateReport(report, validationOptions);

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
