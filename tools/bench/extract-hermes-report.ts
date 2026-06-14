import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import type { BenchmarkSuiteReport } from "../../apps/rn-bench/src/benchmarks/types";

const marker = "IRIS_BENCHMARK_ARTIFACT";
const chunkMarker = "IRIS_BENCHMARK_ARTIFACT_CHUNK";
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

function readOptionalArgs(name: string, fallback: string[]) {
  const values = readArgs(name);
  return values.length > 0 ? values : fallback;
}

function extractJsonPayload(line: string) {
  if (line.includes(chunkMarker)) {
    return null;
  }

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

function extractJsonPayloadChunk(line: string) {
  const markerIndex = line.indexOf(chunkMarker);

  if (markerIndex === -1) {
    return null;
  }

  const payload = line.slice(markerIndex + chunkMarker.length).trimStart();
  const match = payload.match(/^(\d+)\/(\d+) (.*)$/);

  if (!match) {
    return null;
  }

  return {
    chunk: match[3],
    index: Number(match[1]),
    total: Number(match[2]),
  };
}

function collectJsonPayloads(logText: string) {
  const payloads: string[] = [];
  let chunks: string[] = [];
  let expectedTotal = 0;

  for (const line of logText.split(/\r?\n/)) {
    const chunk = extractJsonPayloadChunk(line);

    if (chunk != null) {
      if (chunk.index === 1) {
        chunks = [];
        expectedTotal = chunk.total;
      }

      if (chunk.total === expectedTotal && chunk.index === chunks.length + 1) {
        chunks.push(chunk.chunk);
      } else {
        chunks = [];
        expectedTotal = 0;
      }

      if (expectedTotal > 0 && chunks.length === expectedTotal) {
        payloads.push(chunks.join(""));
        chunks = [];
        expectedTotal = 0;
      }

      continue;
    }

    const payload = extractJsonPayload(line);

    if (payload != null) {
      payloads.push(payload);
    }
  }

  return payloads;
}

function parseReport(logText: string, suiteIds: string[]): BenchmarkSuiteReport {
  const payloads = collectJsonPayloads(logText);

  if (payloads.length === 0) {
    throw new Error(`No ${marker} JSON payload was found.`);
  }

  const reports = payloads
    .map((payload) => {
      try {
        return JSON.parse(payload) as BenchmarkSuiteReport;
      } catch {
        return null;
      }
    })
    .filter((report): report is BenchmarkSuiteReport => report != null);
  const matchingReports = reports.filter((report) => suiteIds.includes(report.suite.id));

  if (matchingReports.length === 0) {
    const availableSuiteIds = reports.map((report) => report.suite.id).join(", ");

    throw new Error(
      `No ${marker} payload matched suite ${suiteIds.join(", ")}. Available suites: ${availableSuiteIds}`,
    );
  }

  return matchingReports[matchingReports.length - 1];
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
  suiteIds: string[];
};

function validateReport(report: BenchmarkSuiteReport, options: ValidationOptions) {
  if (report.schemaVersion !== "iris.benchmark.v1") {
    throw new Error(`Unexpected benchmark schema: ${report.schemaVersion}`);
  }

  if (!options.suiteIds.includes(report.suite.id)) {
    throw new Error(
      `Unexpected benchmark suite: ${report.suite.id}. Expected one of: ${options.suiteIds.join(", ")}`,
    );
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
  suiteIds: readOptionalArgs("--suite-id", ["rn-hermes-js-baseline"]),
};
const report = parseReport(readFileSync(inputPath, "utf8"), validationOptions.suiteIds);

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

console.log(`Benchmark artifact: ${outputPath}`);
for (const result of outputReport.cases) {
  console.log(
    `${result.id}: p50=${result.stats.p50}${result.unit} p95=${result.stats.p95}${result.unit}`,
  );
}
