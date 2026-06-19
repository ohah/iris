import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

const root = resolve(import.meta.dir, "../..");
const defaultRunOutputDir = "artifacts/bench/strict-hbc-repeat";
const defaultSummaryOutputPath = "artifacts/bench/strict-hbc-engine-comparison-repeat-summary.json";

type Stats = {
  p50: number;
  p95: number;
};

type StrictHbcCase = {
  id: string;
  checksum: {
    hermes: boolean | number | string | null;
    iris: boolean | number | string | null;
    matches: boolean;
  };
  hermes: Stats;
  iris: Stats;
  p50IrisOverHermes: number | null;
  p95IrisOverHermes: number | null;
};

type StrictHbcReport = {
  schemaVersion: string;
  generatedBy: string;
  createdAt: string;
  warmupIterations: number;
  measuredIterations: number;
  rounds: number;
  engineOrder: string;
  totalMeasuredIterations: number;
  cases: StrictHbcCase[];
};

type MetricSummary = {
  absoluteSpreadMs: number | null;
  max: number | null;
  mean: number | null;
  median: number | null;
  min: number | null;
  relativeSpreadPercent: number | null;
  values: Array<number | null>;
};

type CaseRun = {
  artifactPath: string;
  checksum: StrictHbcCase["checksum"];
  hermesP50: number;
  hermesP95: number;
  irisP50: number;
  irisP95: number;
  p50IrisOverHermes: number | null;
  p95IrisOverHermes: number | null;
};

type CaseSummary = {
  id: string;
  checksumStable: boolean;
  stability: "stable" | "unstable";
  unstableReasons: string[];
  hermesP50: MetricSummary;
  hermesP95: MetricSummary;
  irisP50: MetricSummary;
  irisP95: MetricSummary;
  p50IrisOverHermes: MetricSummary;
  p95IrisOverHermes: MetricSummary;
  runs: CaseRun[];
};

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

function readNumberArg(name: string, fallback: number, allowZero = false) {
  const rawValue = readArg(name);
  if (rawValue == null) {
    return fallback;
  }

  const value = Number(rawValue);
  if (!Number.isInteger(value) || value < 0 || (!allowZero && value === 0)) {
    throw new Error(`${name} must be ${allowZero ? "a non-negative" : "a positive"} integer.`);
  }
  return value;
}

function readFiniteNumberArg(name: string, fallback: number) {
  const rawValue = readArg(name);
  if (rawValue == null) {
    return fallback;
  }

  const value = Number(rawValue);
  if (!Number.isFinite(value) || value < 0) {
    throw new Error(`${name} must be a non-negative finite number.`);
  }
  return value;
}

function relativePath(path: string) {
  return path.replace(`${root}/`, "");
}

function shellLine(command: string, args: string[]) {
  return [command, ...args.map((arg) => (arg.includes(" ") ? JSON.stringify(arg) : arg))].join(" ");
}

function run(command: string, args: string[]) {
  console.log(`$ ${shellLine(command, args)}`);
  execFileSync(command, args, {
    cwd: root,
    stdio: "inherit",
  });
}

function round(value: number) {
  return Number(value.toFixed(6));
}

function readStrictHbcReport(path: string): StrictHbcReport {
  if (!existsSync(path)) {
    throw new Error(`strict HBC comparison artifact does not exist: ${path}`);
  }
  const report = JSON.parse(readFileSync(path, "utf8")) as StrictHbcReport;
  if (!Array.isArray(report.cases)) {
    throw new Error(`strict HBC comparison artifact has no cases array: ${path}`);
  }
  return report;
}

function repeatRunOutputPath(runOutputDir: string, runIndex: number, runCount: number) {
  const width = String(runCount).length;
  const runLabel = String(runIndex + 1).padStart(width, "0");
  return resolve(runOutputDir, `strict-hbc-engine-comparison-run-${runLabel}.json`);
}

function comparisonArgs() {
  if (readArg("--output") != null) {
    throw new Error(
      "Use --summary-output and --run-output-dir with repeat comparison; per-run --output is managed by this script.",
    );
  }

  return process.argv.slice(2).filter((arg) => {
    return (
      !arg.startsWith("--runs=") &&
      !arg.startsWith("--discard-initial-runs=") &&
      !arg.startsWith("--summary-output=") &&
      !arg.startsWith("--run-output-dir=") &&
      !arg.startsWith("--max-spread-percent=") &&
      !arg.startsWith("--max-absolute-spread-ms=") &&
      arg !== "--reuse-existing"
    );
  });
}

function caseMap(report: StrictHbcReport) {
  return new Map(report.cases.map((entry) => [entry.id, entry]));
}

function checksumKey(checksum: StrictHbcCase["checksum"]) {
  return JSON.stringify({
    hermes: checksum.hermes,
    iris: checksum.iris,
    matches: checksum.matches,
  });
}

function summarizeMetric(values: Array<number | null>): MetricSummary {
  const numericValues = values.filter((value): value is number => value != null);
  if (numericValues.length === 0) {
    return {
      absoluteSpreadMs: null,
      max: null,
      mean: null,
      median: null,
      min: null,
      relativeSpreadPercent: null,
      values,
    };
  }

  const sortedValues = [...numericValues].sort((left, right) => left - right);
  const min = sortedValues[0];
  const max = sortedValues[sortedValues.length - 1];
  const median =
    sortedValues.length % 2 === 0
      ? (sortedValues[sortedValues.length / 2 - 1] + sortedValues[sortedValues.length / 2]) / 2
      : sortedValues[Math.floor(sortedValues.length / 2)];
  const mean = sortedValues.reduce((sum, value) => sum + value, 0) / sortedValues.length;

  return {
    absoluteSpreadMs: round(max - min),
    max: round(max),
    mean: round(mean),
    median: round(median),
    min: round(min),
    relativeSpreadPercent: median === 0 ? null : round(((max - min) / median) * 100),
    values: values.map((value) => (value == null ? null : round(value))),
  };
}

function summarizeCase(
  id: string,
  runs: CaseRun[],
  maxSpreadPercent: number,
  maxAbsoluteSpreadMs: number,
): CaseSummary {
  const checksumKeys = new Set(runs.map((runEntry) => checksumKey(runEntry.checksum)));
  const checksumStable =
    checksumKeys.size === 1 && runs.every((runEntry) => runEntry.checksum.matches);
  const irisP50 = summarizeMetric(runs.map((runEntry) => runEntry.irisP50));
  const unstableReasons: string[] = [];
  if (!checksumStable) {
    unstableReasons.push("checksum");
  }
  const exceedsRelativeSpread =
    irisP50.relativeSpreadPercent != null && irisP50.relativeSpreadPercent > maxSpreadPercent;
  const exceedsAbsoluteSpread =
    maxAbsoluteSpreadMs > 0 &&
    irisP50.absoluteSpreadMs != null &&
    irisP50.absoluteSpreadMs > maxAbsoluteSpreadMs;
  if (exceedsRelativeSpread && (maxAbsoluteSpreadMs <= 0 || exceedsAbsoluteSpread)) {
    unstableReasons.push("iris-p50-spread");
  }

  return {
    id,
    checksumStable,
    stability: unstableReasons.length === 0 ? "stable" : "unstable",
    unstableReasons,
    hermesP50: summarizeMetric(runs.map((runEntry) => runEntry.hermesP50)),
    hermesP95: summarizeMetric(runs.map((runEntry) => runEntry.hermesP95)),
    irisP50,
    irisP95: summarizeMetric(runs.map((runEntry) => runEntry.irisP95)),
    p50IrisOverHermes: summarizeMetric(runs.map((runEntry) => runEntry.p50IrisOverHermes)),
    p95IrisOverHermes: summarizeMetric(runs.map((runEntry) => runEntry.p95IrisOverHermes)),
    runs,
  };
}

function summarizeCases(
  artifacts: Array<{ path: string; report: StrictHbcReport }>,
  maxSpreadPercent: number,
  maxAbsoluteSpreadMs: number,
) {
  const firstArtifact = artifacts[0];
  if (firstArtifact == null) {
    throw new Error("at least one strict HBC comparison artifact is required.");
  }
  const caseIds = firstArtifact.report.cases.map((entry) => entry.id);
  const summaries: CaseSummary[] = [];

  for (const id of caseIds) {
    const runs: CaseRun[] = [];
    for (const artifact of artifacts) {
      const entry = caseMap(artifact.report).get(id);
      if (entry == null) {
        throw new Error(`${relativePath(artifact.path)} is missing case ${id}.`);
      }
      runs.push({
        artifactPath: relativePath(artifact.path),
        checksum: entry.checksum,
        hermesP50: entry.hermes.p50,
        hermesP95: entry.hermes.p95,
        irisP50: entry.iris.p50,
        irisP95: entry.iris.p95,
        p50IrisOverHermes: entry.p50IrisOverHermes,
        p95IrisOverHermes: entry.p95IrisOverHermes,
      });
    }
    summaries.push(summarizeCase(id, runs, maxSpreadPercent, maxAbsoluteSpreadMs));
  }

  return summaries;
}

function printSummary(cases: CaseSummary[]) {
  const rows = cases.map((entry) => ({
    case: entry.id,
    irisP50Median: entry.irisP50.median?.toFixed(3) ?? "n/a",
    irisP50Spread:
      entry.irisP50.relativeSpreadPercent == null
        ? "n/a"
        : `${entry.irisP50.relativeSpreadPercent.toFixed(3)}%`,
    irisP50AbsSpread:
      entry.irisP50.absoluteSpreadMs == null
        ? "n/a"
        : `${entry.irisP50.absoluteSpreadMs.toFixed(3)}ms`,
    ratioMedian:
      entry.p50IrisOverHermes.median == null
        ? "n/a"
        : `${entry.p50IrisOverHermes.median.toFixed(3)}x`,
    stability: entry.stability,
  }));
  console.table(rows);
}

function main() {
  const runCount = readNumberArg("--runs", 4);
  const discardInitialRuns = readNumberArg("--discard-initial-runs", 1, true);
  if (discardInitialRuns >= runCount) {
    throw new Error("--discard-initial-runs must be smaller than --runs.");
  }
  if (runCount - discardInitialRuns < 2) {
    throw new Error("repeat summary requires at least two summarized runs after discard.");
  }
  const maxSpreadPercent = readFiniteNumberArg("--max-spread-percent", 5);
  const maxAbsoluteSpreadMs = readFiniteNumberArg("--max-absolute-spread-ms", 0);
  const summaryOutputPath = resolve(root, readArg("--summary-output") ?? defaultSummaryOutputPath);
  const runOutputDir = resolve(root, readArg("--run-output-dir") ?? defaultRunOutputDir);
  const reuseExisting = process.argv.includes("--reuse-existing");
  const passThroughArgs = comparisonArgs();
  const artifacts: Array<{ path: string; report: StrictHbcReport }> = [];

  mkdirSync(runOutputDir, { recursive: true });
  for (let runIndex = 0; runIndex < runCount; runIndex += 1) {
    const outputPath = repeatRunOutputPath(runOutputDir, runIndex, runCount);
    if (!reuseExisting) {
      run("bun", [
        "run",
        "tools/bench/run-strict-hbc-engine-comparison.ts",
        ...passThroughArgs,
        `--output=${outputPath}`,
      ]);
    }
    artifacts.push({
      path: outputPath,
      report: readStrictHbcReport(outputPath),
    });
  }

  const summarizedArtifacts = artifacts.slice(discardInitialRuns);
  const cases = summarizeCases(summarizedArtifacts, maxSpreadPercent, maxAbsoluteSpreadMs);
  const report = {
    schemaVersion: "iris.benchmark.strict-hbc-engine-comparison-repeat-summary.v1",
    generatedBy: "tools/bench/run-strict-hbc-engine-comparison-repeat.ts",
    createdAt: new Date().toISOString(),
    methodology: {
      scope:
        "Runs the host-side strict HBC engine comparison repeatedly and summarizes run-to-run stability. It is not an additional engine benchmark workload.",
      stableWhen:
        maxAbsoluteSpreadMs > 0
          ? "All checksums match and Iris p50 relative spread is at or below maxSpreadPercent, or Iris p50 absolute spread is at or below maxAbsoluteSpreadMs."
          : "All checksums match and Iris p50 relative spread is at or below maxSpreadPercent.",
      maxSpreadPercent,
      maxAbsoluteSpreadMs,
      discardInitialRuns,
      reuseExisting,
    },
    totalRunCount: runCount,
    summarizedRunCount: summarizedArtifacts.length,
    discardInitialRuns,
    runOutputDir: relativePath(runOutputDir),
    runArtifacts: artifacts.map((artifact) => relativePath(artifact.path)),
    summarizedRunArtifacts: summarizedArtifacts.map((artifact) => relativePath(artifact.path)),
    comparisonArgs: passThroughArgs,
    summaryOutputPath: relativePath(summaryOutputPath),
    cases,
  };

  mkdirSync(dirname(summaryOutputPath), { recursive: true });
  writeFileSync(summaryOutputPath, `${JSON.stringify(report, null, 2)}\n`);
  printSummary(cases);
  console.log(`Strict HBC repeat summary: ${summaryOutputPath}`);
}

main();
