import { summarizeSamples } from "./statistics";
import type {
  BenchmarkArtifact,
  BenchmarkCase,
  BenchmarkCaseReport,
  BenchmarkMetadata,
  BenchmarkRunValue,
  BenchmarkRunOptions,
  BenchmarkSuiteReport,
} from "./types";

export const BENCHMARK_SCHEMA_VERSION = "iris.benchmark.v1";

type RunSuiteOptions = BenchmarkRunOptions & {
  artifact?: BenchmarkArtifact;
  now?: () => number;
  yieldBetweenCases?: boolean;
};

function defaultNow() {
  const timingGlobal = globalThis as typeof globalThis & {
    performance?: {
      now?: () => number;
    };
  };

  return timingGlobal.performance?.now?.() ?? Date.now();
}

function waitForNextTurn() {
  return new Promise<void>((resolve) => setTimeout(() => resolve(), 0));
}

async function runCase(
  benchmarkCase: BenchmarkCase,
  now: () => number,
  options: BenchmarkRunOptions,
): Promise<BenchmarkCaseReport> {
  const warmupIterations = options.warmupIterations ?? benchmarkCase.warmupIterations;
  const measuredIterations = options.measuredIterations ?? benchmarkCase.measuredIterations;

  for (let index = 0; index < warmupIterations; index += 1) {
    benchmarkCase.run();
  }

  const samples: number[] = [];
  let lastRun: BenchmarkRunValue | null = null;

  for (let index = 0; index < measuredIterations; index += 1) {
    const startedAt = now();
    lastRun = benchmarkCase.run();
    samples.push(now() - startedAt);
  }

  return {
    checksum: lastRun?.checksum ?? "not-run",
    description: benchmarkCase.description,
    detail: lastRun?.detail ?? "not-run",
    id: benchmarkCase.id,
    label: benchmarkCase.label,
    measuredIterations,
    stats: summarizeSamples(samples),
    unit: benchmarkCase.unit,
    warmupIterations,
  };
}

export async function runBenchmarkSuite(
  cases: BenchmarkCase[],
  metadata: BenchmarkMetadata,
  options: RunSuiteOptions = {},
): Promise<BenchmarkSuiteReport> {
  const now = options.now ?? defaultNow;
  const startedAt = now();
  const reports: BenchmarkCaseReport[] = [];

  for (const benchmarkCase of cases) {
    reports.push(await runCase(benchmarkCase, now, options));

    if (options.yieldBetweenCases) {
      await waitForNextTurn();
    }
  }

  return {
    artifact: options.artifact ?? {
      generatedBy: "apps/rn-bench",
      kind: "runtime-log",
      path: "Metro console",
    },
    cases: reports,
    createdAt: new Date().toISOString(),
    metadata,
    schemaVersion: BENCHMARK_SCHEMA_VERSION,
    suite: {
      id: "rn-hermes-js-baseline",
      name: "React Native Hermes JS Baseline",
    },
    summary: {
      caseCount: reports.length,
      measuredIterations: reports.reduce((total, report) => total + report.measuredIterations, 0),
      totalElapsedMs: Number((now() - startedAt).toFixed(3)),
    },
  };
}
