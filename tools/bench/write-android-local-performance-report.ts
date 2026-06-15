import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

const root = resolve(import.meta.dir, "../..");
const defaultHermesSummaryPath = "artifacts/bench/hermes-release-baseline-summary.json";
const defaultIrisBootstrapSummaryPath = "artifacts/bench/iris-bootstrap-baseline-summary.json";
const defaultOutputPath = "artifacts/bench/android-local-performance-report.json";

type CaseMetricSummary = {
  max: number;
  mean: number;
  min: number;
  samples: number[];
};

type RepeatedCaseSummary = {
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

type LocalPerformanceReport = {
  caseComparisons: Array<{
    hermesCaseId: string;
    irisCaseId: string;
    nativeMirrorComparable: boolean;
    p50: {
      hermes: number;
      iris: number;
      nativeMirrorOverHermesRatio: number;
    };
    p95: {
      hermes: number;
      iris: number;
      nativeMirrorOverHermesRatio: number;
    };
    reason: string;
    strictComparable: boolean;
    unit: string;
  }>;
  comparability: {
    ratioAllowed: false;
    reason: string;
  };
  createdAt: string;
  generatedBy: string;
  measurements: Array<{
    cases: Array<{
      id: string;
      label: string;
      p50Mean: number;
      p95Mean: number;
      unit: string;
    }>;
    id: "hermes-release" | "iris-bootstrap";
    metadata: unknown;
    runCount: number;
    sourceReports: string[];
    suite: RepeatedBenchmarkSummary["suite"];
    summaryPath: string;
  }>;
  schemaVersion: "iris.benchmark.android-local-performance.v1";
};

const nativeMirrorCaseMappings = [
  ["js-compute", "iris-native-js-compute-mirror"],
  ["json-round-trip", "iris-native-json-round-trip-mirror"],
  ["object-traversal", "iris-native-object-traversal-mirror"],
  ["typed-array-copy", "iris-native-typed-array-copy-mirror"],
  ["turbomodule-number-round-trip", "iris-native-number-round-trip-mirror"],
  ["turbomodule-string-round-trip", "iris-native-string-round-trip-mirror"],
  ["iris-module-native-compute", "iris-native-module-compute-mirror"],
] as const;

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

function relativePath(path: string) {
  return path.replace(`${root}/`, "");
}

function readSummary(path: string) {
  const summary = JSON.parse(readFileSync(path, "utf8")) as RepeatedBenchmarkSummary;

  if (summary.schemaVersion !== "iris.benchmark.repeated.v1") {
    throw new Error(`${relativePath(path)} has unexpected schema: ${summary.schemaVersion}`);
  }

  if (summary.runCount <= 0) {
    throw new Error(`${relativePath(path)} has no benchmark runs.`);
  }

  if (summary.cases.length === 0) {
    throw new Error(`${relativePath(path)} has no benchmark cases.`);
  }

  return summary;
}

function measurement(
  id: LocalPerformanceReport["measurements"][number]["id"],
  summaryPath: string,
  summary: RepeatedBenchmarkSummary,
): LocalPerformanceReport["measurements"][number] {
  return {
    cases: summary.cases.map((benchmarkCase) => ({
      id: benchmarkCase.id,
      label: benchmarkCase.label,
      p50Mean: benchmarkCase.p50.mean,
      p95Mean: benchmarkCase.p95.mean,
      unit: benchmarkCase.unit,
    })),
    id,
    metadata: summary.metadata,
    runCount: summary.runCount,
    sourceReports: summary.sourceReports,
    suite: summary.suite,
    summaryPath: relativePath(summaryPath),
  };
}

function ratio(numerator: number, denominator: number) {
  return Number((numerator / denominator).toFixed(4));
}

function requireCase(summary: RepeatedBenchmarkSummary, caseId: string) {
  const benchmarkCase = summary.cases.find((candidate) => candidate.id === caseId);

  if (benchmarkCase == null) {
    throw new Error(`${summary.suite.id} is missing benchmark case ${caseId}.`);
  }

  return benchmarkCase;
}

function createNativeMirrorComparisons(
  hermesSummary: RepeatedBenchmarkSummary,
  irisBootstrapSummary: RepeatedBenchmarkSummary,
): LocalPerformanceReport["caseComparisons"] {
  return nativeMirrorCaseMappings.map(([hermesCaseId, irisCaseId]) => {
    const hermesCase = requireCase(hermesSummary, hermesCaseId);
    const irisCase = requireCase(irisBootstrapSummary, irisCaseId);

    if (hermesCase.unit !== irisCase.unit) {
      throw new Error(
        `Unit mismatch for ${hermesCaseId}/${irisCaseId}: hermes=${hermesCase.unit} iris=${irisCase.unit}.`,
      );
    }

    return {
      hermesCaseId,
      irisCaseId,
      nativeMirrorComparable: true,
      p50: {
        hermes: hermesCase.p50.mean,
        iris: irisCase.p50.mean,
        nativeMirrorOverHermesRatio: ratio(irisCase.p50.mean, hermesCase.p50.mean),
      },
      p95: {
        hermes: hermesCase.p95.mean,
        iris: irisCase.p95.mean,
        nativeMirrorOverHermesRatio: ratio(irisCase.p95.mean, hermesCase.p95.mean),
      },
      reason:
        "Hermes executes the RN JavaScript/TurboModule case; Iris currently executes a native mirror probe with the same sample shape. This ratio is useful for component direction, not strict engine replacement claims.",
      strictComparable: false,
      unit: hermesCase.unit,
    };
  });
}

const hermesSummaryPath = resolve(root, readArg("--hermes-summary") ?? defaultHermesSummaryPath);
const irisBootstrapSummaryPath = resolve(
  root,
  readArg("--iris-bootstrap-summary") ?? defaultIrisBootstrapSummaryPath,
);
const outputPath = resolve(root, readArg("--output") ?? defaultOutputPath);
const hermesSummary = readSummary(hermesSummaryPath);
const irisBootstrapSummary = readSummary(irisBootstrapSummaryPath);

if (hermesSummary.suite.id !== "rn-hermes-js-baseline") {
  throw new Error(
    `${relativePath(hermesSummaryPath)} must contain rn-hermes-js-baseline, got ${hermesSummary.suite.id}.`,
  );
}

if (irisBootstrapSummary.suite.id !== "iris-engine-bootstrap") {
  throw new Error(
    `${relativePath(irisBootstrapSummaryPath)} must contain iris-engine-bootstrap, got ${irisBootstrapSummary.suite.id}.`,
  );
}

const report: LocalPerformanceReport = {
  caseComparisons: createNativeMirrorComparisons(hermesSummary, irisBootstrapSummary),
  comparability: {
    ratioAllowed: false,
    reason:
      "Hermes release measures the RN JS benchmark suite; local Iris currently measures native HBC bootstrap/scalar execution plus native mirror probes. Strict engine ratios are allowed only after both engines emit the same suite and case set from JavaScript execution.",
  },
  createdAt: new Date().toISOString(),
  generatedBy: "tools/bench/write-android-local-performance-report.ts",
  measurements: [
    measurement("hermes-release", hermesSummaryPath, hermesSummary),
    measurement("iris-bootstrap", irisBootstrapSummaryPath, irisBootstrapSummary),
  ],
  schemaVersion: "iris.benchmark.android-local-performance.v1",
};

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(report, null, 2)}\n`);

console.log(`Android local performance report: ${outputPath}`);
for (const summary of report.measurements) {
  console.log(`${summary.id}: ${summary.suite.id}, runs=${summary.runCount}`);
  for (const benchmarkCase of summary.cases) {
    console.log(
      `${benchmarkCase.id}: p50(mean=${benchmarkCase.p50Mean}${benchmarkCase.unit}) p95(mean=${benchmarkCase.p95Mean}${benchmarkCase.unit})`,
    );
  }
}
console.log(`ratioAllowed=${report.comparability.ratioAllowed}: ${report.comparability.reason}`);
for (const comparison of report.caseComparisons) {
  console.log(
    `${comparison.hermesCaseId} -> ${comparison.irisCaseId}: nativeMirror p50 iris/hermes=${comparison.p50.nativeMirrorOverHermesRatio} p95 iris/hermes=${comparison.p95.nativeMirrorOverHermesRatio} strictComparable=${comparison.strictComparable}`,
  );
}
