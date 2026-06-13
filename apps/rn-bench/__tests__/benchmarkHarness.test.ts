import { runBenchmarkSuite } from "../src/benchmarks/harness";
import type { BenchmarkCase, BenchmarkMetadata } from "../src/benchmarks/types";

const metadata: BenchmarkMetadata = {
  app: {
    name: "IrisBench",
    version: "0.0.1",
  },
  build: {
    commit: "test",
    mode: "unknown",
    source: "jest",
  },
  platform: {
    device: "jest",
    os: "node",
    version: "jest",
  },
  reactNative: {
    version: "0.85.0",
  },
  runtime: {
    fabric: false,
    hermes: false,
    hermesVersion: "not-applicable",
    jsEngine: "unknown",
    newArchitecture: false,
    turboModuleProxy: false,
  },
};

test("builds a benchmark suite report with statistics", async () => {
  let calls = 0;
  let clock = 0;
  const benchmarkCase: BenchmarkCase = {
    description: "test case",
    id: "test-case",
    label: "Test case",
    measuredIterations: 4,
    run: () => {
      calls += 1;

      return {
        checksum: calls,
        detail: "test run",
      };
    },
    unit: "ms",
    warmupIterations: 2,
  };

  const report = await runBenchmarkSuite([benchmarkCase], metadata, {
    now: () => {
      clock += 2;
      return clock;
    },
  });

  expect(report.schemaVersion).toBe("iris.benchmark.v1");
  expect(report.summary.caseCount).toBe(1);
  expect(report.summary.measuredIterations).toBe(4);
  expect(report.cases[0].stats.samples).toEqual([2, 2, 2, 2]);
  expect(report.cases[0].stats.p50).toBe(2);
  expect(report.cases[0].stats.p95).toBe(2);
  expect(calls).toBe(6);
});
