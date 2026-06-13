export type BenchmarkUnit = "ms";

export type BenchmarkCase = {
  description: string;
  id: string;
  label: string;
  measuredIterations: number;
  run: () => BenchmarkRunValue;
  unit: BenchmarkUnit;
  warmupIterations: number;
};

export type BenchmarkRunValue = {
  detail: string;
  checksum: number | string;
};

export type BenchmarkStats = {
  max: number;
  mean: number;
  min: number;
  p50: number;
  p95: number;
  samples: number[];
};

export type BenchmarkCaseReport = {
  checksum: number | string;
  description: string;
  detail: string;
  id: string;
  label: string;
  measuredIterations: number;
  stats: BenchmarkStats;
  unit: BenchmarkUnit;
  warmupIterations: number;
};

export type BenchmarkMetadata = {
  app: {
    name: string;
    version: string;
  };
  build: {
    commit: string;
    mode: "development" | "release" | "unknown";
    source: string;
  };
  platform: {
    device: string;
    os: string;
    version: string;
  };
  reactNative: {
    version: string;
  };
  runtime: {
    fabric: boolean;
    hermes: boolean;
    hermesVersion: string;
    jsEngine: "hermes" | "unknown";
    newArchitecture: boolean;
    turboModuleProxy: boolean;
  };
};

export type BenchmarkArtifact = {
  generatedBy: string;
  kind: "file" | "runtime-log";
  path: string;
};

export type BenchmarkSuiteReport = {
  artifact: BenchmarkArtifact;
  cases: BenchmarkCaseReport[];
  createdAt: string;
  metadata: BenchmarkMetadata;
  schemaVersion: "iris.benchmark.v1";
  suite: {
    id: string;
    name: string;
  };
  summary: {
    caseCount: number;
    measuredIterations: number;
    totalElapsedMs: number;
  };
};

export type BenchmarkRunOptions = {
  measuredIterations?: number;
  warmupIterations?: number;
};
