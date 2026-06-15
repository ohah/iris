import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, parse, resolve } from "node:path";
import { execFileSync } from "node:child_process";
import type {
  BenchmarkCaseReport,
  BenchmarkMetadata,
  BenchmarkSuiteReport,
} from "../../apps/rn-bench/src/benchmarks/types";
import { strictRnBenchmarkCaseIds, strictRnBenchmarkSuite } from "./strict-rn-benchmark-contract";

const root = resolve(import.meta.dir, "../..");
const defaultApkPath =
  "apps/rn-bench/android/app/build/outputs/apk/hermes/release/app-hermes-release.apk";
const defaultAppId = "com.iris.bench.hermes";
const defaultLogPath = "artifacts/bench/rn-release-hermes.log";
const defaultReportPath = "artifacts/bench/hermes-release-baseline.json";
const defaultSummaryPath = "artifacts/bench/hermes-release-baseline-summary.json";

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

function readArgs(name: string) {
  const prefix = `${name}=`;
  return process.argv
    .filter((arg) => arg.startsWith(prefix))
    .map((arg) => arg.slice(prefix.length));
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

function hasArg(name: string) {
  return process.argv.includes(name);
}

const agentDevice = process.env.AGENT_DEVICE_BIN ?? "agent-device";
const appId = readArg("--app-id") ?? defaultAppId;
const apkPath = resolve(root, readArg("--apk") ?? defaultApkPath);
const logPath = resolve(root, readArg("--log-output") ?? defaultLogPath);
const reportPath = resolve(root, readArg("--report-output") ?? defaultReportPath);
const summaryPath = resolve(root, readArg("--summary-output") ?? defaultSummaryPath);
const session = readArg("--session") ?? "rnbench-android-release";
const waitMs = readNumberArg("--wait-ms", 45_000);
const runs = readNumberArg("--runs", 1);
const allowNonHermes = hasArg("--allow-non-hermes");
const keepSession = hasArg("--keep-session");
const nativeBootstrapArtifact = hasArg("--native-bootstrap-artifact");
const suiteIds = readArgs("--suite-id");
const requiredCases = readArgs("--require-case");

type AgentJson<T> = {
  data: T;
  success: boolean;
};

type DeviceInfo = {
  booted: boolean;
  id: string;
  kind: string;
  name: string;
  platform: string;
  target: string;
};

type LogPathInfo = {
  path: string;
};

type CaseMetricSummary = {
  max: number;
  mean: number;
  min: number;
  samples: number[];
};

type RepeatedCaseSummary = {
  checksum: BenchmarkCaseReport["checksum"];
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
  metadata: BenchmarkMetadata;
  runCount: number;
  schemaVersion: "iris.benchmark.repeated.v1";
  sourceReports: string[];
  suite: BenchmarkSuiteReport["suite"];
};

function commandLine(command: string, args: string[]) {
  return [command, ...args].join(" ");
}

function run(command: string, args: string[]) {
  console.log(`$ ${commandLine(command, args)}`);

  try {
    return execFileSync(command, args, {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
  } catch (error) {
    if (typeof error === "object" && error != null && "code" in error && error.code === "ENOENT") {
      throw new Error(
        `${command} was not found. Run this through mise or set AGENT_DEVICE_BIN to the agent-device binary path.`,
      );
    }

    if (error instanceof Error && "stdout" in error && "stderr" in error) {
      const stdout = error.stdout == null ? "" : String(error.stdout);
      const stderr = error.stderr == null ? "" : String(error.stderr);
      const output = `${stdout}${stderr}`.trim();
      throw new Error(`${commandLine(command, args)} failed.\n${output}`);
    }

    throw error;
  }
}

function readCommand(command: string, args: string[]) {
  console.log(`$ ${commandLine(command, args)}`);

  try {
    return execFileSync(command, args, {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      stdio: ["ignore", "pipe", "pipe"],
    });
  } catch (error) {
    if (error instanceof Error && "stdout" in error && "stderr" in error) {
      const stdout = error.stdout == null ? "" : String(error.stdout);
      const stderr = error.stderr == null ? "" : String(error.stderr);
      const output = `${stdout}${stderr}`.trim();
      throw new Error(`${commandLine(command, args)} failed.\n${output}`);
    }

    throw error;
  }
}

function runPassthrough(command: string, args: string[]) {
  console.log(`$ ${commandLine(command, args)}`);
  execFileSync(command, args, {
    cwd: root,
    stdio: "inherit",
  });
}

function sleep(ms: number) {
  console.log(`Waiting ${ms}ms for benchmark completion.`);
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

function runAgent(args: string[]) {
  return run(agentDevice, args);
}

function parseJsonOutput<T>(output: string): T {
  const jsonStart = output.indexOf("{");
  const jsonEnd = output.lastIndexOf("}");

  if (jsonStart === -1 || jsonEnd === -1 || jsonEnd <= jsonStart) {
    throw new Error(`No JSON payload found in command output:\n${output}`);
  }

  return JSON.parse(output.slice(jsonStart, jsonEnd + 1)) as T;
}

function requireAgentDevice() {
  const output = runAgent(["--version"]);
  const version = output.match(/\d+\.\d+\.\d+/)?.[0];

  if (version == null) {
    throw new Error(`Unable to read agent-device version from output:\n${output}`);
  }

  const [major, minor] = version.split(".").map(Number);
  if (major === 0 && minor < 14) {
    throw new Error(`agent-device >= 0.14.0 is required. Found ${version}.`);
  }
}

function requireAndroidDevice() {
  const output = runAgent(["devices", "--platform", "android", "--json"]);
  const result = parseJsonOutput<AgentJson<{ devices: DeviceInfo[] }>>(output);
  const devices = result.data.devices.filter(
    (device) => device.platform === "android" && device.kind === "device" && device.booted,
  );

  if (devices.length === 0) {
    throw new Error("No booted Android physical device was found by agent-device.");
  }

  const [device] = devices;
  console.log(`Android device: ${device.name} (${device.id})`);
}

function requireReleaseApk() {
  if (!existsSync(apkPath)) {
    throw new Error(`Release APK does not exist: ${apkPath}`);
  }
}

function relativePath(path: string) {
  return path.replace(`${root}/`, "");
}

function pathWithRunIndex(path: string, runIndex: number) {
  const parsedPath = parse(path);
  return resolve(parsedPath.dir, `${parsedPath.name}-run-${runIndex}${parsedPath.ext}`);
}

function summarizeMetric(samples: number[]): CaseMetricSummary {
  if (samples.length === 0) {
    throw new Error("Cannot summarize an empty metric sample set.");
  }

  return {
    max: Math.max(...samples),
    mean: Number(
      (samples.reduce((total, sample) => total + sample, 0) / samples.length).toFixed(3),
    ),
    min: Math.min(...samples),
    samples,
  };
}

function checksumKey(checksum: BenchmarkCaseReport["checksum"]) {
  return `${typeof checksum}:${JSON.stringify(checksum)}`;
}

function extractBenchmarkReport(inputLogPath: string, outputReportPath: string) {
  const selectedSuiteIds =
    suiteIds.length > 0
      ? suiteIds
      : nativeBootstrapArtifact
        ? ["iris-engine-bootstrap"]
        : [strictRnBenchmarkSuite.id];
  const selectedRequiredCases =
    requiredCases.length > 0
      ? requiredCases
      : nativeBootstrapArtifact
        ? [
            "iris-hbc-metadata-parse",
            "iris-hbc-static-coverage-scan",
            "iris-hbc-scalar-execution-frontier",
          ]
        : [...strictRnBenchmarkCaseIds];
  const extractionArgs = [
    "run",
    "tools/bench/extract-hermes-report.ts",
    `--input=${inputLogPath}`,
    `--output=${outputReportPath}`,
    ...(allowNonHermes ? ["--allow-non-hermes"] : []),
    "--require-release",
    ...selectedSuiteIds.map((suiteId) => `--suite-id=${suiteId}`),
    ...selectedRequiredCases.map((caseId) => `--require-case=${caseId}`),
  ];

  if (!nativeBootstrapArtifact) {
    extractionArgs.push("--require-new-architecture");
  }

  runPassthrough("bun", extractionArgs);

  return JSON.parse(readFileSync(outputReportPath, "utf8")) as BenchmarkSuiteReport;
}

function summarizeReports(reports: BenchmarkSuiteReport[], sourceReports: string[]) {
  if (reports.length === 0) {
    throw new Error("Cannot summarize an empty report set.");
  }

  const firstReport = reports[0];
  const caseIds = firstReport.cases.map((benchmarkCase) => benchmarkCase.id);

  for (const report of reports) {
    if (report.suite.id !== firstReport.suite.id || report.suite.name !== firstReport.suite.name) {
      throw new Error(
        "Repeated Android benchmark reports came from different suites.\n" +
          `first=${firstReport.suite.id} (${firstReport.suite.name})\n` +
          `next=${report.suite.id} (${report.suite.name})`,
      );
    }
  }

  const cases = caseIds.map((caseId): RepeatedCaseSummary => {
    const matchingCases = reports.map((report) => {
      const benchmarkCase = report.cases.find((candidate) => candidate.id === caseId);

      if (benchmarkCase == null) {
        throw new Error(`Report is missing case ${caseId}.`);
      }

      return benchmarkCase;
    });
    const [firstCase] = matchingCases as [BenchmarkCaseReport, ...BenchmarkCaseReport[]];
    const firstChecksumKey = checksumKey(firstCase.checksum);
    const mismatchedChecksum = matchingCases.find(
      (benchmarkCase) => checksumKey(benchmarkCase.checksum) !== firstChecksumKey,
    );

    if (mismatchedChecksum != null) {
      throw new Error(
        `${caseId} checksum changed across repeated Android benchmark runs: ` +
          `first=${JSON.stringify(firstCase.checksum)} next=${JSON.stringify(
            mismatchedChecksum.checksum,
          )}`,
      );
    }

    return {
      checksum: firstCase.checksum,
      id: caseId,
      label: firstCase.label,
      p50: summarizeMetric(matchingCases.map((benchmarkCase) => benchmarkCase.stats.p50)),
      p95: summarizeMetric(matchingCases.map((benchmarkCase) => benchmarkCase.stats.p95)),
      unit: firstCase.unit,
    };
  });

  return {
    cases,
    createdAt: new Date().toISOString(),
    generatedBy: "tools/bench/run-android-release-benchmark.ts",
    metadata: firstReport.metadata,
    runCount: reports.length,
    schemaVersion: "iris.benchmark.repeated.v1",
    sourceReports: sourceReports.map((sourceReport) => relativePath(sourceReport)),
    suite: firstReport.suite,
  } satisfies RepeatedBenchmarkSummary;
}

function writeRepeatedSummary(reports: BenchmarkSuiteReport[], sourceReports: string[]) {
  const summary = summarizeReports(reports, sourceReports);

  mkdirSync(dirname(summaryPath), { recursive: true });
  writeFileSync(summaryPath, `${JSON.stringify(summary, null, 2)}\n`);

  console.log(`Repeated Android release benchmark summary: ${summaryPath}`);
  for (const result of summary.cases) {
    console.log(
      `${result.id}: p50(mean=${result.p50.mean}${result.unit}, min=${result.p50.min}${result.unit}, max=${result.p50.max}${result.unit}) p95(mean=${result.p95.mean}${result.unit}, min=${result.p95.min}${result.unit}, max=${result.p95.max}${result.unit})`,
    );
  }
}

function closeSession(sessionName: string) {
  try {
    runAgent(["close", "--session", sessionName, "--platform", "android"]);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);

    if (!message.includes("SESSION_NOT_FOUND")) {
      console.warn(message);
    }
  }
}

function openApp() {
  const openArgs = ["open", appId, "--session", session, "--platform", "android", "--relaunch"];

  try {
    runAgent(openArgs);
    return;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const conflictingSession = message.match(/session "([^"]+)"/)?.[1];

    if (!message.includes("DEVICE_IN_USE") || conflictingSession == null) {
      throw error;
    }

    console.warn(`Closing conflicting agent-device session: ${conflictingSession}`);
    closeSession(conflictingSession);
    runAgent(openArgs);
  }
}

function runBenchmarkIteration(runIndex: number) {
  const nextLogPath = runs === 1 ? logPath : pathWithRunIndex(logPath, runIndex);
  const nextReportPath = runs === 1 ? reportPath : pathWithRunIndex(reportPath, runIndex);

  console.log(`Android release benchmark run ${runIndex}/${runs}`);
  if (nativeBootstrapArtifact) {
    run("adb", ["logcat", "-c"]);
    closeSession(session);
    openApp();
    sleep(waitMs);
    writeFileSync(nextLogPath, readCommand("adb", ["logcat", "-d", "-v", "time"]));

    return {
      report: extractBenchmarkReport(nextLogPath, nextReportPath),
      reportPath: nextReportPath,
    };
  }

  runAgent(["wait", 'label="Run suite"', "10000", "--session", session, "--platform", "android"]);
  runAgent(["logs", "clear", "--restart", "--session", session, "--platform", "android"]);
  runAgent([
    "logs",
    "mark",
    `before Android release benchmark run ${runIndex}`,
    "--session",
    session,
    "--platform",
    "android",
  ]);
  runAgent(["press", 'label="Run suite"', "--session", session, "--platform", "android"]);
  sleep(waitMs);
  runAgent(["wait", 'label="Run suite"', "10000", "--session", session, "--platform", "android"]);

  const logPathOutput = runAgent([
    "logs",
    "path",
    "--session",
    session,
    "--platform",
    "android",
    "--json",
  ]);
  const logPathResult = parseJsonOutput<AgentJson<LogPathInfo>>(logPathOutput);
  copyFileSync(logPathResult.data.path, nextLogPath);

  return {
    report: extractBenchmarkReport(nextLogPath, nextReportPath),
    reportPath: nextReportPath,
  };
}

function runBenchmark() {
  runAgent(["install", appId, apkPath, "--platform", "android"]);
  closeSession(session);
  if (!nativeBootstrapArtifact) {
    openApp();
  }

  const reports: BenchmarkSuiteReport[] = [];
  const sourceReports: string[] = [];

  for (let runIndex = 1; runIndex <= runs; runIndex += 1) {
    const result = runBenchmarkIteration(runIndex);
    reports.push(result.report);
    sourceReports.push(result.reportPath);
  }

  if (runs > 1 || readArg("--summary-output") != null) {
    writeRepeatedSummary(reports, sourceReports);
  }
}

function main() {
  requireAgentDevice();
  requireAndroidDevice();
  requireReleaseApk();

  mkdirSync(dirname(logPath), { recursive: true });
  mkdirSync(dirname(reportPath), { recursive: true });
  mkdirSync(dirname(summaryPath), { recursive: true });

  try {
    runBenchmark();
  } finally {
    if (!keepSession) {
      closeSession(session);
    }
  }
}

main();
