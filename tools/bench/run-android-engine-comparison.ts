import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { execFileSync } from "node:child_process";

const root = resolve(import.meta.dir, "../..");
const defaultRuns = 3;
const defaultWaitMs = 45_000;
const defaultOutputPath = "artifacts/bench/android-engine-comparison.json";

type EngineConfig = {
  allowNonHermes: boolean;
  apkPath: string;
  appId: string;
  id: "hermes" | "iris";
  label: string;
  logPath: string;
  reportPath: string;
  session: string;
  summaryPath: string;
};

type RepeatedBenchmarkSummary = {
  cases: Array<{
    id: string;
    label: string;
    p50: {
      mean: number;
    };
    p95: {
      mean: number;
    };
    unit: string;
  }>;
  metadata: unknown;
  runCount: number;
  schemaVersion: "iris.benchmark.repeated.v1";
  sourceReports: string[];
  suite: {
    id: string;
    name: string;
  };
};

type EngineComparison = {
  cases: Array<{
    id: string;
    label: string;
    p50: {
      hermes: number;
      iris: number;
      ratio: number;
    };
    p95: {
      hermes: number;
      iris: number;
      ratio: number;
    };
    unit: string;
  }>;
  createdAt: string;
  engines: Array<{
    appId: string;
    id: EngineConfig["id"];
    label: string;
    metadata: unknown;
    sourceReports: string[];
    summaryPath: string;
  }>;
  generatedBy: string;
  runCount: number;
  schemaVersion: "iris.benchmark.engine-comparison.v1";
  suite: RepeatedBenchmarkSummary["suite"];
};

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

function hasArg(name: string) {
  return process.argv.includes(name);
}

function readNumberArg(name: string, fallback: number) {
  const rawValue = readArg(name);
  if (rawValue == null) {
    return fallback;
  }

  const value = Number(rawValue);
  if (!Number.isFinite(value) || value <= 0) {
    throw new Error(`${name} must be a positive number.`);
  }

  return value;
}

function relativePath(path: string) {
  return path.replace(`${root}/`, "");
}

function shellLine(command: string, args: string[]) {
  return [command, ...args.map((arg) => (arg.includes(" ") ? JSON.stringify(arg) : arg))].join(" ");
}

function run(command: string, args: string[], dryRun: boolean) {
  console.log(`$ ${shellLine(command, args)}`);

  if (dryRun) {
    return;
  }

  execFileSync(command, args, {
    cwd: root,
    stdio: "inherit",
  });
}

function assertApkExists(engine: EngineConfig) {
  if (!existsSync(engine.apkPath)) {
    const hint =
      engine.id === "iris"
        ? "Build a real Iris engine APK first with IRIS_ENGINE_AAR and `mise run rn-android-build-iris-release`."
        : "Build the Hermes release APK first with `mise run rn-android-build-release`.";

    throw new Error(`${engine.label} APK does not exist: ${engine.apkPath}\n${hint}`);
  }
}

function createEngines(): EngineConfig[] {
  return [
    {
      allowNonHermes: false,
      apkPath: resolve(
        root,
        readArg("--hermes-apk") ??
          "apps/rn-bench/android/app/build/outputs/apk/hermes/release/app-hermes-release.apk",
      ),
      appId: readArg("--hermes-app-id") ?? "com.iris.bench.hermes",
      id: "hermes",
      label: "Hermes release",
      logPath: resolve(
        root,
        readArg("--hermes-log-output") ?? "artifacts/bench/rn-release-hermes.log",
      ),
      reportPath: resolve(
        root,
        readArg("--hermes-report-output") ?? "artifacts/bench/hermes-release-baseline.json",
      ),
      session: readArg("--hermes-session") ?? "rnbench-android-hermes-release",
      summaryPath: resolve(
        root,
        readArg("--hermes-summary-output") ??
          "artifacts/bench/hermes-release-baseline-summary.json",
      ),
    },
    {
      allowNonHermes: true,
      apkPath: resolve(
        root,
        readArg("--iris-apk") ??
          "apps/rn-bench/android/app/build/outputs/apk/iris/release/app-iris-release.apk",
      ),
      appId: readArg("--iris-app-id") ?? "com.iris.bench.iris",
      id: "iris",
      label: "Iris release",
      logPath: resolve(root, readArg("--iris-log-output") ?? "artifacts/bench/rn-release-iris.log"),
      reportPath: resolve(
        root,
        readArg("--iris-report-output") ?? "artifacts/bench/iris-release-baseline.json",
      ),
      session: readArg("--iris-session") ?? "rnbench-android-iris-release",
      summaryPath: resolve(
        root,
        readArg("--iris-summary-output") ?? "artifacts/bench/iris-release-baseline-summary.json",
      ),
    },
  ];
}

function runEngineBenchmark(engine: EngineConfig, runs: number, waitMs: number, dryRun: boolean) {
  const args = [
    "run",
    "tools/bench/run-android-release-benchmark.ts",
    `--app-id=${engine.appId}`,
    `--apk=${engine.apkPath}`,
    `--log-output=${engine.logPath}`,
    `--report-output=${engine.reportPath}`,
    `--summary-output=${engine.summaryPath}`,
    `--session=${engine.session}`,
    `--runs=${runs}`,
    `--wait-ms=${waitMs}`,
  ];

  if (engine.allowNonHermes) {
    args.push("--allow-non-hermes");
  }

  run("bun", args, dryRun);
}

function readSummary(engine: EngineConfig) {
  return JSON.parse(readFileSync(engine.summaryPath, "utf8")) as RepeatedBenchmarkSummary;
}

function ratio(numerator: number, denominator: number) {
  return Number((numerator / denominator).toFixed(4));
}

function compareSummaries(
  engines: [EngineConfig, EngineConfig],
  summaries: [RepeatedBenchmarkSummary, RepeatedBenchmarkSummary],
) {
  const [hermesSummary, irisSummary] = summaries;
  const irisCases = new Map(
    irisSummary.cases.map((benchmarkCase) => [benchmarkCase.id, benchmarkCase]),
  );

  const cases = hermesSummary.cases.map((hermesCase) => {
    const irisCase = irisCases.get(hermesCase.id);

    if (irisCase == null) {
      throw new Error(`Iris summary is missing case ${hermesCase.id}.`);
    }

    return {
      id: hermesCase.id,
      label: hermesCase.label,
      p50: {
        hermes: hermesCase.p50.mean,
        iris: irisCase.p50.mean,
        ratio: ratio(irisCase.p50.mean, hermesCase.p50.mean),
      },
      p95: {
        hermes: hermesCase.p95.mean,
        iris: irisCase.p95.mean,
        ratio: ratio(irisCase.p95.mean, hermesCase.p95.mean),
      },
      unit: hermesCase.unit,
    };
  });

  return {
    cases,
    createdAt: new Date().toISOString(),
    engines: engines.map((engine, index) => ({
      appId: engine.appId,
      id: engine.id,
      label: engine.label,
      metadata: summaries[index].metadata,
      sourceReports: summaries[index].sourceReports,
      summaryPath: relativePath(engine.summaryPath),
    })),
    generatedBy: "tools/bench/run-android-engine-comparison.ts",
    runCount: hermesSummary.runCount,
    schemaVersion: "iris.benchmark.engine-comparison.v1",
    suite: hermesSummary.suite,
  } satisfies EngineComparison;
}

function writeComparison(comparison: EngineComparison, outputPath: string) {
  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, `${JSON.stringify(comparison, null, 2)}\n`);

  console.log(`Android engine comparison artifact: ${outputPath}`);
  for (const benchmarkCase of comparison.cases) {
    console.log(
      `${benchmarkCase.id}: p50 iris/hermes=${benchmarkCase.p50.ratio} p95 iris/hermes=${benchmarkCase.p95.ratio}`,
    );
  }
}

function main() {
  const dryRun = hasArg("--dry-run");
  const outputPath = resolve(root, readArg("--output") ?? defaultOutputPath);
  const runs = readNumberArg("--runs", defaultRuns);
  const waitMs = readNumberArg("--wait-ms", defaultWaitMs);
  const engines = createEngines() as [EngineConfig, EngineConfig];

  if (!dryRun) {
    for (const engine of engines) {
      assertApkExists(engine);
    }
  }

  for (const engine of engines) {
    runEngineBenchmark(engine, runs, waitMs, dryRun);
  }

  if (dryRun) {
    console.log("Dry run completed without running device benchmarks.");
    return;
  }

  writeComparison(
    compareSummaries(engines, [readSummary(engines[0]), readSummary(engines[1])]),
    outputPath,
  );
}

main();
