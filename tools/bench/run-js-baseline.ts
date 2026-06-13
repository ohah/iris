import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { benchmarkCases } from "../../apps/rn-bench/src/benchmarks/cases";
import { runBenchmarkSuite } from "../../apps/rn-bench/src/benchmarks/harness";
import type { BenchmarkMetadata } from "../../apps/rn-bench/src/benchmarks/types";

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

function hasArg(name: string) {
  return process.argv.includes(name);
}

function readGitValue(args: string[]) {
  try {
    return execFileSync("git", args, {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return "unknown";
  }
}

const root = resolve(import.meta.dir, "../..");
const smoke = hasArg("--smoke");
const outputPath = resolve(
  root,
  readArg("--output") ??
    (smoke ? "artifacts/bench/js-baseline-smoke.json" : "artifacts/bench/js-baseline.json"),
);
const commit = readGitValue(["rev-parse", "HEAD"]);
const shortCommit = readGitValue(["rev-parse", "--short", "HEAD"]);

const metadata: BenchmarkMetadata = {
  app: {
    name: "IrisBench",
    version: "0.0.1",
  },
  build: {
    commit,
    mode: "unknown",
    source: `git:${shortCommit}`,
  },
  platform: {
    device: process.env.RUNNER_OS ? "github-actions" : "local",
    os: `${process.platform}-${process.arch}`,
    version: process.version,
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

const report = await runBenchmarkSuite(benchmarkCases, metadata, {
  artifact: {
    generatedBy: "tools/bench/run-js-baseline.ts",
    kind: "file",
    path: outputPath.replace(`${root}/`, ""),
  },
  measuredIterations: smoke ? 3 : undefined,
  warmupIterations: smoke ? 1 : undefined,
});

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(report, null, 2)}\n`);

console.log(`benchmark artifact: ${outputPath}`);
for (const result of report.cases) {
  console.log(
    `${result.id}: p50=${result.stats.p50}${result.unit} p95=${result.stats.p95}${result.unit}`,
  );
}
