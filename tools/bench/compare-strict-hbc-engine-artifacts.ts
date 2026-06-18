import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, resolve } from "node:path";

const root = resolve(import.meta.dir, "../..");
const defaultOutputPath = "artifacts/bench/strict-hbc-artifact-comparison.json";

type Stats = {
  p50: number;
  p95: number;
};

type StrictHbcCase = {
  id: string;
  checksum: {
    matches: boolean;
  };
  stability?: "stable" | "unstable";
  unstableReasons?: string[];
  hermes: Stats;
  iris: Stats;
  p50IrisOverHermes: number | null;
  p95IrisOverHermes: number | null;
};

type StrictHbcReport = {
  schemaVersion: string;
  generatedBy: string;
  createdAt: string;
  cases: StrictHbcCase[];
};

type RepeatMetricSummary = {
  median: number | null;
};

type StrictHbcRepeatCase = {
  id: string;
  checksumStable: boolean;
  stability: "stable" | "unstable";
  unstableReasons: string[];
  hermesP50: RepeatMetricSummary;
  hermesP95: RepeatMetricSummary;
  irisP50: RepeatMetricSummary;
  irisP95: RepeatMetricSummary;
  p50IrisOverHermes: RepeatMetricSummary;
  p95IrisOverHermes: RepeatMetricSummary;
};

type StrictHbcRepeatSummary = {
  schemaVersion: string;
  generatedBy: string;
  createdAt: string;
  cases: StrictHbcRepeatCase[];
};

type Delta = {
  baseline: number | null;
  candidate: number | null;
  delta: number | null;
  deltaPercent: number | null;
};

type ComparedCase = {
  id: string;
  checksumMatches: {
    baseline: boolean;
    candidate: boolean;
  };
  stability: {
    baseline: "stable" | "unstable" | "not-reported";
    candidate: "stable" | "unstable" | "not-reported";
  };
  unstableReasons: {
    baseline: string[];
    candidate: string[];
  };
  hermesP50: Delta;
  hermesP95: Delta;
  irisP50: Delta;
  irisP95: Delta;
  p50IrisOverHermes: Delta;
  p95IrisOverHermes: Delta;
  irisP50Verdict: "faster" | "slower" | "within-threshold" | "unknown";
};

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

function requireArg(name: string) {
  const value = readArg(name);
  if (value == null || value.length === 0) {
    throw new Error(`${name} is required.`);
  }
  return value;
}

function readNumberArg(name: string, fallback: number) {
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

function readIntegerArg(name: string, fallback: number) {
  const rawValue = readArg(name);
  if (rawValue == null) {
    return fallback;
  }

  const value = Number(rawValue);
  if (!Number.isInteger(value) || value < 0) {
    throw new Error(`${name} must be a non-negative integer.`);
  }
  return value;
}

function readArtifact(path: string): StrictHbcReport {
  if (!existsSync(path)) {
    throw new Error(`strict HBC artifact does not exist: ${path}`);
  }

  const report = JSON.parse(readFileSync(path, "utf8")) as StrictHbcReport | StrictHbcRepeatSummary;
  if (!Array.isArray(report.cases)) {
    throw new Error(`strict HBC artifact has no cases array: ${path}`);
  }
  if (report.schemaVersion === "iris.benchmark.strict-hbc-engine-comparison-repeat-summary.v1") {
    return normalizeRepeatSummary(report, path);
  }
  return report;
}

function requireMedian(
  metric: RepeatMetricSummary,
  path: string,
  caseId: string,
  metricName: string,
) {
  if (metric.median == null) {
    throw new Error(`${path} case ${caseId} has no median for ${metricName}.`);
  }
  return metric.median;
}

function normalizeRepeatSummary(report: StrictHbcRepeatSummary, path: string): StrictHbcReport {
  return {
    schemaVersion: report.schemaVersion,
    generatedBy: report.generatedBy,
    createdAt: report.createdAt,
    cases: report.cases.map((entry) => ({
      id: entry.id,
      checksum: {
        matches: entry.checksumStable,
      },
      stability: entry.stability ?? (entry.checksumStable ? "stable" : "unstable"),
      unstableReasons: entry.unstableReasons ?? [],
      hermes: {
        p50: requireMedian(entry.hermesP50, path, entry.id, "hermesP50"),
        p95: requireMedian(entry.hermesP95, path, entry.id, "hermesP95"),
      },
      iris: {
        p50: requireMedian(entry.irisP50, path, entry.id, "irisP50"),
        p95: requireMedian(entry.irisP95, path, entry.id, "irisP95"),
      },
      p50IrisOverHermes: requireMedian(
        entry.p50IrisOverHermes,
        path,
        entry.id,
        "p50IrisOverHermes",
      ),
      p95IrisOverHermes: requireMedian(
        entry.p95IrisOverHermes,
        path,
        entry.id,
        "p95IrisOverHermes",
      ),
    })),
  };
}

function relativePath(path: string) {
  return path.replace(`${root}/`, "");
}

function round(value: number) {
  return Number(value.toFixed(6));
}

function delta(baseline: number | null, candidate: number | null): Delta {
  if (baseline == null || candidate == null) {
    return {
      baseline,
      candidate,
      delta: null,
      deltaPercent: null,
    };
  }

  const rawDelta = candidate - baseline;
  return {
    baseline: round(baseline),
    candidate: round(candidate),
    delta: round(rawDelta),
    deltaPercent: baseline === 0 ? null : round((rawDelta / baseline) * 100),
  };
}

function indexedByCase(report: StrictHbcReport) {
  return new Map(report.cases.map((entry) => [entry.id, entry]));
}

function verdict(
  deltaPercent: number | null,
  thresholdPercent: number,
): ComparedCase["irisP50Verdict"] {
  if (deltaPercent == null) {
    return "unknown";
  }
  if (Math.abs(deltaPercent) <= thresholdPercent) {
    return "within-threshold";
  }
  return deltaPercent < 0 ? "faster" : "slower";
}

function compareCases(
  baseline: StrictHbcCase,
  candidate: StrictHbcCase,
  thresholdPercent: number,
): ComparedCase {
  const irisP50 = delta(baseline.iris.p50, candidate.iris.p50);
  return {
    id: candidate.id,
    checksumMatches: {
      baseline: baseline.checksum.matches,
      candidate: candidate.checksum.matches,
    },
    stability: {
      baseline: baseline.stability ?? "not-reported",
      candidate: candidate.stability ?? "not-reported",
    },
    unstableReasons: {
      baseline: baseline.unstableReasons ?? [],
      candidate: candidate.unstableReasons ?? [],
    },
    hermesP50: delta(baseline.hermes.p50, candidate.hermes.p50),
    hermesP95: delta(baseline.hermes.p95, candidate.hermes.p95),
    irisP50,
    irisP95: delta(baseline.iris.p95, candidate.iris.p95),
    p50IrisOverHermes: delta(baseline.p50IrisOverHermes, candidate.p50IrisOverHermes),
    p95IrisOverHermes: delta(baseline.p95IrisOverHermes, candidate.p95IrisOverHermes),
    irisP50Verdict: verdict(irisP50.deltaPercent, thresholdPercent),
  };
}

function gateFailures(
  cases: ComparedCase[],
  missingBaselineCases: string[],
  allowSlowerCases: number,
  requireStableRepeat: boolean,
) {
  const failures: string[] = [];
  if (missingBaselineCases.length > 0) {
    failures.push(`missing baseline cases: ${missingBaselineCases.join(", ")}`);
  }

  const checksumFailures = cases
    .filter((entry) => !entry.checksumMatches.baseline || !entry.checksumMatches.candidate)
    .map((entry) => entry.id);
  if (checksumFailures.length > 0) {
    failures.push(`checksum failures: ${checksumFailures.join(", ")}`);
  }

  if (requireStableRepeat) {
    const unstableBaselineCases = cases
      .filter((entry) => entry.stability.baseline === "unstable")
      .map((entry) => `${entry.id}(${entry.unstableReasons.baseline.join("|") || "unstable"})`);
    if (unstableBaselineCases.length > 0) {
      failures.push(`unstable baseline repeat cases: ${unstableBaselineCases.join(", ")}`);
    }

    const unstableCandidateCases = cases
      .filter((entry) => entry.stability.candidate === "unstable")
      .map((entry) => `${entry.id}(${entry.unstableReasons.candidate.join("|") || "unstable"})`);
    if (unstableCandidateCases.length > 0) {
      failures.push(`unstable candidate repeat cases: ${unstableCandidateCases.join(", ")}`);
    }
  }

  const slowerCases = cases.filter((entry) => entry.irisP50Verdict === "slower");
  if (slowerCases.length > allowSlowerCases) {
    failures.push(
      `slower Iris p50 cases: ${slowerCases
        .map((entry) => `${entry.id}(${entry.irisP50.deltaPercent?.toFixed(3)}%)`)
        .join(", ")}`,
    );
  }

  return failures;
}

function sortComparedCases(cases: ComparedCase[]) {
  return [...cases].sort((left, right) => {
    const leftDelta = left.irisP50.deltaPercent ?? 0;
    const rightDelta = right.irisP50.deltaPercent ?? 0;
    if (leftDelta !== rightDelta) {
      return leftDelta - rightDelta;
    }
    return left.id.localeCompare(right.id);
  });
}

function formatNumber(value: number | null, suffix = "") {
  if (value == null) {
    return "n/a";
  }
  return `${value.toFixed(3)}${suffix}`;
}

function printSummary(cases: ComparedCase[]) {
  const sortedCases = sortComparedCases(cases);
  const rows = sortedCases.map((entry) => ({
    case: entry.id,
    irisP50: `${formatNumber(entry.irisP50.baseline)} -> ${formatNumber(entry.irisP50.candidate)}`,
    irisP50Delta: formatNumber(entry.irisP50.deltaPercent, "%"),
    ratioP50:
      `${formatNumber(entry.p50IrisOverHermes.baseline, "x")} -> ` +
      `${formatNumber(entry.p50IrisOverHermes.candidate, "x")}`,
    ratioDelta: formatNumber(entry.p50IrisOverHermes.deltaPercent, "%"),
    verdict: entry.irisP50Verdict,
  }));

  console.table(rows);
}

function main() {
  const baselinePath = resolve(root, requireArg("--baseline"));
  const candidatePath = resolve(root, requireArg("--candidate"));
  const outputPath = resolve(root, readArg("--output") ?? defaultOutputPath);
  const noiseThresholdPercent = readNumberArg("--noise-threshold-percent", 2);
  const gate = process.argv.includes("--gate");
  const allowSlowerCases = readIntegerArg("--allow-slower-cases", 0);
  const requireStableRepeat = !process.argv.includes("--allow-unstable-repeat");
  const baselineReport = readArtifact(baselinePath);
  const candidateReport = readArtifact(candidatePath);
  const baselineCases = indexedByCase(baselineReport);
  const comparedCases: ComparedCase[] = [];
  const missingBaselineCases: string[] = [];

  for (const candidateCase of candidateReport.cases) {
    const baselineCase = baselineCases.get(candidateCase.id);
    if (baselineCase == null) {
      missingBaselineCases.push(candidateCase.id);
      continue;
    }
    comparedCases.push(compareCases(baselineCase, candidateCase, noiseThresholdPercent));
  }

  const report = {
    schemaVersion: "iris.benchmark.strict-hbc-artifact-comparison.v1",
    generatedBy: "tools/bench/compare-strict-hbc-engine-artifacts.ts",
    createdAt: new Date().toISOString(),
    methodology: {
      scope:
        "Compares two existing host-side strict HBC engine comparison artifacts. It does not run either engine.",
      direction:
        "Negative Iris p50 delta means the candidate artifact is faster than the baseline for Iris absolute time.",
      noiseThresholdPercent,
      gate,
      allowSlowerCases,
      requireStableRepeat,
    },
    baseline: {
      path: relativePath(baselinePath),
      label: readArg("--baseline-label") ?? basename(baselinePath, ".json"),
      createdAt: baselineReport.createdAt,
      generatedBy: baselineReport.generatedBy,
    },
    candidate: {
      path: relativePath(candidatePath),
      label: readArg("--candidate-label") ?? basename(candidatePath, ".json"),
      createdAt: candidateReport.createdAt,
      generatedBy: candidateReport.generatedBy,
    },
    comparedCaseCount: comparedCases.length,
    missingBaselineCases,
    cases: sortComparedCases(comparedCases),
  };

  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, `${JSON.stringify(report, null, 2)}\n`);
  printSummary(comparedCases);
  if (gate) {
    const failures = gateFailures(
      comparedCases,
      missingBaselineCases,
      allowSlowerCases,
      requireStableRepeat,
    );
    if (failures.length > 0) {
      console.error(`Strict HBC comparison gate failed:\n- ${failures.join("\n- ")}`);
      process.exitCode = 1;
    } else {
      console.log("Strict HBC comparison gate passed.");
    }
  }
  console.log(`Strict HBC artifact comparison: ${outputPath}`);
}

main();
