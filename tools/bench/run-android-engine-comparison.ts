import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { execFileSync } from "node:child_process";

const root = resolve(import.meta.dir, "../..");
const defaultRuns = 3;
const defaultWaitMs = 45_000;
const defaultOutputPath = "artifacts/bench/android-engine-comparison.json";
const hermesBytecodeMagic = 0x1f1903c103bc1fc6n;
const hermesBytecodeHeaderSize = 128;
const apkBundleEntry = "assets/index.android.bundle";

type EngineConfig = {
  allowNonHermes: boolean;
  apkPath: string;
  appId: string;
  bundlePath: string;
  id: "hermes" | "iris";
  label: string;
  logPath: string;
  reportPath: string;
  session: string;
  summaryPath: string;
};

type HbcBundleInput = {
  fileLength: number;
  path: string;
  sha256: string;
  sizeBytes: number;
  sourceHash: string;
  version: number;
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
    bundle: HbcBundleInput;
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

function readApkEntries(engine: EngineConfig) {
  try {
    return execFileSync("unzip", ["-Z1", engine.apkPath], {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    })
      .split("\n")
      .filter(Boolean);
  } catch (error) {
    if (typeof error === "object" && error != null && "code" in error && error.code === "ENOENT") {
      throw new Error("unzip was not found. Install unzip before validating Android APK inputs.");
    }

    throw error;
  }
}

function hasNativeLibrary(entries: string[], libraryName: string) {
  return entries.some((entry) => entry.startsWith("lib/") && entry.endsWith(`/${libraryName}`));
}

function assertNativeLibraryPresent(engine: EngineConfig, entries: string[], libraryName: string) {
  if (!hasNativeLibrary(entries, libraryName)) {
    throw new Error(`${engine.label} APK must include ${libraryName}: ${engine.apkPath}`);
  }
}

function assertNativeLibraryAbsent(engine: EngineConfig, entries: string[], libraryName: string) {
  if (hasNativeLibrary(entries, libraryName)) {
    throw new Error(`${engine.label} APK must not include ${libraryName}: ${engine.apkPath}`);
  }
}

function assertApkRuntimeBoundary(engine: EngineConfig) {
  const entries = readApkEntries(engine);

  if (!entries.includes(apkBundleEntry)) {
    throw new Error(`${engine.label} APK must include ${apkBundleEntry}: ${engine.apkPath}`);
  }

  if (engine.id === "hermes") {
    assertNativeLibraryPresent(engine, entries, "libhermesvm.so");
    assertNativeLibraryAbsent(engine, entries, "libirisengine.so");
    assertNativeLibraryAbsent(engine, entries, "libjsc.so");
    console.log(`${engine.label} APK runtime boundary: Hermes VM present, Iris/JSC absent.`);
    return;
  }

  assertNativeLibraryPresent(engine, entries, "libirisengine.so");
  assertNativeLibraryAbsent(engine, entries, "libhermesvm.so");
  assertNativeLibraryAbsent(engine, entries, "libjsc.so");
  console.log(`${engine.label} APK runtime boundary: Iris engine present, Hermes VM/JSC absent.`);
}

function readApkBundle(engine: EngineConfig) {
  try {
    return execFileSync("unzip", ["-p", engine.apkPath, apkBundleEntry], {
      cwd: root,
      maxBuffer: 64 * 1024 * 1024,
      stdio: ["ignore", "pipe", "pipe"],
    });
  } catch (error) {
    if (typeof error === "object" && error != null && "code" in error && error.code === "ENOENT") {
      throw new Error("unzip was not found. Install unzip before validating Android APK inputs.");
    }

    throw error;
  }
}

function parseHbcBytes(engine: EngineConfig, bytes: Buffer, path: string) {
  if (bytes.length < hermesBytecodeHeaderSize) {
    throw new Error(`${engine.label} bundle is too small to be Hermes bytecode: ${path}`);
  }

  const magic = bytes.readBigUInt64LE(0);
  if (magic !== hermesBytecodeMagic) {
    throw new Error(
      `${engine.label} bundle is not Hermes bytecode: ${path}\n` +
        `Expected HBC magic 0x${hermesBytecodeMagic.toString(16)}, got 0x${magic.toString(16)}.`,
    );
  }

  const fileLength = bytes.readUInt32LE(32);
  if (fileLength !== bytes.length) {
    throw new Error(
      `${engine.label} HBC file length mismatch: header=${fileLength}, actual=${bytes.length}, path=${path}`,
    );
  }

  return {
    fileLength,
    path,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    sizeBytes: bytes.length,
    sourceHash: bytes.subarray(12, 32).toString("hex"),
    version: bytes.readUInt32LE(8),
  } satisfies HbcBundleInput;
}

function parseGeneratedHbcBundle(engine: EngineConfig) {
  if (!existsSync(engine.bundlePath)) {
    const hint =
      engine.id === "iris"
        ? "Build irisRelease from the same React Native source before comparing. Do not substitute a plain JS bundle or a non-HBC Iris compiler output."
        : "Build hermesRelease before comparing so the Hermes HBC baseline is available.";

    throw new Error(`${engine.label} HBC bundle does not exist: ${engine.bundlePath}\n${hint}`);
  }

  return parseHbcBytes(engine, readFileSync(engine.bundlePath), relativePath(engine.bundlePath));
}

function parsePackagedHbcBundle(engine: EngineConfig) {
  return parseHbcBytes(
    engine,
    readApkBundle(engine),
    `${relativePath(engine.apkPath)}!/${apkBundleEntry}`,
  );
}

function assertMatchingBundleMetadata(
  label: string,
  leftLabel: string,
  left: HbcBundleInput,
  rightLabel: string,
  right: HbcBundleInput,
) {
  const mismatches = [
    ["version", left.version, right.version],
    ["sourceHash", left.sourceHash, right.sourceHash],
    ["fileLength", left.fileLength, right.fileLength],
    ["sha256", left.sha256, right.sha256],
  ].filter(([, leftValue, rightValue]) => leftValue !== rightValue);

  if (mismatches.length === 0) {
    return;
  }

  const details = mismatches
    .map(
      ([field, leftValue, rightValue]) =>
        `${field}: ${leftLabel}=${leftValue} ${rightLabel}=${rightValue}`,
    )
    .join("\n");

  throw new Error(`${label}\n${details}`);
}

function assertPackagedBundleMatchesGenerated(engine: EngineConfig) {
  const generatedBundle = parseGeneratedHbcBundle(engine);
  const packagedBundle = parsePackagedHbcBundle(engine);

  assertMatchingBundleMetadata(
    `${engine.label} APK bundle must match the generated HBC bundle.`,
    "generated",
    generatedBundle,
    "apk",
    packagedBundle,
  );

  return packagedBundle;
}

function assertMatchingHbcBundles(engines: [EngineConfig, EngineConfig]) {
  const bundles = engines.map(assertPackagedBundleMatchesGenerated) as [
    HbcBundleInput,
    HbcBundleInput,
  ];
  const [hermesBundle, irisBundle] = bundles;

  assertMatchingBundleMetadata(
    "Hermes/Iris comparison requires byte-identical Hermes bytecode inputs.\n" +
      "Rebuild both release variants from the same RN source and keep the hermesc HBC pipeline for irisRelease.",
    "hermes",
    hermesBundle,
    "iris",
    irisBundle,
  );

  console.log(
    `HBC input parity: ${hermesBundle.path} == ${irisBundle.path} ` +
      `(v${hermesBundle.version}, ${hermesBundle.sizeBytes} bytes, sha256=${hermesBundle.sha256})`,
  );

  return bundles;
}

function assertComparisonInputs(engines: [EngineConfig, EngineConfig]) {
  for (const engine of engines) {
    assertApkExists(engine);
    assertApkRuntimeBoundary(engine);
  }

  return assertMatchingHbcBundles(engines);
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
      bundlePath: resolve(
        root,
        readArg("--hermes-bundle") ??
          "apps/rn-bench/android/app/build/generated/assets/react/hermesRelease/index.android.bundle",
      ),
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
      bundlePath: resolve(
        root,
        readArg("--iris-bundle") ??
          "apps/rn-bench/android/app/build/generated/assets/react/irisRelease/index.android.bundle",
      ),
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

function caseList(cases: Array<{ id: string }>) {
  return cases
    .map((benchmarkCase) => benchmarkCase.id)
    .sort()
    .join(", ");
}

function assertComparableSummaries(
  engines: [EngineConfig, EngineConfig],
  summaries: [RepeatedBenchmarkSummary, RepeatedBenchmarkSummary],
) {
  const [hermesSummary, irisSummary] = summaries;

  for (const [index, summary] of summaries.entries()) {
    if (summary.schemaVersion !== "iris.benchmark.repeated.v1") {
      throw new Error(
        `${engines[index].label} summary has unexpected schema: ${summary.schemaVersion}`,
      );
    }
  }

  if (hermesSummary.runCount !== irisSummary.runCount) {
    throw new Error(
      `Hermes/Iris run counts differ: hermes=${hermesSummary.runCount} iris=${irisSummary.runCount}`,
    );
  }

  if (
    hermesSummary.suite.id !== irisSummary.suite.id ||
    hermesSummary.suite.name !== irisSummary.suite.name
  ) {
    throw new Error(
      "Hermes/Iris summaries came from different benchmark suites.\n" +
        `hermes=${hermesSummary.suite.id} (${hermesSummary.suite.name})\n` +
        `iris=${irisSummary.suite.id} (${irisSummary.suite.name})`,
    );
  }

  const hermesCases = new Map(
    hermesSummary.cases.map((benchmarkCase) => [benchmarkCase.id, benchmarkCase]),
  );
  const irisCases = new Map(
    irisSummary.cases.map((benchmarkCase) => [benchmarkCase.id, benchmarkCase]),
  );
  const missingFromIris = hermesSummary.cases.filter(
    (benchmarkCase) => !irisCases.has(benchmarkCase.id),
  );
  const extraInIris = irisSummary.cases.filter(
    (benchmarkCase) => !hermesCases.has(benchmarkCase.id),
  );

  if (missingFromIris.length > 0 || extraInIris.length > 0) {
    throw new Error(
      "Hermes/Iris benchmark case sets differ.\n" +
        `hermes cases: ${caseList(hermesSummary.cases)}\n` +
        `iris cases: ${caseList(irisSummary.cases)}`,
    );
  }

  for (const hermesCase of hermesSummary.cases) {
    const irisCase = irisCases.get(hermesCase.id);

    if (irisCase == null) {
      throw new Error(`Iris summary is missing case ${hermesCase.id}.`);
    }

    if (hermesCase.unit !== irisCase.unit) {
      throw new Error(
        `Hermes/Iris units differ for ${hermesCase.id}: hermes=${hermesCase.unit} iris=${irisCase.unit}`,
      );
    }
  }
}

function compareSummaries(
  engines: [EngineConfig, EngineConfig],
  summaries: [RepeatedBenchmarkSummary, RepeatedBenchmarkSummary],
  bundles: [HbcBundleInput, HbcBundleInput],
) {
  const [hermesSummary, irisSummary] = summaries;
  assertComparableSummaries(engines, summaries);

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
      bundle: bundles[index],
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
  const checkInputsOnly = hasArg("--check-inputs-only");
  const outputPath = resolve(root, readArg("--output") ?? defaultOutputPath);
  const runs = readNumberArg("--runs", defaultRuns);
  const waitMs = readNumberArg("--wait-ms", defaultWaitMs);
  const engines = createEngines() as [EngineConfig, EngineConfig];
  const bundles = dryRun ? null : assertComparisonInputs(engines);

  if (checkInputsOnly) {
    if (dryRun) {
      assertMatchingHbcBundles(engines);
    }
    console.log("Android engine comparison input check completed.");
    return;
  }

  for (const engine of engines) {
    runEngineBenchmark(engine, runs, waitMs, dryRun);
  }

  if (dryRun) {
    console.log("Dry run completed without running device benchmarks.");
    return;
  }

  writeComparison(
    compareSummaries(
      engines,
      [readSummary(engines[0]), readSummary(engines[1])],
      bundles ?? assertMatchingHbcBundles(engines),
    ),
    outputPath,
  );
}

main();
