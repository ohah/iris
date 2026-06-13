import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { execFileSync } from "node:child_process";

const root = resolve(import.meta.dir, "../..");
const appId = "com.iris.bench";
const defaultApkPath = "apps/rn-bench/android/app/build/outputs/apk/release/app-release.apk";
const defaultLogPath = "artifacts/bench/rn-release-hermes.log";
const defaultReportPath = "artifacts/bench/hermes-release-baseline.json";

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
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
const apkPath = resolve(root, readArg("--apk") ?? defaultApkPath);
const logPath = resolve(root, readArg("--log-output") ?? defaultLogPath);
const reportPath = resolve(root, readArg("--report-output") ?? defaultReportPath);
const session = readArg("--session") ?? "rnbench-android-release";
const waitMs = readNumberArg("--wait-ms", 45_000);
const keepSession = hasArg("--keep-session");

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

function extractBenchmarkReport() {
  runPassthrough("bun", [
    "run",
    "tools/bench/extract-hermes-report.ts",
    `--input=${logPath}`,
    `--output=${reportPath}`,
    "--require-release",
    "--require-new-architecture",
    "--require-case=turbomodule-number-round-trip",
    "--require-case=turbomodule-string-round-trip",
  ]);
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

function runBenchmark() {
  runAgent(["install", appId, apkPath, "--platform", "android"]);
  closeSession(session);
  openApp();
  runAgent(["wait", 'label="Run suite"', "10000", "--session", session, "--platform", "android"]);
  runAgent(["logs", "clear", "--restart", "--session", session, "--platform", "android"]);
  runAgent([
    "logs",
    "mark",
    "before Android release benchmark",
    "--session",
    session,
    "--platform",
    "android",
  ]);
  runAgent(["press", 'label="Run suite"', "--session", session, "--platform", "android"]);
  sleep(waitMs);

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
  copyFileSync(logPathResult.data.path, logPath);

  extractBenchmarkReport();
}

function main() {
  requireAgentDevice();
  requireAndroidDevice();
  requireReleaseApk();

  mkdirSync(dirname(logPath), { recursive: true });
  mkdirSync(dirname(reportPath), { recursive: true });

  try {
    runBenchmark();
  } finally {
    if (!keepSession) {
      closeSession(session);
    }
  }
}

main();
