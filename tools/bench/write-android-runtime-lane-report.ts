import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import {
  allBenchmarkLanes,
  benchmarkLane,
  type BenchmarkLane,
  type BenchmarkLaneId,
} from "./runtime-lanes";

const root = resolve(import.meta.dir, "../..");
const defaultOutputPath = "artifacts/bench/android-runtime-lanes-report.json";
const baselineLaneId: BenchmarkLaneId = "hermes-baseline";

type CaseMetricSummary = {
  mean: number;
};

type RepeatedCaseSummary = {
  checksum?: number | string;
  id: string;
  label: string;
  p50: CaseMetricSummary;
  p95: CaseMetricSummary;
  unit: string;
};

type RepeatedBenchmarkSummary = {
  cases: RepeatedCaseSummary[];
  createdAt: string;
  generatedBy: string;
  metadata: unknown;
  runCount: number;
  schemaVersion: "iris.benchmark.repeated.v1";
  sourceReports: string[];
  suite: {
    id: string;
    name: string;
  };
};

type BridgeComparisonClass = "format-shift-diagnostic" | "same-method-surface" | "same-payload";

type LaneMeasurement = {
  cases: Array<{
    checksum?: number | string;
    id: string;
    label: string;
    p50Mean: number;
    p95Mean: number;
    unit: string;
  }>;
  metadata: unknown;
  runCount: number;
  sourceReports: string[];
  suite: RepeatedBenchmarkSummary["suite"];
  summaryPath: string;
};

type RuntimeLaneReport = {
  baselineLaneId: BenchmarkLaneId;
  bridgeComparisons: Array<{
    cases: Array<{
      candidateId: string;
      comparisonClass: BridgeComparisonClass;
      label: string;
      p50: {
        candidate: number;
        candidateOverReference: number;
        reference: number;
      };
      p95: {
        candidate: number;
        candidateOverReference: number;
        reference: number;
      };
      referenceId: string;
      unit: string;
    }>;
    laneId: BenchmarkLaneId;
    reason: string;
  }>;
  comparisons: Array<{
    baselineLaneId: BenchmarkLaneId;
    candidateLaneId: BenchmarkLaneId;
    cases: Array<{
      id: string;
      label: string;
      p50: {
        baseline: number;
        candidate: number;
        candidateOverBaseline: number;
      };
      p95: {
        baseline: number;
        candidate: number;
        candidateOverBaseline: number;
      };
      unit: string;
    }>;
    ratioAllowed: boolean;
    reason: string;
    strictComparable: boolean;
  }>;
  createdAt: string;
  generatedBy: string;
  lanes: Array<{
    appId?: string;
    comparisonReason: string;
    currentMeasurement: string;
    engine: BenchmarkLane["engine"];
    id: BenchmarkLaneId;
    label: string;
    measurement: LaneMeasurement | null;
    measurementStatus: "measured" | "missing";
    objective: string;
    requiredCapabilities: string[];
    runtimeBackend: string;
    status: BenchmarkLane["status"];
    strictComparableWithBaseline: boolean;
    suiteId?: string;
    summaryPath?: string;
  }>;
  schemaVersion: "iris.benchmark.android-runtime-lanes.v1";
};

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

function hasArg(name: string) {
  return process.argv.includes(name);
}

function relativePath(path: string) {
  return path.replace(`${root}/`, "");
}

function ratio(numerator: number, denominator: number) {
  return Number((numerator / denominator).toFixed(4));
}

function resolveFromRoot(path: string) {
  return resolve(root, path);
}

function readSummary(path: string, lane: BenchmarkLane) {
  const summaryPath = resolveFromRoot(path);
  if (!existsSync(summaryPath)) {
    return null;
  }

  const summary = JSON.parse(readFileSync(summaryPath, "utf8")) as RepeatedBenchmarkSummary;
  if (summary.schemaVersion !== "iris.benchmark.repeated.v1") {
    throw new Error(`${path} has unexpected schema: ${summary.schemaVersion}`);
  }

  if (lane.suiteId != null && summary.suite.id !== lane.suiteId) {
    throw new Error(`${path} must contain ${lane.suiteId}, got ${summary.suite.id}.`);
  }

  return summary;
}

function measurementFromSummary(
  summaryPath: string,
  summary: RepeatedBenchmarkSummary,
): LaneMeasurement {
  return {
    cases: summary.cases.map((benchmarkCase) => ({
      checksum: benchmarkCase.checksum,
      id: benchmarkCase.id,
      label: benchmarkCase.label,
      p50Mean: benchmarkCase.p50.mean,
      p95Mean: benchmarkCase.p95.mean,
      unit: benchmarkCase.unit,
    })),
    metadata: summary.metadata,
    runCount: summary.runCount,
    sourceReports: summary.sourceReports,
    suite: summary.suite,
    summaryPath: relativePath(resolveFromRoot(summaryPath)),
  };
}

function laneEntry(lane: BenchmarkLane): RuntimeLaneReport["lanes"][number] {
  const summary = lane.summaryPath == null ? null : readSummary(lane.summaryPath, lane);
  const measurement =
    summary == null || lane.summaryPath == null
      ? null
      : measurementFromSummary(lane.summaryPath, summary);

  return {
    appId: lane.appId,
    comparisonReason: lane.comparisonReason,
    currentMeasurement: lane.currentMeasurement,
    engine: lane.engine,
    id: lane.id,
    label: lane.label,
    measurement,
    measurementStatus: measurement == null ? "missing" : "measured",
    objective: lane.objective,
    requiredCapabilities: lane.requiredCapabilities,
    runtimeBackend: lane.runtimeBackend,
    status: lane.status,
    strictComparableWithBaseline: lane.strictComparableWithBaseline,
    suiteId: lane.suiteId,
    summaryPath: lane.summaryPath,
  };
}

function compareLane(
  baseline: RuntimeLaneReport["lanes"][number],
  candidate: RuntimeLaneReport["lanes"][number],
): RuntimeLaneReport["comparisons"][number] | null {
  if (baseline.measurement == null || candidate.measurement == null) {
    return null;
  }

  const baselineCases = new Map(
    baseline.measurement.cases.map((benchmarkCase) => [benchmarkCase.id, benchmarkCase]),
  );
  const candidateCases = candidate.measurement.cases.filter((benchmarkCase) =>
    baselineCases.has(benchmarkCase.id),
  );

  if (candidateCases.length === 0) {
    return null;
  }

  const sameSuite = baseline.measurement.suite.id === candidate.measurement.suite.id;
  const baselineCaseIds = baseline.measurement.cases
    .map((benchmarkCase) => benchmarkCase.id)
    .sort();
  const candidateCaseIds = candidate.measurement.cases
    .map((benchmarkCase) => benchmarkCase.id)
    .sort();
  const sameCaseSet = baselineCaseIds.join("\n") === candidateCaseIds.join("\n");
  const sameChecksums =
    sameCaseSet &&
    candidate.measurement.cases.every((candidateCase) => {
      const baselineCase = baselineCases.get(candidateCase.id);

      return (
        baselineCase != null &&
        baselineCase.checksum != null &&
        candidateCase.checksum != null &&
        `${typeof baselineCase.checksum}:${JSON.stringify(baselineCase.checksum)}` ===
          `${typeof candidateCase.checksum}:${JSON.stringify(candidateCase.checksum)}`
      );
    });
  const candidateLane = benchmarkLane(candidate.id);
  const strictComparable =
    candidateLane.strictComparableWithBaseline && sameSuite && sameCaseSet && sameChecksums;
  const ratioAllowed = strictComparable;
  const reason = ratioAllowed
    ? "Same RN suite, case set, checksum, and units on a strict comparison lane."
    : candidateLane.comparisonReason;

  return {
    baselineLaneId: baseline.id,
    candidateLaneId: candidate.id,
    cases: candidateCases.map((candidateCase) => {
      const baselineCase = baselineCases.get(candidateCase.id);

      if (baselineCase == null) {
        throw new Error(`Missing baseline case ${candidateCase.id}.`);
      }

      if (baselineCase.unit !== candidateCase.unit) {
        throw new Error(
          `Unit mismatch for ${candidateCase.id}: baseline=${baselineCase.unit} candidate=${candidateCase.unit}.`,
        );
      }

      return {
        id: candidateCase.id,
        label: candidateCase.label,
        p50: {
          baseline: baselineCase.p50Mean,
          candidate: candidateCase.p50Mean,
          candidateOverBaseline: ratio(candidateCase.p50Mean, baselineCase.p50Mean),
        },
        p95: {
          baseline: baselineCase.p95Mean,
          candidate: candidateCase.p95Mean,
          candidateOverBaseline: ratio(candidateCase.p95Mean, baselineCase.p95Mean),
        },
        unit: candidateCase.unit,
      };
    }),
    ratioAllowed,
    reason,
    strictComparable,
  };
}

function compareBridgeFastPath(
  lane: RuntimeLaneReport["lanes"][number],
): RuntimeLaneReport["bridgeComparisons"][number] | null {
  if (lane.measurement == null) {
    return null;
  }

  const casePairs = [
    {
      candidateId: "iris-jsi-number-round-trip",
      comparisonClass: "same-payload",
      label: "number round trip",
      referenceId: "turbomodule-number-round-trip",
    },
    {
      candidateId: "iris-jsi-string-round-trip",
      comparisonClass: "same-payload",
      label: "string round trip",
      referenceId: "turbomodule-string-round-trip",
    },
    {
      candidateId: "iris-jsi-native-compute",
      comparisonClass: "same-payload",
      label: "native compute",
      referenceId: "iris-module-native-compute",
    },
    {
      candidateId: "iris-jsi-facade-number-round-trip",
      comparisonClass: "same-method-surface",
      label: "same method facade number",
      referenceId: "turbomodule-number-round-trip",
    },
    {
      candidateId: "iris-jsi-facade-string-round-trip",
      comparisonClass: "same-method-surface",
      label: "same method facade string",
      referenceId: "turbomodule-string-round-trip",
    },
    {
      candidateId: "iris-jsi-facade-native-compute",
      comparisonClass: "same-method-surface",
      label: "same method facade compute",
      referenceId: "iris-module-native-compute",
    },
    {
      candidateId: "iris-jsi-columnar-object-traversal",
      comparisonClass: "format-shift-diagnostic",
      label: "columnar object traversal",
      referenceId: "object-traversal",
    },
    {
      candidateId: "iris-jsi-native-buffer-read",
      comparisonClass: "format-shift-diagnostic",
      label: "native buffer handoff",
      referenceId: "typed-array-copy",
    },
  ] satisfies Array<{
    candidateId: string;
    comparisonClass: BridgeComparisonClass;
    label: string;
    referenceId: string;
  }>;
  const casesById = new Map(
    lane.measurement.cases.map((benchmarkCase) => [benchmarkCase.id, benchmarkCase]),
  );
  const cases = casePairs.flatMap((casePair) => {
    const referenceCase = casesById.get(casePair.referenceId);
    const candidateCase = casesById.get(casePair.candidateId);

    if (referenceCase == null || candidateCase == null) {
      return [];
    }

    if (referenceCase.unit !== candidateCase.unit) {
      throw new Error(
        `Unit mismatch for ${casePair.candidateId}: reference=${referenceCase.unit} candidate=${candidateCase.unit}.`,
      );
    }

    return [
      {
        candidateId: casePair.candidateId,
        comparisonClass: casePair.comparisonClass,
        label: casePair.label,
        p50: {
          candidate: candidateCase.p50Mean,
          candidateOverReference: ratio(candidateCase.p50Mean, referenceCase.p50Mean),
          reference: referenceCase.p50Mean,
        },
        p95: {
          candidate: candidateCase.p95Mean,
          candidateOverReference: ratio(candidateCase.p95Mean, referenceCase.p95Mean),
          reference: referenceCase.p95Mean,
        },
        referenceId: casePair.referenceId,
        unit: candidateCase.unit,
      },
    ];
  });

  if (cases.length === 0) {
    return null;
  }

  return {
    cases,
    laneId: lane.id,
    reason:
      "Same app, same Hermes runtime, comparing Iris-owned JSI host functions against synchronous TurboModule calls.",
  };
}

function createReport(): RuntimeLaneReport {
  const lanes = allBenchmarkLanes().map(laneEntry);
  const baseline = lanes.find((lane) => lane.id === baselineLaneId);

  if (baseline == null) {
    throw new Error(`Missing baseline lane: ${baselineLaneId}`);
  }

  const comparisons = lanes
    .filter((lane) => lane.id !== baselineLaneId)
    .map((lane) => compareLane(baseline, lane))
    .filter((comparison): comparison is NonNullable<typeof comparison> => comparison != null);
  const bridgeComparisons = lanes
    .filter((lane) => lane.id === "hermes-iris-bridge")
    .map(compareBridgeFastPath)
    .filter((comparison): comparison is NonNullable<typeof comparison> => comparison != null);

  return {
    baselineLaneId,
    bridgeComparisons,
    comparisons,
    createdAt: new Date().toISOString(),
    generatedBy: "tools/bench/write-android-runtime-lane-report.ts",
    lanes,
    schemaVersion: "iris.benchmark.android-runtime-lanes.v1",
  };
}

const outputPath = resolve(root, readArg("--output") ?? defaultOutputPath);
const report = createReport();
const missingLanes = report.lanes.filter((lane) => lane.measurementStatus === "missing");

if (hasArg("--require-measured") && missingLanes.length > 0) {
  throw new Error(`Missing lane measurements: ${missingLanes.map((lane) => lane.id).join(", ")}`);
}

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(report, null, 2)}\n`);

console.log(`Android runtime lane report: ${outputPath}`);
for (const lane of report.lanes) {
  console.log(
    `${lane.id}: status=${lane.status}, measurement=${lane.measurementStatus}, backend=${lane.runtimeBackend}`,
  );
}
for (const comparison of report.comparisons) {
  console.log(
    `${comparison.candidateLaneId}/${comparison.baselineLaneId}: ratioAllowed=${comparison.ratioAllowed}, strictComparable=${comparison.strictComparable}`,
  );
  for (const benchmarkCase of comparison.cases) {
    console.log(
      `${benchmarkCase.id}: p50=${benchmarkCase.p50.candidateOverBaseline}x p95=${benchmarkCase.p95.candidateOverBaseline}x`,
    );
  }
}
for (const comparison of report.bridgeComparisons) {
  console.log(`${comparison.laneId}: JSI fast path over reference`);
  for (const benchmarkCase of comparison.cases) {
    console.log(
      `${benchmarkCase.label} [${benchmarkCase.comparisonClass}]: p50=${benchmarkCase.p50.candidateOverReference}x p95=${benchmarkCase.p95.candidateOverReference}x`,
    );
  }
}
