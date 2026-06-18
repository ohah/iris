import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { basename, dirname, resolve } from "node:path";

const root = resolve(import.meta.dir, "../..");
const defaultCaseDir = "tools/bench/strict-hbc/cases";
const defaultOutputPath = "artifacts/bench/strict-hbc-engine-comparison.json";
const defaultHbcDir = "artifacts/bench/strict-hbc";
const defaultHermesFrameworkDir =
  "apps/rn-bench/ios/Pods/hermes-engine/destroot/Library/Frameworks/macosx";
const defaultHermesc = "apps/rn-bench/node_modules/hermes-compiler/hermesc/osx-bin/hermesc";

type EngineRun = {
  engine: "hermes" | "iris";
  casePath: string;
  value: boolean | number | string | null;
  warmupIterations: number;
  measuredIterations: number;
  samplesMs: number[];
  declaredGlobals?: number;
};

type EngineName = EngineRun["engine"];
type EngineOrder = "alternate" | "hermes-first" | "iris-first";

type Stats = {
  max: number;
  mean: number;
  min: number;
  p50: number;
  p95: number;
};

type ComparisonCase = {
  id: string;
  sourcePath: string;
  hbcPath: string;
  checksum: {
    hermes: EngineRun["value"];
    iris: EngineRun["value"];
    matches: boolean;
  };
  hermes: Stats & {
    samplesMs: number[];
  };
  iris: Stats & {
    samplesMs: number[];
  };
  p50IrisOverHermes: number | null;
  p95IrisOverHermes: number | null;
  strictComparable: true;
};

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

function readNumberArg(name: string, fallback: number, allowZero = false) {
  const rawValue = readArg(name);
  if (rawValue == null) {
    return fallback;
  }

  const value = Number(rawValue);
  if (!Number.isInteger(value) || value < 0 || (!allowZero && value === 0)) {
    throw new Error(`${name} must be ${allowZero ? "a non-negative" : "a positive"} integer.`);
  }
  return value;
}

function relativePath(path: string) {
  return path.replace(`${root}/`, "");
}

function shellLine(command: string, args: string[]) {
  return [command, ...args.map((arg) => (arg.includes(" ") ? JSON.stringify(arg) : arg))].join(" ");
}

function run(command: string, args: string[]) {
  console.log(`$ ${shellLine(command, args)}`);
  execFileSync(command, args, {
    cwd: root,
    stdio: "inherit",
  });
}

function captureJson<T>(command: string, args: string[]) {
  console.log(`$ ${shellLine(command, args)}`);
  const output = execFileSync(command, args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
    stdio: ["ignore", "pipe", "inherit"],
  }).trim();

  return JSON.parse(output) as T;
}

function ensureFreshHermesRunner(sourcePath: string, outputPath: string, frameworkDir: string) {
  if (process.platform !== "darwin") {
    throw new Error(
      "Hermes strict HBC runner currently requires macOS Hermes framework artifacts.",
    );
  }
  if (!existsSync(frameworkDir)) {
    throw new Error(
      `Hermes framework directory does not exist: ${frameworkDir}\n` +
        "Run the RN iOS dependency setup before strict host-side HBC comparison.",
    );
  }

  const needsBuild =
    !existsSync(outputPath) || statSync(sourcePath).mtimeMs > statSync(outputPath).mtimeMs;
  if (!needsBuild) {
    return;
  }

  mkdirSync(dirname(outputPath), { recursive: true });
  run("xcrun", [
    "clang++",
    "-std=c++17",
    sourcePath,
    "-I",
    resolve(root, "apps/rn-bench/ios/Pods/hermes-engine/destroot/include"),
    "-F",
    frameworkDir,
    "-framework",
    "hermesvm",
    `-Wl,-rpath,${frameworkDir}`,
    "-o",
    outputPath,
  ]);
}

function resolveCasePath(input: string) {
  if (input.endsWith(".js") || input.includes("/")) {
    return resolve(root, input);
  }
  return resolve(root, defaultCaseDir, `${input}.js`);
}

function defaultCasePaths() {
  const caseDir = resolve(root, defaultCaseDir);
  return readdirSync(caseDir)
    .filter((entry) => entry.endsWith(".js"))
    .filter((entry) => !entry.endsWith("-lexical.js"))
    .filter((entry) => !entry.endsWith("-diagnostic.js"))
    .sort()
    .map((entry) => resolve(caseDir, entry));
}

function caseId(path: string) {
  return basename(path, ".js");
}

function compileHbc(hermesc: string, sourcePath: string, hbcDir: string) {
  const outputPath = resolve(hbcDir, `${caseId(sourcePath)}.hbc`);
  mkdirSync(dirname(outputPath), { recursive: true });
  run(hermesc, ["-O", "-emit-binary", "-out", outputPath, sourcePath]);
  return outputPath;
}

function round(value: number) {
  return Number(value.toFixed(6));
}

function percentile(sortedSamples: number[], percentileValue: number) {
  const rank = Math.ceil((percentileValue / 100) * sortedSamples.length) - 1;
  return sortedSamples[Math.max(0, Math.min(sortedSamples.length - 1, rank))] ?? 0;
}

function summarize(samples: number[]): Stats {
  const sortedSamples = [...samples].sort((left, right) => left - right);
  const total = sortedSamples.reduce((sum, value) => sum + value, 0);

  return {
    max: round(sortedSamples[sortedSamples.length - 1] ?? 0),
    mean: round(total / (sortedSamples.length || 1)),
    min: round(sortedSamples[0] ?? 0),
    p50: round(percentile(sortedSamples, 50)),
    p95: round(percentile(sortedSamples, 95)),
  };
}

function ratio(numerator: number, denominator: number) {
  if (denominator === 0) {
    return null;
  }
  return Number((numerator / denominator).toFixed(4));
}

function sameValue(left: unknown, right: unknown) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function resolveCargo() {
  const configuredCargo = readArg("--cargo");
  if (configuredCargo != null) {
    return configuredCargo.includes("/") ? resolve(root, configuredCargo) : configuredCargo;
  }

  const cargoHomePath = resolve(homedir(), ".cargo/bin/cargo");
  return existsSync(cargoHomePath) ? cargoHomePath : "cargo";
}

function compareCase(
  sourcePath: string,
  hbcPath: string,
  hermesRun: EngineRun,
  irisRun: EngineRun,
): ComparisonCase {
  if (hermesRun.measuredIterations !== irisRun.measuredIterations) {
    throw new Error(
      `${caseId(sourcePath)} measured iterations differ: hermes=${hermesRun.measuredIterations} iris=${irisRun.measuredIterations}`,
    );
  }

  const hermesStats = summarize(hermesRun.samplesMs);
  const irisStats = summarize(irisRun.samplesMs);
  const checksumMatches = sameValue(hermesRun.value, irisRun.value);
  if (!checksumMatches) {
    throw new Error(
      `${caseId(sourcePath)} checksum mismatch: hermes=${JSON.stringify(hermesRun.value)} iris=${JSON.stringify(irisRun.value)}`,
    );
  }

  return {
    id: caseId(sourcePath),
    sourcePath: relativePath(sourcePath),
    hbcPath: relativePath(hbcPath),
    checksum: {
      hermes: hermesRun.value,
      iris: irisRun.value,
      matches: checksumMatches,
    },
    hermes: {
      ...hermesStats,
      samplesMs: hermesRun.samplesMs.map(round),
    },
    iris: {
      ...irisStats,
      samplesMs: irisRun.samplesMs.map(round),
    },
    p50IrisOverHermes: ratio(irisStats.p50, hermesStats.p50),
    p95IrisOverHermes: ratio(irisStats.p95, hermesStats.p95),
    strictComparable: true,
  };
}

function readEngineOrder(rounds: number): EngineOrder {
  const rawOrder = readArg("--engine-order") ?? (rounds > 1 ? "alternate" : "hermes-first");
  if (rawOrder !== "alternate" && rawOrder !== "hermes-first" && rawOrder !== "iris-first") {
    throw new Error("--engine-order must be one of alternate, hermes-first, or iris-first.");
  }
  return rawOrder;
}

function engineOrderForRound(engineOrder: EngineOrder, roundIndex: number): EngineName[] {
  if (engineOrder === "hermes-first") {
    return ["hermes", "iris"];
  }
  if (engineOrder === "iris-first") {
    return ["iris", "hermes"];
  }
  return roundIndex % 2 === 0 ? ["hermes", "iris"] : ["iris", "hermes"];
}

function mergeEngineRuns(engine: EngineName, runs: EngineRun[]): EngineRun {
  if (runs.length === 0) {
    throw new Error(`missing ${engine} run samples.`);
  }
  const [firstRun] = runs;
  for (const run of runs.slice(1)) {
    if (!sameValue(firstRun.value, run.value)) {
      throw new Error(
        `${engine} repeated checksum mismatch: first=${JSON.stringify(firstRun.value)} next=${JSON.stringify(run.value)}`,
      );
    }
    if (firstRun.casePath !== run.casePath) {
      throw new Error(
        `${engine} repeated case mismatch: first=${firstRun.casePath} next=${run.casePath}`,
      );
    }
  }

  return {
    engine,
    casePath: firstRun.casePath,
    value: firstRun.value,
    warmupIterations: runs.reduce((sum, run) => sum + run.warmupIterations, 0),
    measuredIterations: runs.reduce((sum, run) => sum + run.measuredIterations, 0),
    samplesMs: runs.flatMap((run) => run.samplesMs),
    declaredGlobals: firstRun.declaredGlobals,
  };
}

function main() {
  const warmupIterations = readNumberArg("--warmup", 3, true);
  const measuredIterations = readNumberArg("--iterations", 20);
  const rounds = readNumberArg("--rounds", 1);
  const engineOrder = readEngineOrder(rounds);
  const outputPath = resolve(root, readArg("--output") ?? defaultOutputPath);
  const hbcDir = resolve(root, readArg("--hbc-dir") ?? defaultHbcDir);
  const hermesc = resolve(root, readArg("--hermesc") ?? defaultHermesc);
  const hermesRunnerSource = resolve(root, "tools/bench/strict-hbc/hermes-runner.cpp");
  const hermesRunner = resolve(
    root,
    readArg("--hermes-runner") ?? "target/bench/hermes-hbc-runner",
  );
  const hermesFrameworkDir = resolve(
    root,
    readArg("--hermes-framework-dir") ?? defaultHermesFrameworkDir,
  );
  const cargo = resolveCargo();
  const caseInputs = readArgs("--case");
  const casePaths = caseInputs.length > 0 ? caseInputs.map(resolveCasePath) : defaultCasePaths();

  if (!existsSync(hermesc)) {
    throw new Error(`hermesc does not exist: ${hermesc}`);
  }
  for (const sourcePath of casePaths) {
    if (!existsSync(sourcePath)) {
      throw new Error(`strict HBC case does not exist: ${sourcePath}`);
    }
  }

  ensureFreshHermesRunner(hermesRunnerSource, hermesRunner, hermesFrameworkDir);
  run(cargo, ["build", "--release", "-p", "iris-hbc", "--bin", "hbc-bench"]);

  const compiledCases = casePaths.map((sourcePath) => ({
    sourcePath,
    hbcPath: compileHbc(hermesc, sourcePath, hbcDir),
    runs: {
      hermes: [],
      iris: [],
    } satisfies Record<EngineName, EngineRun[]>,
  }));
  for (let roundIndex = 0; roundIndex < rounds; roundIndex += 1) {
    const roundCases = roundIndex % 2 === 0 ? compiledCases : [...compiledCases].reverse();
    for (const compiledCase of roundCases) {
      for (const engine of engineOrderForRound(engineOrder, roundIndex)) {
        const runner =
          engine === "hermes" ? hermesRunner : resolve(root, "target/release/hbc-bench");
        compiledCase.runs[engine].push(
          captureJson<EngineRun>(runner, [
            `--warmup=${warmupIterations}`,
            `--iterations=${measuredIterations}`,
            compiledCase.hbcPath,
          ]),
        );
      }
    }
  }

  const cases = compiledCases.map(({ sourcePath, hbcPath, runs }) => {
    const hermesRun = mergeEngineRuns("hermes", runs.hermes);
    const irisRun = mergeEngineRuns("iris", runs.iris);
    const comparison = compareCase(sourcePath, hbcPath, hermesRun, irisRun);

    console.log(
      `${comparison.id}: p50 iris/hermes=${comparison.p50IrisOverHermes} p95 iris/hermes=${comparison.p95IrisOverHermes} checksum=${JSON.stringify(comparison.checksum.iris)}`,
    );

    return comparison;
  });

  const report = {
    schemaVersion: "iris.benchmark.strict-hbc-engine-comparison.v1",
    generatedBy: "tools/bench/run-strict-hbc-engine-comparison.ts",
    createdAt: new Date().toISOString(),
    methodology: {
      input:
        "Each case is compiled once to Hermes bytecode and both engines run the same HBC file.",
      hermes:
        "Hermes framework runner prepares HBC once, then evaluates prepared bytecode for each sample.",
      iris: "Iris parses HBC once, then executes the global function with a fresh scalar executor state for each sample.",
      scope:
        "Host-side strict HBC microbenchmark, not a full React Native runtime replacement benchmark.",
      order:
        "When rounds is greater than one, engine execution order can be alternated and case order is reversed on odd rounds to reduce first-run and thermal bias.",
    },
    warmupIterations,
    measuredIterations,
    rounds,
    engineOrder,
    totalMeasuredIterations: measuredIterations * rounds,
    engines: {
      hermes: {
        frameworkDir: relativePath(hermesFrameworkDir),
        runner: relativePath(hermesRunner),
      },
      iris: {
        runner: "target/release/hbc-bench",
      },
    },
    cases,
  };

  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, `${JSON.stringify(report, null, 2)}\n`);
  console.log(`Strict HBC engine comparison artifact: ${outputPath}`);
}

main();
